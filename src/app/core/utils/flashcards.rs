// SPDX-License-Identifier: GPL-3.0

use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use fsrs::MemoryState;

use crate::app::core::{
    models::flashcard::{Flashcard, FlashcardField, FlashcardStatus},
    utils::fsrs_scheduler::FSRSScheduler,
};

/// Custom Import into Oboete Flashcards
pub fn parse_import_content(
    line_delimiter: &str,
    term_delimiter: &str,
    content: &str,
) -> Vec<Flashcard> {
    let line_delimiter = unescape_delimiter(line_delimiter);
    let term_delimiter = unescape_delimiter(term_delimiter);

    content
        .split(line_delimiter.as_str())
        .filter_map(|line| {
            let mut terms = line.split(term_delimiter.as_str());
            if let (Some(front), Some(back)) = (terms.next(), terms.next()) {
                Some(Flashcard {
                    id: None,
                    front: FlashcardField::Text(front.to_string()),
                    back: FlashcardField::Text(back.to_string()),
                    status: FlashcardStatus::None,
                    ..Default::default()
                })
            } else {
                None
            }
        })
        .collect()
}

/// Lets the user type `\n` and `\t` in the single-line delimiter inputs to mean a new line or a tab
fn unescape_delimiter(delimiter: &str) -> String {
    delimiter.replace("\\n", "\n").replace("\\t", "\t")
}

/// Given a path to an anki export file parses it to Flashcards in Oboete
pub fn parse_ankifile(file_path: &Path) -> Result<Vec<Flashcard>, anywho::Error> {
    let content = fs::read_to_string(file_path)?;
    Ok(parse_anki_content(&content))
}

/// Parses the contents of an anki plain text export, supports the `#separator` and `#html` headers and quoted fields
fn parse_anki_content(content: &str) -> Vec<Flashcard> {
    let mut separator = '\t';
    let mut html = false;
    let mut body = content;

    while body.starts_with('#') {
        let (line, rest) = body.split_once('\n').unwrap_or((body, ""));
        let line = line.trim_end_matches('\r');
        if let Some(value) = line.strip_prefix("#separator:") {
            separator = match value.to_lowercase().as_str() {
                "tab" => '\t',
                "comma" => ',',
                "semicolon" => ';',
                "space" => ' ',
                "pipe" => '|',
                "colon" => ':',
                other => other.chars().next().unwrap_or('\t'),
            };
        } else if let Some(value) = line.strip_prefix("#html:") {
            html = value.eq_ignore_ascii_case("true");
        }
        body = rest;
    }

    parse_separated_records(body, separator)
        .into_iter()
        .filter_map(|fields| match <[String; 2]>::try_from(fields) {
            Ok([front, back]) => {
                let (front, back) = if html {
                    (html_to_text(&front), html_to_text(&back))
                } else {
                    (front, back)
                };
                Some(Flashcard {
                    id: None,
                    front: FlashcardField::Text(front),
                    back: FlashcardField::Text(back),
                    status: FlashcardStatus::None,
                    ..Default::default()
                })
            }
            Err(_) => None,
        })
        .collect()
}

/// Splits the given text into records of fields, fields starting with a quote can contain the separator, newlines and doubled quotes
fn parse_separated_records(input: &str, separator: char) -> Vec<Vec<String>> {
    let mut records = Vec::new();
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut field_start = true;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(c);
            }
            continue;
        }

        match c {
            '"' if field_start => {
                in_quotes = true;
                field_start = false;
            }
            '\r' if chars.peek() == Some(&'\n') => {}
            '\n' => {
                fields.push(std::mem::take(&mut field));
                if !(fields.len() == 1 && fields[0].is_empty()) {
                    records.push(std::mem::take(&mut fields));
                }
                fields.clear();
                field_start = true;
            }
            c if c == separator => {
                fields.push(std::mem::take(&mut field));
                field_start = true;
            }
            c => {
                field.push(c);
                field_start = false;
            }
        }
    }

    if !field.is_empty() || !fields.is_empty() {
        fields.push(field);
        records.push(fields);
    }

    records
}

/// Converts the basic html anki adds to its exports into plain text
fn html_to_text(html: &str) -> String {
    html.replace("<br />", "\n")
        .replace("<br/>", "\n")
        .replace("<br>", "\n")
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// Quotes a field for an anki export if it contains characters that would break the format
fn anki_field(field: &str) -> String {
    if field.contains(['\t', '\n', '\r', '"']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Given a path to save the file and a Vec<Flashcard> creates a file with the flashcards data
pub fn export_flashcards(file_path: &Path, flashcards: &[Flashcard]) -> Result<(), anywho::Error> {
    let mut file = File::create(file_path)?;

    for flashcard in flashcards {
        writeln!(
            file,
            "{}\\#*#\\{}",
            flashcard.front.display_text(),
            flashcard.back.display_text()
        )?;
        writeln!(file, "/#")?;
    }

    Ok(())
}

/// Given a path to save the file and a Vec<Flashcard> creates a file compatible with Anki to import the flashcards
pub fn export_flashcards_anki(
    file_path: &Path,
    flashcards: &[Flashcard],
) -> Result<(), anywho::Error> {
    let mut file = if file_path.extension().is_some_and(|ext| ext == "txt") {
        File::create(file_path)?
    } else {
        let mut with_extension = file_path.as_os_str().to_owned();
        with_extension.push(".txt");
        File::create(with_extension)?
    };

    writeln!(file, "#separator:tab")?;
    writeln!(file, "#html:false")?;
    writeln!(file)?; //adds a \n before the flashcards

    for flashcard in flashcards {
        writeln!(
            file,
            "{}\t{}",
            anki_field(flashcard.front.display_text()),
            anki_field(flashcard.back.display_text())
        )?;
    }

    Ok(())
}

// Helper function to get current day since epoch
pub fn current_day() -> i32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    (duration.as_secs() / 86400) as i32
}

// Helper function to generate updated FSRS data for the given flashcard
pub fn update_fsrs_data(
    selected_state: &FlashcardStatus,
    flashcard: &Flashcard,
    scheduler: &FSRSScheduler,
) -> Option<(MemoryState, i32)> {
    // Calculate days elapsed since last review
    let current = current_day();
    let days_elapsed = flashcard
        .last_reviewed
        .map(|last| (current - last).max(0) as u32)
        .unwrap_or(0);

    // Get next states from FSRS
    let next_states_result = scheduler.get_next_states(
        flashcard.fsrs_state.clone().map(MemoryState::from),
        days_elapsed,
    );

    let (new_memory_state, interval_days) = match next_states_result {
        Ok(next_states) => {
            // Select the appropriate state based on user rating
            let selected_state = match selected_state {
                FlashcardStatus::Bad => next_states.again,
                FlashcardStatus::Ok => next_states.hard,
                FlashcardStatus::Great => next_states.good,
                FlashcardStatus::Easy => next_states.easy,
                FlashcardStatus::None => return None,
            };

            // Extract memory state and interval from selected state
            (selected_state.memory, selected_state.interval)
        }
        Err(e) => {
            eprintln!("FSRS error: {}", e);
            // Fallback: simple interval based on rating
            let interval = match selected_state {
                FlashcardStatus::Bad => 1,
                FlashcardStatus::Ok => 3,
                FlashcardStatus::Great => 7,
                FlashcardStatus::Easy => 12,
                FlashcardStatus::None => 1,
            };
            // Use a default memory state if FSRS fails
            (
                fsrs::MemoryState {
                    stability: interval as f32,
                    difficulty: 5.0,
                },
                interval as f32,
            )
        }
    };

    // Force minimum 1 day interval to prevent infinite same-day loops
    let effective_interval = (interval_days.round() as i32).max(1);
    let new_due_date = current + effective_interval;

    Some((new_memory_state, new_due_date))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(cards: &[Flashcard]) -> Vec<(String, String)> {
        cards
            .iter()
            .map(|c| {
                (
                    c.front.display_text().to_string(),
                    c.back.display_text().to_string(),
                )
            })
            .collect()
    }

    fn pair(a: &str, b: &str) -> (String, String) {
        (a.to_string(), b.to_string())
    }

    #[test]
    fn parses_oboete_style_export_with_blank_line() {
        let cards = parse_anki_content("#separator:tab\n#html:false\n\nCat\tGato\nDog\tPerro\n");
        assert_eq!(
            texts(&cards),
            vec![pair("Cat", "Gato"), pair("Dog", "Perro")]
        );
    }

    #[test]
    fn keeps_first_card_when_there_is_no_blank_line() {
        let cards = parse_anki_content("#separator:tab\n#html:false\nCat\tGato\n");
        assert_eq!(texts(&cards), vec![pair("Cat", "Gato")]);
    }

    #[test]
    fn parses_quoted_multiline_fields_and_doubled_quotes() {
        let cards = parse_anki_content(
            "#separator:tab\n#html:false\n\"el gato\nla gata\"\t\"the \"\"cat\"\"\"\r\nDog\tPerro\r\n",
        );
        assert_eq!(
            texts(&cards),
            vec![
                pair("el gato\nla gata", "the \"cat\""),
                pair("Dog", "Perro")
            ]
        );
    }

    #[test]
    fn supports_other_separators_and_html() {
        let cards = parse_anki_content("#separator:Semicolon\n#html:true\na<br>b;\"x &amp; y\"\n");
        assert_eq!(texts(&cards), vec![pair("a\nb", "x & y")]);
    }

    #[test]
    fn skips_rows_without_exactly_two_fields() {
        let cards = parse_anki_content("only one\na\tb\tc\nok\tfine");
        assert_eq!(texts(&cards), vec![pair("ok", "fine")]);
    }

    #[test]
    fn export_round_trips_through_import() {
        let card = Flashcard {
            front: FlashcardField::Text("line 1\nline \"2\"\twith tab".to_string()),
            back: FlashcardField::Text("plain".to_string()),
            ..Default::default()
        };
        let dir = std::env::temp_dir().join(format!("oboete-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cards.txt");
        export_flashcards_anki(&path, std::slice::from_ref(&card)).unwrap();
        let cards = parse_ankifile(&path).unwrap();
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(
            texts(&cards),
            vec![pair("line 1\nline \"2\"\twith tab", "plain")]
        );
    }

    #[test]
    fn custom_import_supports_newline_and_tab_delimiters() {
        let cards = parse_import_content("\\n", "\\t", "Cat\tGato\nDog\tPerro");
        assert_eq!(
            texts(&cards),
            vec![pair("Cat", "Gato"), pair("Dog", "Perro")]
        );
        let cards = parse_import_content(";", "-", "Cat-Gato;Dog-Perro");
        assert_eq!(
            texts(&cards),
            vec![pair("Cat", "Gato"), pair("Dog", "Perro")]
        );
    }
}

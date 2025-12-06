// SPDX-License-Identifier: GPL-3.0

use std::{
    fs::File,
    io::{self, BufRead, Write},
    path::Path,
};

use fsrs::MemoryState;
use percent_encoding::percent_decode_str;

use crate::app::core::{
    models::flashcard::{Flashcard, FlashcardField, FlashcardStatus},
    utils::fsrs_scheduler::FSRSScheduler,
};

/// Custom Import into Oboete Flashcards
pub fn parse_import_content(
    line_delimiter: &String,
    term_delimiter: &String,
    content: &str,
) -> Vec<Flashcard> {
    content
        .split(line_delimiter)
        .filter_map(|line| {
            let mut terms = line.split(term_delimiter);
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

/// Given a path to an anki export file parses it to Flashcards in Oboete
pub fn parse_ankifile(file_path: &str) -> Result<Vec<Flashcard>, anywho::Error> {
    let decoded_path = percent_decode_str(file_path)
        .decode_utf8_lossy()
        .to_string();
    let path = Path::new(&decoded_path);
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut flashcards = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        // Skip the first three lines which are metadata
        if index < 3 {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() == 2 {
            flashcards.push(Flashcard {
                id: None,
                front: FlashcardField::Text(parts[0].to_string()),
                back: FlashcardField::Text(parts[1].to_string()),
                status: FlashcardStatus::None,
                ..Default::default()
            });
        }
    }

    Ok(flashcards)
}

/// Given a path to save the file and a Vec<Flashcard> creates a file with the flashcards data
pub fn export_flashcards(file_path: &str, flashcards: &[Flashcard]) -> Result<(), anywho::Error> {
    let mut file = File::create(file_path)?;

    for flashcard in flashcards {
        let front = match &flashcard.front {
            FlashcardField::Text(t) => t,
            FlashcardField::Image {
                path: _path,
                alt_text,
            } => alt_text,
        };

        let back = match &flashcard.back {
            FlashcardField::Text(t) => t,
            FlashcardField::Image {
                path: _path,
                alt_text,
            } => alt_text,
        };

        writeln!(file, "{}\\#*#\\{}", front, back)?;
        writeln!(file, "/#")?;
    }

    Ok(())
}

/// Given a path to save the file and a Vec<Flashcard> creates a file compatible with Anki to import the flashcards
pub fn export_flashcards_anki(
    file_path: &str,
    flashcards: &[Flashcard],
) -> Result<(), anywho::Error> {
    let correct_file_path = format!("{file_path}.txt");
    let mut file = File::create(correct_file_path)?;

    writeln!(file, "#separator:tab")?;
    writeln!(file, "#html:false")?;
    writeln!(file)?; //adds a \n before the flashcards

    for flashcard in flashcards {
        let front = match &flashcard.front {
            FlashcardField::Text(t) => t,
            FlashcardField::Image {
                path: _path,
                alt_text,
            } => alt_text,
        };

        let back = match &flashcard.back {
            FlashcardField::Text(t) => t,
            FlashcardField::Image {
                path: _path,
                alt_text,
            } => alt_text,
        };

        writeln!(file, "{}\t{}", front, back)?;
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
        .map(|last| (current - last) as u32)
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
    let effective_interval = interval_days.max(1.0) as i32;
    let new_due_date = current + effective_interval;

    Some((new_memory_state, new_due_date))
}

// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs::File,
    io::{self, BufRead, Write},
    path::Path,
};

use percent_encoding::percent_decode_str;
use rand::prelude::*;
use rand::rng;

use super::models::{flashcard::Flashcard, studyset::StudySet};

/// Selects a random flashcard from the vec, keeping in mind the flashcard status
pub fn select_random_flashcard(flashcards: &Vec<Flashcard>) -> Option<Flashcard> {
    let mut rng = rng();
    let mut weighted_flashcards = Vec::new();

    for flashcard in flashcards {
        let weight = match flashcard.status {
            1 => 4, // High chance (status = 1 = flashcard Bad)
            2 => 3, // Medium chance (status = 2 = flashcard Ok)
            3 => 1, // Low chance (status = 3 = flashcard Good)
            _ => 2, // Default chance for other statuses
        };

        for _ in 0..weight {
            weighted_flashcards.push(flashcard);
        }
    }

    // Select a random flashcard from the weighted list
    weighted_flashcards.choose(&mut rng).copied().cloned()
}

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
                    front: front.to_string(),
                    back: back.to_string(),
                    status: 0,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Given a path to an anki export file parses it to Flashcards in Oboete
pub fn parse_ankifile(file_path: &str) -> Result<Vec<Flashcard>, io::Error> {
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
                front: parts[0].to_string(),
                back: parts[1].to_string(),
                status: 0,
            });
        }
    }

    Ok(flashcards)
}

/// Given a path to save the file and a Vec<Flashcard> creates a file with the flashcards data
pub fn export_flashcards(file_path: &str, flashcards: &Vec<Flashcard>) -> Result<(), io::Error> {
    let mut file = File::create(file_path)?;

    for flashcard in flashcards {
        writeln!(file, "{}\\#*#\\{}", flashcard.front, flashcard.back)?;
        writeln!(file, "/#")?;
    }

    Ok(())
}

/// Given a path to save the file and a Vec<Flashcard> creates a file compatible with Anki to import the flashcards
pub fn export_flashcards_anki(
    file_path: &str,
    flashcards: &Vec<Flashcard>,
) -> Result<(), io::Error> {
    let correct_file_path = format!("{file_path}.txt");
    let mut file = File::create(correct_file_path)?;

    writeln!(file, "#separator:tab")?;
    writeln!(file, "#html:false")?;
    writeln!(file)?; //adds a \n before the flashcards

    for flashcard in flashcards {
        writeln!(file, "{}\t{}", flashcard.front, flashcard.back)?;
    }

    Ok(())
}

pub fn export_flashcards_json(file_path: &str, studysets: &Vec<StudySet>) -> Result<(), io::Error> {
    let correct_file_path = format!("{file_path}.json");
    let path = Path::new(&correct_file_path);
    let mut file = File::create(path)?;

    let json_data = serde_json::to_string_pretty(&studysets)
        .map_err(|e| io::Error::other(format!("Serialization error: {e}")))?;

    file.write_all(json_data.as_bytes())?;

    Ok(())
}

pub fn import_flashcards_json(file_path: &str) -> Result<Vec<StudySet>, io::Error> {
    let mut file = File::open(file_path)?;

    let mut json_data = String::new();
    io::Read::read_to_string(&mut file, &mut json_data)?;

    let studysets: Vec<StudySet> = serde_json::from_str(&json_data)
        .map_err(|e| io::Error::other(format!("Deserialization error: {e}")))?;

    Ok(studysets)
}

use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::models::Flashcard;
use percent_encoding::percent_decode_str;
use rand::prelude::*;
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub struct OboeteError {
    //TODO: Improve Error Handling implies removing this allow(dead_code)
    #[allow(dead_code)]
    pub message: String,
}

impl From<sqlx::Error> for OboeteError {
    fn from(err: sqlx::Error) -> Self {
        OboeteError {
            message: err.to_string(),
        }
    }
}

pub fn select_random_flashcard(flashcards: &Vec<Flashcard>) -> Option<Flashcard> {
    let mut rng = thread_rng();
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

pub fn parse_import_content(
    line_delimiter: &String,
    term_delimiter: &String,
    content: &String,
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

pub fn parse_ankifile(file_path: &str) -> Result<Vec<Flashcard>, io::Error> {
    let decoded_path = percent_decode_str(file_path)
        .decode_utf8_lossy()
        .to_string();
    let path = Path::new(&decoded_path);
    let file = File::open(&path)?;
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

pub fn export_flashcards(file_path: &str, flashcards: &Vec<Flashcard>) -> Result<(), io::Error> {
    let mut file = File::create(file_path)?;

    for flashcard in flashcards {
        writeln!(file, "{}\\#*#\\{}", flashcard.front, flashcard.back)?;
        writeln!(file, "/#")?;
    }

    Ok(())
}

pub fn export_flashcards_anki(
    file_path: &str,
    flashcards: &Vec<Flashcard>,
) -> Result<(), io::Error> {
    let correct_file_path = format!("{}.txt", file_path);
    let mut file = File::create(correct_file_path)?;

    writeln!(file, "#separator:tab")?;
    writeln!(file, "#html:false")?;

    for flashcard in flashcards {
        writeln!(file, "{}\t{}", flashcard.front, flashcard.back)?;
    }

    Ok(())
}

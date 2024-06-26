use crate::models::Flashcard;
use rand::prelude::*;
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub struct OboeteError {
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

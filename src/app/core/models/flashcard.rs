// SPDX-License-Identifier: GPL-3.0

use crate::{app::core::utils::current_day, fl};
use cosmic::iced::Color;
use futures::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Flashcard {
    pub id: Option<i32>,
    pub front: FlashcardField,
    pub back: FlashcardField,
    pub status: FlashcardStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fsrs_state: Option<SerializableMemoryState>,
    pub due_date: Option<i32>,      // Days since epoch
    pub last_reviewed: Option<i32>, // Days since epoch
}

impl PartialEq for Flashcard {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Flashcard {}

/// The different Status a [`Flashcard`] can have
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FlashcardStatus {
    #[default]
    None,
    Bad,
    Ok,
    Great,
    Easy,
}

impl std::fmt::Display for FlashcardStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            FlashcardStatus::None => write!(f, "{}", fl!("none-status")),
            FlashcardStatus::Bad => write!(f, "{}", fl!("bad-status")),
            FlashcardStatus::Ok => write!(f, "{}", fl!("ok-status")),
            FlashcardStatus::Great => write!(f, "{}", fl!("good-status")),
            FlashcardStatus::Easy => write!(f, "{}", fl!("easy-status")),
        }
    }
}

impl FlashcardStatus {
    /// Convert the [`FlashcardStatus`] to it's appropiate id
    pub fn to_id(self) -> i32 {
        match self {
            FlashcardStatus::None => 1,
            FlashcardStatus::Bad => 2,
            FlashcardStatus::Ok => 3,
            FlashcardStatus::Great => 4,
            FlashcardStatus::Easy => 5,
        }
    }

    /// Convert into a [`FlashcardStatus`] the given id
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::None),
            2 => Some(Self::Bad),
            3 => Some(Self::Ok),
            4 => Some(Self::Great),
            5 => Some(Self::Easy),
            _ => None,
        }
    }

    /// Appropiate background color for the [`FlashcardStatus`]
    pub fn get_color(&self) -> Color {
        match &self {
            FlashcardStatus::None => Color::default(),
            FlashcardStatus::Bad => Color {
                r: 191.0 / 255.0,
                g: 57.0 / 255.0,
                b: 57.0 / 255.0,
                a: 0.75,
            },
            FlashcardStatus::Ok => Color {
                r: 245.0 / 255.0,
                g: 188.0 / 255.0,
                b: 66.0 / 255.0,
                a: 0.75,
            },
            FlashcardStatus::Great => Color {
                r: 21.0 / 255.0,
                g: 191.0 / 255.0,
                b: 89.0 / 255.0,
                a: 0.75,
            },
            FlashcardStatus::Easy => Color {
                r: 66.0 / 255.0,
                g: 133.0 / 255.0,
                b: 244.0 / 255.0,
                a: 0.75,
            },
        }
    }

    /// Appropiate border color for the [`FlashcardStatus`]
    pub fn get_border_color(&self) -> Color {
        match &self {
            FlashcardStatus::None => Color::default(),
            FlashcardStatus::Bad => Color {
                r: 191.0 / 255.0,
                g: 57.0 / 255.0,
                b: 57.0 / 255.0,
                a: 0.75,
            },
            FlashcardStatus::Ok => Color {
                r: 250.0 / 255.0,
                g: 146.0 / 255.0,
                b: 12.0 / 255.0,
                a: 1.0,
            },
            FlashcardStatus::Great => Color {
                r: 10.0 / 255.0,
                g: 209.0 / 255.0,
                b: 90.0 / 255.0,
                a: 1.0,
            },
            FlashcardStatus::Easy => Color {
                r: 30.0 / 255.0,
                g: 90.0 / 255.0,
                b: 180.0 / 255.0,
                a: 1.0,
            },
        }
    }
}

/// Represents the different field types the flashcard can have in either it's front or it's back
/// get's serialized into ron on the database
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FlashcardField {
    Text(String),
    Image { path: String, alt_text: String },
}

impl Default for FlashcardField {
    fn default() -> Self {
        Self::Text(String::from(""))
    }
}

impl std::fmt::Display for FlashcardField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            FlashcardField::Text(_) => write!(f, "Text"),
            FlashcardField::Image {
                path: _path,
                alt_text: _alt_text,
            } => write!(f, "Image"),
        }
    }
}

impl FlashcardField {
    pub const ALL: &'static [Self] = &[
        Self::Text(String::new()),
        Self::Image {
            path: String::new(),
            alt_text: String::new(),
        },
    ];

    /// Get the serialized ron of the [`FlashcardField`]
    // TODO: Maybe saving RON on the DB is not the best practice, because we will have to be carefull If we ever want to
    // modify the FlashcardField Enum
    pub fn to_ron(&self) -> Result<String, anywho::Error> {
        Ok(ron::to_string(&self)?)
    }

    /// Get the [`FlashcardField`] from a ron string
    pub fn from_ron(ron: String) -> Result<Self, anywho::Error> {
        Ok(ron::from_str(&ron)?)
    }

    /// Returns true if [`FlashcardField`] is ready for database submission
    pub fn is_valid(&self) -> bool {
        match self {
            FlashcardField::Text(t) => !t.is_empty(),
            FlashcardField::Image { path, alt_text } => !path.is_empty() && !alt_text.is_empty(),
        }
    }
}

/// Wrapper of the library MemoryState that implements [`Serialize`] and [`Deserialize`] to get saved as ron in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableMemoryState {
    pub stability: f32,
    pub difficulty: f32,
}

impl From<fsrs::MemoryState> for SerializableMemoryState {
    fn from(state: fsrs::MemoryState) -> Self {
        Self {
            stability: state.stability,
            difficulty: state.difficulty,
        }
    }
}

impl From<SerializableMemoryState> for fsrs::MemoryState {
    fn from(state: SerializableMemoryState) -> Self {
        fsrs::MemoryState {
            stability: state.stability,
            difficulty: state.difficulty,
        }
    }
}

impl Flashcard {
    /// Returns true if the flashcard is ready for db submission
    pub fn is_valid(&self) -> bool {
        self.front.is_valid() && self.back.is_valid()
    }

    // Check if card is due for review
    pub fn is_due(&self) -> bool {
        match self.due_date {
            Some(due) => due <= current_day(),
            // None => true, // New cards are always "due"
            // Only never-reviewed cards (no FSRS state) are due
            None => self.fsrs_state.is_none(),
        }
    }

    /// Get all flashcards of the given [`Folder`] from the database, also returns the desired retention rate for the folder of this flashcards
    pub async fn get_all(
        pool: Arc<Pool<Sqlite>>,
        folder_id: i32,
    ) -> Result<Vec<Flashcard>, anywho::Error> {
        let mut rows = sqlx::query(
            "SELECT id, front, back, status, fsrs_state, due_date, last_reviewed 
             FROM flashcards 
             WHERE folder_id = $1 
             ORDER BY id ASC",
        )
        .bind(folder_id)
        .fetch(pool.as_ref());

        let mut result = Vec::<Flashcard>::new();

        while let Some(row) = rows.try_next().await? {
            let id: i32 = row.try_get("id")?;
            let front: String = row.try_get("front")?;
            let back: String = row.try_get("back")?;
            let status: i32 = row.try_get("status")?;

            let fsrs_state: Option<String> = row.try_get("fsrs_state").ok();
            let due_date: Option<i32> = row.try_get("due_date").ok();
            let last_reviewed: Option<i32> = row.try_get("last_reviewed").ok();

            let flashcard = Flashcard {
                id: Some(id),
                front: FlashcardField::from_ron(front)?,
                back: FlashcardField::from_ron(back)?,
                status: FlashcardStatus::from_id(status).unwrap_or_default(),
                fsrs_state: fsrs_state.and_then(|s| ron::from_str(&s).ok()),
                due_date,
                last_reviewed,
            };

            result.push(flashcard);
        }

        Ok(result)
    }

    /// Get all flashcards of the given [`Folder`] from the database, also returns the desired retention rate for the folder of this flashcards
    pub async fn get_all_with_retention_rate(
        pool: Arc<Pool<Sqlite>>,
        folder_id: i32,
    ) -> Result<(Vec<Flashcard>, f32), anywho::Error> {
        let desired_retention =
            sqlx::query_scalar("SELECT desired_retention FROM folders WHERE id = ?")
                .bind(folder_id)
                .fetch_one(pool.as_ref())
                .await?;

        let mut rows = sqlx::query(
            "SELECT id, front, back, status, fsrs_state, due_date, last_reviewed 
             FROM flashcards 
             WHERE folder_id = $1 
             ORDER BY id ASC",
        )
        .bind(folder_id)
        .fetch(pool.as_ref());

        let mut result = Vec::<Flashcard>::new();

        while let Some(row) = rows.try_next().await? {
            let id: i32 = row.try_get("id")?;
            let front: String = row.try_get("front")?;
            let back: String = row.try_get("back")?;
            let status: i32 = row.try_get("status")?;

            let fsrs_state: Option<String> = row.try_get("fsrs_state").ok();
            let due_date: Option<i32> = row.try_get("due_date").ok();
            let last_reviewed: Option<i32> = row.try_get("last_reviewed").ok();

            let flashcard = Flashcard {
                id: Some(id),
                front: FlashcardField::from_ron(front)?,
                back: FlashcardField::from_ron(back)?,
                status: FlashcardStatus::from_id(status).unwrap_or_default(),
                fsrs_state: fsrs_state.and_then(|s| ron::from_str(&s).ok()),
                due_date,
                last_reviewed,
            };

            result.push(flashcard);
        }

        Ok((result, desired_retention))
    }

    /// Add a [`Flashcard`] to the database
    pub async fn add(
        pool: Arc<Pool<Sqlite>>,
        flashcard: Flashcard,
        folder_id: i32,
    ) -> Result<(), anywho::Error> {
        sqlx::query("INSERT INTO flashcards (front, back, status, fsrs_state, due_date, last_reviewed, folder_id) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(&flashcard.front.to_ron()?)
            .bind(&flashcard.back.to_ron()?)
            .bind(flashcard.status.to_id())
            .bind(
                flashcard
                    .fsrs_state
                    .as_ref()
                    .and_then(|s| ron::to_string(s).ok()),
            )
            .bind(flashcard.due_date)
            .bind(flashcard.last_reviewed)
            .bind(folder_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    /// Edit a [`Flashcard`] on the database
    pub async fn edit(pool: Arc<Pool<Sqlite>>, flashcard: Flashcard) -> Result<(), anywho::Error> {
        let front = &flashcard.front.to_ron()?;
        let back = &flashcard.back.to_ron()?;

        sqlx::query("UPDATE flashcards SET front = $1, back = $2 WHERE id = $3")
            .bind(front)
            .bind(back)
            .bind(flashcard.id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    /// Delete a [`Flashcard`] on the database
    pub async fn delete(pool: Arc<Pool<Sqlite>>, flashcard_id: i32) -> Result<(), anywho::Error> {
        sqlx::query("DELETE FROM flashcards WHERE id = ?")
            .bind(flashcard_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    /// Updates the status and FSRS data of a [`Flashcard`] on the database
    pub async fn update_status(
        pool: Arc<Pool<Sqlite>>,
        status: FlashcardStatus,
        flashcard_id: i32,
        fsrs_state: SerializableMemoryState,
        due_date: i32,
    ) -> Result<(), anywho::Error> {
        sqlx::query(
            "UPDATE flashcards 
             SET status = $1, fsrs_state = $2, due_date = $3, last_reviewed = $4 
             WHERE id = $5",
        )
        .bind(status.to_id())
        .bind(ron::to_string(&fsrs_state)?)
        .bind(due_date)
        .bind(current_day())
        .bind(flashcard_id)
        .execute(pool.as_ref())
        .await?;

        Ok(())
    }

    /// Resets the status of a [`Flashcard`] on the database, also deletes the [`Flashcard`] fsrs data
    pub async fn reset_single_status(
        pool: Arc<Pool<Sqlite>>,
        flashcard_id: i32,
    ) -> Result<(), anywho::Error> {
        sqlx::query("UPDATE flashcards SET status = $1, fsrs_state = NULL, due_date = NULL, last_reviewed = NULL WHERE id = $2")
            .bind(FlashcardStatus::None.to_id())
            .bind(flashcard_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    /// Resets the status of all the [`Flashcard`] of a given folder on the database, also deletes the [`Flashcard`] fsrs data
    pub async fn reset_all_status(
        pool: Arc<Pool<Sqlite>>,
        folder_id: i32,
    ) -> Result<(), anywho::Error> {
        sqlx::query("UPDATE flashcards SET status = $1, fsrs_state = NULL, due_date = NULL, last_reviewed = NULL WHERE folder_id = $2")
            .bind(FlashcardStatus::None.to_id())
            .bind(folder_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    /// Add more than one [`Flashcard`] to the database
    pub async fn add_bulk(
        pool: Arc<Pool<Sqlite>>,
        flashcards: Vec<Flashcard>,
        folder_id: i32,
    ) -> Result<(), anywho::Error> {
        for flashcard in flashcards {
            Self::add(pool.clone(), flashcard, folder_id).await?
        }
        Ok(())
    }
}

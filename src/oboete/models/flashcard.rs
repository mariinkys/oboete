// SPDX-License-Identifier: GPL-3.0-only

use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flashcard {
    pub id: Option<i32>,
    pub front: String,
    pub back: String,
    pub status: i32,
}

impl Flashcard {
    pub fn new_error_variant() -> Self {
        Flashcard {
            id: None,
            front: String::from("Error"),
            back: String::from("Error"),
            status: 0,
        }
    }

    pub async fn get_all(
        pool: Arc<Pool<Sqlite>>,
        folder_id: i32,
    ) -> Result<Vec<Flashcard>, sqlx::Error> {
        let mut rows = sqlx::query(
            "SELECT id, front, back, status FROM flashcards WHERE folder_id = $1 ORDER BY id ASC",
        )
        .bind(folder_id)
        .fetch(pool.as_ref());

        let mut result = Vec::<Flashcard>::new();

        while let Some(row) = rows.try_next().await? {
            let id: i32 = row.try_get("id")?;
            let front: String = row.try_get("front")?;
            let back: String = row.try_get("back")?;
            let status: i32 = row.try_get("status")?;

            let flashcard = Flashcard {
                id: Some(id),
                front,
                back,
                status,
            };

            result.push(flashcard);
        }

        Ok(result)
    }

    pub async fn edit(pool: Arc<Pool<Sqlite>>, flashcard: Flashcard) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE flashcards SET front = $1, back = $2 WHERE id = $3")
            .bind(&flashcard.front)
            .bind(&flashcard.back)
            .bind(flashcard.id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn add(
        pool: Arc<Pool<Sqlite>>,
        flashcard: Flashcard,
        folder_id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO flashcards (front, back, status, folder_id) VALUES (?, ?, ?, ?)")
            .bind(&flashcard.front)
            .bind(&flashcard.back)
            .bind(flashcard.status)
            .bind(folder_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn delete(pool: Arc<Pool<Sqlite>>, flashcard_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM flashcards WHERE id = ?")
            .bind(flashcard_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    // Updates the status of the given flashcard and returns all the flashcards for the given folder
    pub async fn update_status(
        pool: Arc<Pool<Sqlite>>,
        flashcard: Flashcard,
        folder_id: i32,
    ) -> Result<Vec<Flashcard>, sqlx::Error> {
        sqlx::query("UPDATE flashcards SET status = $1 WHERE id = $2")
            .bind(flashcard.status)
            .bind(flashcard.id)
            .execute(pool.as_ref())
            .await?;

        let mut rows = sqlx::query(
            "SELECT id, front, back, status FROM flashcards WHERE folder_id = $1 ORDER BY id ASC",
        )
        .bind(folder_id)
        .fetch(pool.as_ref());

        let mut result = Vec::<Flashcard>::new();

        while let Some(row) = rows.try_next().await? {
            let id: i32 = row.try_get("id")?;
            let front: String = row.try_get("front")?;
            let back: String = row.try_get("back")?;
            let status: i32 = row.try_get("status")?;

            let flashcard = Flashcard {
                id: Some(id),
                front,
                back,
                status,
            };

            result.push(flashcard);
        }

        Ok(result)
    }

    pub async fn add_bulk(
        pool: Arc<Pool<Sqlite>>,
        flashcards: Vec<Flashcard>,
        folder_id: i32,
    ) -> Result<(), sqlx::Error> {
        for flashcard in flashcards {
            #[allow(clippy::question_mark)]
            if let Err(err) = Self::add(pool.clone(), flashcard, folder_id).await {
                return Err(err);
            }
        }
        Ok(())
    }
}

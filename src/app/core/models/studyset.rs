// SPDX-License-Identifier: GPL-3.0

use futures::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StudySet {
    pub id: Option<i32>,
    pub name: String,
}

impl StudySet {
    /// Get all [`StudySet`] from the database
    pub async fn get_all(pool: Arc<Pool<Sqlite>>) -> Result<Vec<StudySet>, anywho::Error> {
        let mut rows =
            sqlx::query("SELECT id, name FROM studysets ORDER BY id ASC").fetch(pool.as_ref());

        let mut result = Vec::<StudySet>::new();

        while let Some(row) = rows.try_next().await? {
            let id: i32 = row.try_get("id")?;
            let name: String = row.try_get("name")?;

            let studyset = StudySet { id: Some(id), name };

            result.push(studyset);
        }

        Ok(result)
    }

    /// Add a [`StudySet`] to the database
    pub async fn add(pool: Arc<Pool<Sqlite>>, name: String) -> Result<(), anywho::Error> {
        sqlx::query("INSERT INTO studysets (name) VALUES (?)")
            .bind(&name)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    /// Edit a [`StudySet`] on the database
    pub async fn edit(pool: Arc<Pool<Sqlite>>, studyset: StudySet) -> Result<(), anywho::Error> {
        sqlx::query("UPDATE studysets SET name = $1 WHERE id = $2")
            .bind(&studyset.name)
            .bind(studyset.id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    /// Delete a [`StudySet`] from the database (and it's folders and the folders flashcards)
    pub async fn delete(pool: Arc<Pool<Sqlite>>, studyset_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM studysets WHERE id = ?")
            .bind(studyset_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}

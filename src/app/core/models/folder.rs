// SPDX-License-Identifier: GPL-3.0

use futures::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: Option<i32>,
    pub name: String,
    pub desired_retention: f32,
}

impl Default for Folder {
    fn default() -> Self {
        Self {
            id: Default::default(),
            desired_retention: 0.90,
            name: Default::default(),
        }
    }
}

impl Folder {
    /// Get all folders of the given [`StudySet`] from the database
    pub async fn get_all(
        pool: Arc<Pool<Sqlite>>,
        set_id: i32,
    ) -> Result<Vec<Folder>, anywho::Error> {
        let mut rows =
            sqlx::query("SELECT id, name, desired_retention FROM folders WHERE studyset_id = $1 ORDER BY id ASC")
                .bind(set_id)
                .fetch(pool.as_ref());

        let mut result = Vec::<Folder>::new();

        while let Some(row) = rows.try_next().await? {
            let id: i32 = row.try_get("id")?;
            let name: String = row.try_get("name")?;
            let desired_retention: f32 = row.try_get("desired_retention")?;

            let folder = Folder {
                id: Some(id),
                name,
                desired_retention,
            };

            result.push(folder);
        }

        Ok(result)
    }

    /// Add a [`Folder`] to the database
    pub async fn add(
        pool: Arc<Pool<Sqlite>>,
        studyset_id: i32,
        folder: Folder,
    ) -> Result<(), anywho::Error> {
        sqlx::query("INSERT INTO folders (name, studyset_id, desired_retention) VALUES (?, ?, ?)")
            .bind(&folder.name)
            .bind(studyset_id)
            .bind(folder.desired_retention)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    /// Edit a [`Folder`] on the database
    pub async fn edit(pool: Arc<Pool<Sqlite>>, folder: Folder) -> Result<(), anywho::Error> {
        sqlx::query("UPDATE folders SET name = $1, desired_retention = $2 WHERE id = $3")
            .bind(&folder.name)
            .bind(folder.desired_retention)
            .bind(folder.id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    /// Delete a [`Folder`] from the database (and it's flashcards)
    pub async fn delete(pool: Arc<Pool<Sqlite>>, folder_id: i32) -> Result<(), anywho::Error> {
        sqlx::query("DELETE FROM folders WHERE id = ?")
            .bind(folder_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}

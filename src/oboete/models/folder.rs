// SPDX-License-Identifier: GPL-3.0-only

use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: Option<i32>,
    pub name: String,
    #[serde(default)]
    pub rtl_fix: bool,
    pub flashcards: Vec<super::flashcard::Flashcard>,
}

impl Folder {
    pub fn new(name: String) -> Folder {
        Folder {
            id: None,
            name,
            rtl_fix: false,
            flashcards: Vec::new(),
        }
    }

    pub async fn get_all(pool: Arc<Pool<Sqlite>>, set_id: i32) -> Result<Vec<Folder>, sqlx::Error> {
        let mut rows = sqlx::query(
            "SELECT id, name, rtl_fix FROM folders WHERE studyset_id = $1 ORDER BY id ASC",
        )
        .bind(set_id)
        .fetch(pool.as_ref());

        let mut result = Vec::<Folder>::new();

        while let Some(row) = rows.try_next().await? {
            let id: i32 = row.try_get("id")?;
            let name: String = row.try_get("name")?;
            let rtl_fix: bool = row.try_get("rtl_fix")?;

            let folder = Folder {
                id: Some(id),
                name,
                rtl_fix,
                flashcards: Vec::new(),
            };

            result.push(folder);
        }

        Ok(result)
    }

    pub async fn edit(pool: Arc<Pool<Sqlite>>, folder: Folder) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE folders SET name = $1, rtl_fix = $2 WHERE id = $3")
            .bind(&folder.name)
            .bind(folder.rtl_fix)
            .bind(folder.id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn add(
        pool: Arc<Pool<Sqlite>>,
        folder: Folder,
        studyset_id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO folders (name, studyset_id) VALUES (?, ?)")
            .bind(&folder.name)
            .bind(studyset_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn delete(pool: Arc<Pool<Sqlite>>, folder_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM folders WHERE id = ?")
            .bind(folder_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}

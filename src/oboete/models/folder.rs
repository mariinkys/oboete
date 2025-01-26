// SPDX-License-Identifier: GPL-3.0-only

use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: Option<i32>,
    pub name: String,
}

impl Folder {
    pub fn new(name: String) -> Folder {
        Folder { id: None, name }
    }

    pub async fn get_all(pool: Arc<Pool<Sqlite>>, set_id: i32) -> Result<Vec<Folder>, sqlx::Error> {
        let mut rows =
            sqlx::query("SELECT id, name FROM folders WHERE studyset_id = $1 ORDER BY id ASC")
                .bind(set_id)
                .fetch(pool.as_ref());

        let mut result = Vec::<Folder>::new();

        while let Some(row) = rows.try_next().await? {
            let id: i32 = row.try_get("id")?;
            let name: String = row.try_get("name")?;

            let folder = Folder { id: Some(id), name };

            result.push(folder);
        }

        Ok(result)
    }

    pub async fn edit(pool: Arc<Pool<Sqlite>>, folder: Folder) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE folders SET name = $1 WHERE id = $2")
            .bind(&folder.name)
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

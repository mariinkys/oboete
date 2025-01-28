// SPDX-License-Identifier: GPL-3.0-only

use futures::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudySet {
    pub id: Option<i32>,
    pub name: String,
    pub folders: Vec<super::folder::Folder>,
}

impl StudySet {
    pub fn new(name: String) -> StudySet {
        StudySet {
            id: None,
            name,
            folders: Vec::new(),
        }
    }

    pub async fn get_all(pool: Arc<Pool<Sqlite>>) -> Result<Vec<StudySet>, sqlx::Error> {
        let mut rows =
            sqlx::query("SELECT id, name FROM studysets ORDER BY id ASC").fetch(pool.as_ref());

        let mut result = Vec::<StudySet>::new();

        while let Some(row) = rows.try_next().await? {
            let id: i32 = row.try_get("id")?;
            let name: String = row.try_get("name")?;

            let studyset = StudySet {
                id: Some(id),
                name,
                folders: Vec::new(),
            };

            result.push(studyset);
        }

        Ok(result)
    }

    pub async fn add(pool: Arc<Pool<Sqlite>>, studyset: StudySet) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO studysets (name) VALUES (?)")
            .bind(&studyset.name)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn edit(pool: Arc<Pool<Sqlite>>, studyset: StudySet) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE studysets SET name = $1 WHERE id = $2")
            .bind(&studyset.name)
            .bind(studyset.id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn delete(pool: Arc<Pool<Sqlite>>, studyset_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM studysets WHERE id = ?")
            .bind(studyset_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn get_all_data(pool: Arc<Pool<Sqlite>>) -> Result<Vec<StudySet>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                s.id AS studyset_id, s.name AS studyset_name,
                f.id AS folder_id, f.name AS folder_name,
                fc.id AS flashcard_id, fc.front, fc.back, fc.status
            FROM studysets s
            LEFT JOIN folders f ON s.id = f.studyset_id
            LEFT JOIN flashcards fc ON f.id = fc.folder_id
            ORDER BY s.id, f.id, fc.id
            "#,
        )
        .fetch_all(pool.as_ref())
        .await?;

        let mut study_sets: Vec<StudySet> = Vec::new();
        let mut current_studyset: Option<StudySet> = None;

        for row in rows {
            let studyset_id: i32 = row.get("studyset_id");
            let studyset_name: String = row.get("studyset_name");
            let folder_id: Option<i32> = row.get("folder_id");
            let folder_name: Option<String> = row.get("folder_name");
            let flashcard_id: Option<i32> = row.get("flashcard_id");

            if current_studyset.is_none()
                || current_studyset.as_ref().unwrap().id != Some(studyset_id)
            {
                if let Some(studyset) = current_studyset {
                    study_sets.push(studyset);
                }
                current_studyset = Some(StudySet {
                    id: Some(studyset_id),
                    name: studyset_name,
                    folders: Vec::new(),
                });
            }

            if let Some(folder_id) = folder_id {
                let current_studyset = current_studyset.as_mut().unwrap();
                let folder = current_studyset
                    .folders
                    .iter_mut()
                    .find(|f| f.id == Some(folder_id));

                if let Some(folder) = folder {
                    if let Some(flashcard_id) = flashcard_id {
                        let flashcard = super::flashcard::Flashcard {
                            id: Some(flashcard_id),
                            front: row.get("front"),
                            back: row.get("back"),
                            status: row.get("status"),
                        };
                        folder.flashcards.push(flashcard);
                    }
                } else {
                    let mut new_folder = super::folder::Folder {
                        id: Some(folder_id),
                        name: folder_name.unwrap(),
                        flashcards: Vec::new(),
                    };

                    if let Some(flashcard_id) = flashcard_id {
                        let flashcard = super::flashcard::Flashcard {
                            id: Some(flashcard_id),
                            front: row.get("front"),
                            back: row.get("back"),
                            status: row.get("status"),
                        };
                        new_folder.flashcards.push(flashcard);
                    }

                    current_studyset.folders.push(new_folder);
                }
            }
        }

        if let Some(studyset) = current_studyset {
            study_sets.push(studyset);
        }

        Ok(study_sets)
    }

    pub async fn import(
        pool: Arc<Pool<Sqlite>>,
        studysets: Vec<StudySet>,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = pool.begin().await?;

        for studyset in studysets {
            let studyset_id = sqlx::query("INSERT INTO studysets (name) VALUES (?) RETURNING id")
                .bind(&studyset.name)
                .fetch_one(&mut *transaction)
                .await?
                .get::<i32, _>("id");

            for folder in studyset.folders {
                let folder_id = sqlx::query(
                    "INSERT INTO folders (name, studyset_id) VALUES (?, ?) RETURNING id",
                )
                .bind(&folder.name)
                .bind(studyset_id)
                .fetch_one(&mut *transaction)
                .await?
                .get::<i32, _>("id");

                for flashcard in folder.flashcards {
                    sqlx::query(
                        "INSERT INTO flashcards (front, back, status, folder_id) VALUES (?, ?, ?, ?)",
                    )
                    .bind(&flashcard.front)
                    .bind(&flashcard.back)
                    .bind(flashcard.status)
                    .bind(folder_id)
                    .execute(&mut *transaction)
                    .await?;
                }
            }
        }

        transaction.commit().await?;

        Ok(())
    }
}

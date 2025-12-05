use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;

use crate::app::core::models::{
    flashcard::{Flashcard, FlashcardField, FlashcardStatus},
    folder::Folder,
    studyset::StudySet,
};

#[derive(Serialize, Deserialize)]
struct BackupData {
    study_sets: Vec<BackupStudySet>,
}

#[derive(Serialize, Deserialize)]
struct BackupStudySet {
    study_set: StudySet,
    folders: Vec<BackupFolder>,
}

#[derive(Serialize, Deserialize)]
struct BackupFolder {
    folder: Folder,
    flashcards: Vec<Flashcard>,
}

pub async fn backup_oboete(
    pool: Arc<Pool<Sqlite>>,
    file_path: String,
) -> Result<(), anywho::Error> {
    let mut rows = sqlx::query(
        r#"
            SELECT
                s.id AS studyset_id, s.name AS studyset_name,
                f.id AS folder_id, f.name AS folder_name,
                fc.id AS flashcard_id, fc.front, fc.back, fc.status,
                fc.fsrs_state, fc.due_date, fc.last_reviewed
            FROM studysets s
            LEFT JOIN folders f ON s.id = f.studyset_id
            LEFT JOIN flashcards fc ON f.id = fc.folder_id
            ORDER BY s.id, f.id, fc.id
            "#,
    )
    .fetch(pool.as_ref());

    let mut study_sets: Vec<BackupStudySet> = Vec::new();
    let mut current_studyset: Option<BackupStudySet> = None;

    while let Some(row) = rows.try_next().await? {
        let studyset_id: i32 = row.try_get("studyset_id")?;
        let studyset_name: String = row.try_get("studyset_name")?;

        let folder_id: Option<i32> = row.try_get("folder_id").ok();
        let folder_name: Option<String> = row.try_get("folder_name").ok();
        let flashcard_id: Option<i32> = row.try_get("flashcard_id").ok();

        if current_studyset.is_none()
            || current_studyset.as_ref().unwrap().study_set.id != Some(studyset_id)
        {
            if let Some(studyset) = current_studyset {
                study_sets.push(studyset);
            }
            current_studyset = Some(BackupStudySet {
                study_set: StudySet {
                    id: Some(studyset_id),
                    name: studyset_name.clone(),
                },
                folders: Vec::new(),
            });
        }

        if let (Some(folder_id), Some(folder_name)) = (folder_id, folder_name) {
            // Ignore "ghost" folders with ID 0.
            // This handles the case where LEFT JOIN returns 0 instead of NULL,
            // preventing the creation of phantom folders with empty names.
            if folder_id == 0 {
                continue;
            }

            let current_studyset = current_studyset.as_mut().unwrap();

            let folder_index = if let Some(idx) = current_studyset
                .folders
                .iter()
                .position(|f| f.folder.id == Some(folder_id))
            {
                idx
            } else {
                let new_folder = BackupFolder {
                    folder: Folder {
                        id: Some(folder_id),
                        name: folder_name,
                    },
                    flashcards: Vec::new(),
                };
                current_studyset.folders.push(new_folder);
                current_studyset.folders.len() - 1
            };

            if let Some(flashcard_id) = flashcard_id {
                // Also filter out ghost flashcards if any
                if flashcard_id > 0 {
                    let front: String = row.try_get("front")?;
                    let back: String = row.try_get("back")?;
                    let status: i32 = row.try_get("status")?;
                    let fsrs_state: Option<String> = row.try_get("fsrs_state").ok();
                    let due_date: Option<i32> = row.try_get("due_date").ok();
                    let last_reviewed: Option<i32> = row.try_get("last_reviewed").ok();

                    let flashcard = Flashcard {
                        id: Some(flashcard_id),
                        front: FlashcardField::from_ron(front)?,
                        back: FlashcardField::from_ron(back)?,
                        status: FlashcardStatus::from_id(status).unwrap_or_default(),
                        fsrs_state: fsrs_state.and_then(|s| ron::from_str(&s).ok()),
                        due_date,
                        last_reviewed,
                    };

                    current_studyset.folders[folder_index]
                        .flashcards
                        .push(flashcard);
                }
            }
        }
    }

    if let Some(studyset) = current_studyset {
        study_sets.push(studyset);
    }

    let backup_data = BackupData { study_sets };
    let ron_string = ron::ser::to_string_pretty(&backup_data, ron::ser::PrettyConfig::default())
        .map_err(|e| anywho::Error::msg(format!("Failed to serialize to RON: {}", e)))?;

    tokio::fs::write(&file_path, ron_string)
        .await
        .map_err(|e| anywho::Error::msg(format!("Failed to write file: {}", e)))?;

    Ok(())
}

pub async fn import_oboete(
    pool: Arc<Pool<Sqlite>>,
    file_path: String,
) -> Result<(), anywho::Error> {
    let ron_string = tokio::fs::read_to_string(file_path).await?;
    let backup_data: BackupData = ron::from_str(&ron_string)?;

    let mut transaction = pool.begin().await?;

    for backup_studyset in backup_data.study_sets {
        let studyset_id = sqlx::query("INSERT INTO studysets (name) VALUES (?) RETURNING id")
            .bind(&backup_studyset.study_set.name)
            .fetch_one(&mut *transaction)
            .await?
            .try_get::<i32, _>("id")?;

        for backup_folder in backup_studyset.folders {
            let folder_id =
                sqlx::query("INSERT INTO folders (name, studyset_id) VALUES (?, ?) RETURNING id")
                    .bind(&backup_folder.folder.name)
                    .bind(studyset_id)
                    .fetch_one(&mut *transaction)
                    .await?
                    .try_get::<i32, _>("id")?;

            for flashcard in backup_folder.flashcards {
                sqlx::query(
                    r#"
                        INSERT INTO flashcards 
                        (front, back, status, fsrs_state, due_date, last_reviewed, folder_id) 
                        VALUES (?, ?, ?, ?, ?, ?, ?)
                        "#,
                )
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
                .execute(&mut *transaction)
                .await?;
            }
        }
    }

    transaction.commit().await?;

    Ok(())
}

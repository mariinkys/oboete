use std::fs;

use futures::TryStreamExt;
use sqlx::{sqlite::SqlitePool, Pool, Row, Sqlite};

use crate::{
    models::{Flashcard, Folder, StudySet},
    utils::OboeteError,
};

const DB_NAME: &str = "oboete.db";

#[derive(Debug, Clone)]
pub struct OboeteDb {
    db_pool: Pool<Sqlite>,
}

impl OboeteDb {
    pub async fn init(app_id: &str) -> OboeteDb {
        let db_path = dirs::data_dir()
            .unwrap()
            .join(app_id)
            .join("database")
            .join(DB_NAME);
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directories for database file");
        }

        if !db_path.exists() {
            fs::File::create(&db_path).expect("Failed to create the database file");
        }

        let pool = SqlitePool::connect(db_path.into_os_string().to_str().unwrap())
            .await
            .expect("Error creating database");

        let migrations = Self::migrate_database(&pool).await;
        match migrations {
            Ok(_) => println!("Migrations SUCCESSFUL"),
            Err(_) => println!("Error running migrations"),
        }

        OboeteDb { db_pool: pool }
    }

    async fn migrate_database(db_pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS studysets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                studyset_id INTEGER NOT NULL,
                FOREIGN KEY (studyset_id) REFERENCES studysets(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS flashcards (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                front TEXT NOT NULL,
                back TEXT NOT NULL,
                status INTEGER NOT NULL,
                folder_id INTEGER NOT NULL,
                FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(db_pool)
        .await?;

        Ok(())
    }
}

pub async fn get_all_studysets(db: Option<OboeteDb>) -> Result<Vec<StudySet>, OboeteError> {
    let pool = match db {
        Some(db) => db,
        None => {
            return Err(OboeteError {
                message: String::from("Cannot access DB pool"),
            })
        }
    };

    let mut rows = sqlx::query("SELECT * FROM studysets ORDER BY id ASC").fetch(&pool.db_pool);

    let mut result = Vec::<StudySet>::new();

    while let Some(row) = rows.try_next().await? {
        let id = row.try_get("id").unwrap_or(0);
        let name = row.try_get("name").unwrap_or("Error");

        let studyset = StudySet {
            id: Some(id),
            name: String::from(name),
            folders: Vec::<Folder>::new(),
        };

        if let Some(_id) = studyset.id {
            result.push(studyset);
        }
    }

    Ok(result)
}

pub async fn upsert_studyset(
    db: Option<OboeteDb>,
    studyset: StudySet,
) -> Result<StudySet, OboeteError> {
    let pool = match db {
        Some(db) => db,
        None => {
            return Err(OboeteError {
                message: String::from("Cannot access DB pool"),
            })
        }
    };

    let command = if studyset.id.is_some() {
        sqlx::query(
            "UPDATE studysets
                SET
                    name = ?
                WHERE
                    id = ?
            ",
        )
        .bind(studyset.name)
        .bind(studyset.id.unwrap())
        .execute(&pool.db_pool)
        .await
    } else {
        sqlx::query(
            "INSERT INTO studysets (
                name
            )
            VALUES (?)",
        )
        .bind(studyset.name)
        .execute(&pool.db_pool)
        .await
    };

    match command {
        Ok(result) => {
            let id = result.last_insert_rowid();

            let row_result = sqlx::query("SELECT * FROM studysets WHERE id = ?")
                .bind(id)
                .fetch_one(&pool.db_pool)
                .await;

            match row_result {
                Ok(row) => {
                    let set: StudySet = StudySet {
                        id: row.get("id"),
                        name: row.get("name"),
                        folders: Vec::new(),
                    };
                    Ok(set)
                }
                Err(err) => Err(err.into()),
            }
        }
        Err(err) => Err(err.into()),
    }
}

pub async fn get_studyset_folders(
    db: Option<OboeteDb>,
    id: i32,
) -> Result<Vec<Folder>, OboeteError> {
    let pool = match db {
        Some(db) => db,
        None => {
            return Err(OboeteError {
                message: String::from("Cannot access DB pool"),
            })
        }
    };

    let mut rows = sqlx::query("SELECT * FROM folders WHERE studyset_id = ? ORDER BY id ASC")
        .bind(id)
        .fetch(&pool.db_pool);

    let mut result = Vec::<Folder>::new();

    while let Some(row) = rows.try_next().await? {
        let id = row.try_get("id").unwrap_or(0);
        let name = row.try_get("name").unwrap_or("Error");

        let folder = Folder {
            id: Some(id),
            name: String::from(name),
            flashcards: Vec::<Flashcard>::new(),
        };

        if let Some(_id) = folder.id {
            result.push(folder);
        }
    }

    Ok(result)
}

pub async fn upsert_folder(
    db: Option<OboeteDb>,
    folder: Folder,
    studyset_id: i32,
) -> Result<i64, OboeteError> {
    let pool = match db {
        Some(db) => db,
        None => {
            return Err(OboeteError {
                message: String::from("Cannot access DB pool"),
            })
        }
    };

    let command = if folder.id.is_some() {
        sqlx::query(
            "UPDATE folders
                SET
                    name = ?
                WHERE
                    id = ?
            ",
        )
        .bind(folder.name)
        .bind(folder.id.unwrap())
        .execute(&pool.db_pool)
        .await
    } else {
        sqlx::query(
            r#"
            INSERT INTO folders (name, studyset_id)
            VALUES (?, ?)
            "#,
        )
        .bind(folder.name)
        .bind(studyset_id)
        .execute(&pool.db_pool)
        .await
    };

    match command {
        Ok(result) => Ok(result.last_insert_rowid()),
        Err(err) => Err(err.into()),
    }
}

pub async fn get_folder_flashcards(
    db: Option<OboeteDb>,
    id: i32,
) -> Result<Vec<Flashcard>, OboeteError> {
    let pool = match db {
        Some(db) => db,
        None => {
            return Err(OboeteError {
                message: String::from("Cannot access DB pool"),
            })
        }
    };

    let mut rows = sqlx::query("SELECT * FROM flashcards WHERE folder_id = ? ORDER BY id ASC")
        .bind(id)
        .fetch(&pool.db_pool);

    let mut result = Vec::<Flashcard>::new();

    while let Some(row) = rows.try_next().await? {
        let id = row.try_get("id").unwrap_or(0);
        let front = row.try_get("front").unwrap_or("Error");
        let back = row.try_get("back").unwrap_or("Error");
        let status = row.try_get("status").unwrap_or_default();

        let flashcard: Flashcard = Flashcard {
            id: Some(id),
            front: String::from(front),
            back: String::from(back),
            status: status,
        };

        if let Some(_id) = flashcard.id {
            result.push(flashcard);
        }
    }

    Ok(result)
}

pub async fn upsert_flashcard(
    db: Option<OboeteDb>,
    flashcard: Flashcard,
    folder_id: i32,
) -> Result<i64, OboeteError> {
    let pool = match db {
        Some(db) => db,
        None => {
            return Err(OboeteError {
                message: String::from("Cannot access DB pool"),
            })
        }
    };

    let command = if flashcard.id.is_some() {
        sqlx::query(
            "UPDATE flashcards
             SET
                 front = $1,
                 back = $2,
                 status = $3
             WHERE
                 id = $4",
        )
        .bind(flashcard.front)
        .bind(flashcard.back)
        .bind(flashcard.status)
        .bind(flashcard.id.unwrap())
        .execute(&pool.db_pool)
        .await
    } else {
        sqlx::query(
            r#"
            INSERT INTO flashcards (front, back, status, folder_id)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(flashcard.front)
        .bind(flashcard.back)
        .bind(flashcard.status)
        .bind(folder_id)
        .execute(&pool.db_pool)
        .await
    };

    match command {
        Ok(result) => Ok(result.last_insert_rowid()),
        Err(err) => Err(err.into()),
    }
}

pub async fn get_single_flashcard(db: Option<OboeteDb>, id: i32) -> Result<Flashcard, OboeteError> {
    let pool = match db {
        Some(db) => db,
        None => {
            return Err(OboeteError {
                message: String::from("Cannot access DB pool"),
            })
        }
    };

    let row_result = sqlx::query("SELECT * FROM flashcards WHERE id = ?")
        .bind(id)
        .fetch_one(&pool.db_pool)
        .await;

    match row_result {
        Ok(row) => {
            let flashcard: Flashcard = Flashcard {
                id: row.get("id"),
                front: row.get("front"),
                back: row.get("back"),
                status: row.get("status"),
            };
            Ok(flashcard)
        }
        Err(err) => Err(err.into()),
    }
}

pub async fn update_flashcard_status(
    db: Option<OboeteDb>,
    flashcard: Flashcard,
    folder_id: i32,
) -> Result<Vec<Flashcard>, OboeteError> {
    let pool = match db {
        Some(db) => db,
        None => {
            return Err(OboeteError {
                message: String::from("Cannot access DB pool"),
            })
        }
    };

    let _command = sqlx::query(
        "UPDATE flashcards
             SET
                 status = $1
             WHERE
                 id = $2",
    )
    .bind(flashcard.status)
    .bind(flashcard.id.unwrap())
    .execute(&pool.db_pool)
    .await;

    let mut rows = sqlx::query("SELECT * FROM flashcards WHERE folder_id = ? ORDER BY id ASC")
        .bind(folder_id)
        .fetch(&pool.db_pool);

    let mut result = Vec::<Flashcard>::new();

    while let Some(row) = rows.try_next().await? {
        let id = row.try_get("id").unwrap_or(0);
        let front = row.try_get("front").unwrap_or("Error");
        let back = row.try_get("back").unwrap_or("Error");
        let status = row.try_get("status").unwrap_or_default();

        let flashcard: Flashcard = Flashcard {
            id: Some(id),
            front: String::from(front),
            back: String::from(back),
            status: status,
        };

        if let Some(_id) = flashcard.id {
            result.push(flashcard);
        }
    }

    Ok(result)
}

pub async fn delete_studyset(db: Option<OboeteDb>, id: i32) -> Result<bool, OboeteError> {
    let pool = match db {
        Some(db) => db,
        None => {
            return Err(OboeteError {
                message: String::from("Cannot access DB pool"),
            })
        }
    };

    let command = sqlx::query("DELETE FROM studysets WHERE id = ?")
        .bind(id)
        .execute(&pool.db_pool)
        .await;

    match command {
        Ok(_) => Ok(true),
        Err(err) => Err(err.into()),
    }
}

pub async fn get_single_folder(db: Option<OboeteDb>, id: i32) -> Result<Folder, OboeteError> {
    let pool = match db {
        Some(db) => db,
        None => {
            return Err(OboeteError {
                message: String::from("Cannot access DB pool"),
            })
        }
    };

    let row_result = sqlx::query("SELECT * FROM folders WHERE id = ?")
        .bind(id)
        .fetch_one(&pool.db_pool)
        .await;

    match row_result {
        Ok(row) => {
            let folder: Folder = Folder {
                id: row.get("id"),
                name: row.get("name"),
                flashcards: Vec::new(),
            };
            Ok(folder)
        }
        Err(err) => Err(err.into()),
    }
}

pub async fn delete_folder(db: Option<OboeteDb>, id: i32) -> Result<bool, OboeteError> {
    let pool = match db {
        Some(db) => db,
        None => {
            return Err(OboeteError {
                message: String::from("Cannot access DB pool"),
            })
        }
    };

    let command = sqlx::query("DELETE FROM folders WHERE id = ?")
        .bind(id)
        .execute(&pool.db_pool)
        .await;

    match command {
        Ok(_) => Ok(true),
        Err(err) => Err(err.into()),
    }
}

pub async fn delete_flashcard(db: Option<OboeteDb>, id: i32) -> Result<bool, OboeteError> {
    let pool = match db {
        Some(db) => db,
        None => {
            return Err(OboeteError {
                message: String::from("Cannot access DB pool"),
            })
        }
    };

    let command = sqlx::query("DELETE FROM flashcards WHERE id = ?")
        .bind(id)
        .execute(&pool.db_pool)
        .await;

    match command {
        Ok(_) => Ok(true),
        Err(err) => Err(err.into()),
    }
}

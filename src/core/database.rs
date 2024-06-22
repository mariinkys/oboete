use std::fs;

use futures::TryStreamExt;
use sqlx::{sqlite::SqlitePool, Pool, Row, Sqlite};

use crate::{
    models::{Folder, StudySet},
    utils::OboeteError,
};

const DB_URL: &str = "oboete.db";

#[derive(Debug, Clone)]
pub struct OboeteDb {
    db_pool: Pool<Sqlite>,
}

impl OboeteDb {
    pub async fn init() -> OboeteDb {
        let db_path = std::path::Path::new(DB_URL);
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directories for database file");
        }

        if !db_path.exists() {
            fs::File::create(db_path).expect("Failed to create the database file");
        }

        let pool = SqlitePool::connect(DB_URL)
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
            CREATE TABLE IF NOT EXISTS StudySet (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS Folder (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                studyset_id INTEGER NOT NULL,
                FOREIGN KEY (studyset_id) REFERENCES StudySet(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS Flashcard (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                front TEXT NOT NULL,
                back TEXT NOT NULL,
                status INTEGER NOT NULL,
                folder_id INTEGER NOT NULL,
                FOREIGN KEY (folder_id) REFERENCES Folder(id) ON DELETE CASCADE
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
        let name = row.try_get("client_name").unwrap_or("Error");
        //TODO: Get Folders

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

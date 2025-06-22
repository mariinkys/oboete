// SPDX-License-Identifier: GPL-3.0-only

use std::{fs, sync::Arc};

use sqlx::{Pool, Sqlite, SqlitePool};

const DB_NAME: &str = "oboete.db";

pub async fn init_database(app_id: &str) -> Arc<Pool<Sqlite>> {
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

    let migrations = migrate_database(&pool).await;
    match migrations {
        Ok(_) => println!("Migrations SUCCESSFUL"),
        Err(_) => println!("Error running migrations"),
    }

    Arc::new(pool)
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

    // Add rtl_fix column to folders table if it doesn't exist
    sqlx::query(
        r#"
        ALTER TABLE folders ADD COLUMN rtl_fix BOOLEAN NOT NULL DEFAULT FALSE;
        "#,
    )
    .execute(db_pool)
    .await
    .or_else(|e| {
        // Ignore error if column already exists
        if e.to_string().contains("duplicate column name") {
            Ok(sqlx::sqlite::SqliteQueryResult::default())
        } else {
            Err(e)
        }
    })?;

    Ok(())
}

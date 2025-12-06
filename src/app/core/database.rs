// SPDX-License-Identifier: GPL-3.0

use std::{fs, path::PathBuf, sync::Arc};

use sqlx::{Pool, Sqlite, SqlitePool};

use crate::app::core::models::flashcard::{FlashcardField, FlashcardStatus};

/// Init the application database
pub async fn init_database(app_id: &str) -> Arc<Pool<Sqlite>> {
    let db_path = dirs::data_dir()
        .unwrap()
        .join(app_id)
        .join("database")
        .join("oboete_v2.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create directories for database file");
    }

    if !db_path.exists() {
        fs::File::create(&db_path).expect("Failed to create the database file");
    }

    let pool = SqlitePool::connect(db_path.into_os_string().to_str().unwrap())
        .await
        .expect("Error creating database");

    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => println!("Migrations run successfully"),
        Err(err) => {
            eprintln!("Error occurred running migrations: {err}");
            std::process::exit(1);
        }
    };

    let old_db_path = dirs::data_dir()
        .unwrap()
        .join(app_id)
        .join("database")
        .join("oboete.db");

    if old_db_path.exists() {
        let res = import_old_database_data(&pool, old_db_path).await;
        if let Err(e) = res {
            eprintln!("{}", e);
        }
    }

    Arc::new(pool)
}

/// Imports the content from the old database to the new one and deletes the old database
async fn import_old_database_data(
    pool: &Pool<Sqlite>,
    old_db_path: PathBuf,
) -> Result<(), anywho::Error> {
    use sqlx::Row;

    println!(
        "Old database detected, importing: {}",
        old_db_path.display()
    );

    // Connect to old DB
    let old_conn_str = format!("sqlite://{}", old_db_path.to_string_lossy());
    let old_pool = SqlitePool::connect(&old_conn_str)
        .await
        .expect("Failed to open old database");

    // Import studysets
    let old_studysets = sqlx::query("SELECT id, name FROM studysets")
        .fetch_all(&old_pool)
        .await
        .expect("Failed to read studysets");

    // Map old studyset_id → new studyset_id
    let mut studyset_id_map = std::collections::HashMap::new();

    for row in old_studysets {
        let old_id: i64 = row.get("id");
        let name: String = row.get("name");

        let new_id = sqlx::query("INSERT INTO studysets (name) VALUES (?) RETURNING id")
            .bind(name)
            .fetch_one(pool)
            .await
            .expect("Failed to insert studyset")
            .get::<i64, _>("id");

        studyset_id_map.insert(old_id, new_id);
    }

    // Import folders
    let old_folders = sqlx::query("SELECT id, name, studyset_id FROM folders")
        .fetch_all(&old_pool)
        .await
        .expect("Failed to read folders");

    // Map old folder_id → new folder_id
    let mut folder_id_map = std::collections::HashMap::new();

    for row in old_folders {
        let old_id: i64 = row.get("id");
        let name: String = row.get("name");
        let old_studyset_id: i64 = row.get("studyset_id");

        let new_studyset_id = studyset_id_map[&old_studyset_id];

        let new_id = sqlx::query(
            "INSERT INTO folders (name, desired_retention, studyset_id)
             VALUES (?, ?, ?) RETURNING id",
        )
        .bind(name)
        .bind(0.9f32)
        .bind(new_studyset_id)
        .fetch_one(pool)
        .await
        .expect("Failed to insert folder")
        .get::<i64, _>("id");

        folder_id_map.insert(old_id, new_id);
    }

    // -----------------------------
    // 3. Import flashcards
    // -----------------------------
    let old_flashcards = sqlx::query(
        "SELECT id, front, back, folder_id
         FROM flashcards",
    )
    .fetch_all(&old_pool)
    .await
    .expect("Failed to read flashcards");

    for row in old_flashcards {
        let front: String = row.get("front");
        let back: String = row.get("back");
        let old_folder_id: i64 = row.get("folder_id");

        let new_folder_id = folder_id_map[&old_folder_id];

        // New schema includes fsrs_state, due_date, last_reviewed
        sqlx::query(
            "INSERT INTO flashcards
             (front, back, status, fsrs_state, due_date, last_reviewed, folder_id)
             VALUES (?, ?, ?, NULL, NULL, NULL, ?)",
        )
        .bind(FlashcardField::Text(front).to_ron()?)
        .bind(FlashcardField::Text(back).to_ron()?)
        .bind(FlashcardStatus::None.to_id())
        .bind(new_folder_id)
        .execute(pool)
        .await
        .expect("Failed to insert flashcard");
    }

    fs::remove_file(&old_db_path).expect("Failed to delete old database file");

    println!("Old database imported and deleted.");
    Ok(())
}

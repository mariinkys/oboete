use std::{fs, sync::Arc};

use sqlx::{Pool, Sqlite, SqlitePool};

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

    Arc::new(pool)
}

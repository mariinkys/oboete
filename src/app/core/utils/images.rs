use std::{fs, path::Path};

/// Returns true if the path corresponds to an image already saved in the Oboete directory
pub fn check_path(image_path: &String) -> bool {
    let Some(data_dir) = dirs::data_dir() else {
        return false;
    };

    let output_path = data_dir.join(super::APP_ID).join("images");

    let path = Path::new(image_path);
    path.starts_with(&output_path)
}

pub fn save_image(image_path: &String) -> Result<String, anywho::Error> {
    let output_path = dirs::data_dir()
        .ok_or_else(|| anywho::anywho!("Failed to get data directory"))?
        .join(super::APP_ID)
        .join("images");

    if !output_path.exists() {
        fs::create_dir_all(&output_path)?;
    }

    let extension = Path::new(image_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| anywho::anywho!("Failed to get file extension"))?;

    let new_filename = format!("{}.{}", uuid::Uuid::new_v4(), extension);
    let destination = output_path.join(&new_filename);

    fs::copy(image_path, &destination)?;

    Ok(destination.to_string_lossy().to_string())
}

pub async fn delete_image(image_path: String) -> Result<(), anywho::Error> {
    let path = Path::new(&image_path);

    if tokio::fs::metadata(path).await.is_err() {
        return Err(anywho::anywho!("Image file does not exist: {}", image_path));
    }

    tokio::fs::remove_file(path).await?;

    Ok(())
}

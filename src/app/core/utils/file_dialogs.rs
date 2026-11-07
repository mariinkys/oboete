// SPDX-License-Identifier: GPL-3.0

use std::path::PathBuf;

use cosmic::dialog::ashpd::desktop::file_chooser::SelectedFiles;
use cosmic::dialog::file_chooser::FileFilter;

use crate::fl;

/// The kinds of files Oboete asks the user to pick
#[derive(Debug, Clone, Copy)]
pub enum FileKind {
    Backup,
    Text,
    Image,
}

impl FileKind {
    fn filter(self) -> FileFilter {
        match self {
            FileKind::Backup => FileFilter::new("RON File").glob("*.ron"),
            FileKind::Text => FileFilter::new("TXT File").glob("*.txt"),
            FileKind::Image => FileFilter::new("Image Files")
                .glob("*.png")
                .glob("*.jpeg")
                .glob("*.jpg"),
        }
    }
}

/// Shows the system open file dialog, returns [`None`] if the user cancelled or the dialog failed
pub async fn open_file(title: String, kind: FileKind) -> Option<PathBuf> {
    let accept_label = fl!("open");
    let request = SelectedFiles::open_file()
        .title(title.as_str())
        .accept_label(accept_label.as_str())
        .modal(true)
        .multiple(false)
        .filter(kind.filter())
        .send()
        .await
        .map_err(|e| eprintln!("Failed to open file dialog: {e}"))
        .ok()?;

    first_path(request.response().ok()?)
}

/// Shows the system save file dialog, returns [`None`] if the user cancelled or the dialog failed
pub async fn save_file(title: String, kind: FileKind) -> Option<PathBuf> {
    let accept_label = fl!("save");
    let request = SelectedFiles::save_file()
        .title(title.as_str())
        .accept_label(accept_label.as_str())
        .modal(true)
        .filter(kind.filter())
        .send()
        .await
        .map_err(|e| eprintln!("Failed to open file dialog: {e}"))
        .ok()?;

    first_path(request.response().ok()?)
}

fn first_path(files: SelectedFiles) -> Option<PathBuf> {
    let uri = files.uris().first()?;
    url::Url::parse(&uri.to_string()).ok()?.to_file_path().ok()
}

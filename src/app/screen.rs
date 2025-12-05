// SPDX-License-Identifier: GPL-3.0-only

pub mod flashcards;
pub mod folders;
pub mod study;

pub use flashcards::FlashcardsScreen;
pub use folders::FoldersScreen;
pub use study::StudyScreen;

#[allow(clippy::large_enum_variant)]
pub enum Screen {
    Folders(FoldersScreen),
    Flashcards(FlashcardsScreen),
    Study(StudyScreen),
}

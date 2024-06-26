#[derive(Debug, Clone)]
pub struct Flashcard {
    pub id: Option<i32>,
    pub front: String,
    pub back: String,
    pub status: i32,
}

#[derive(Debug, Clone)]
pub struct StudySet {
    pub id: Option<i32>,
    pub name: String,
    #[allow(dead_code)]
    pub folders: Vec<Folder>,
}

#[derive(Debug, Clone)]
pub struct Folder {
    pub id: Option<i32>,
    pub name: String,
    #[allow(dead_code)]
    pub flashcards: Vec<Flashcard>,
}

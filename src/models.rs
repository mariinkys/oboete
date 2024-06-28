#[derive(Debug, Clone)]
pub struct Flashcard {
    pub id: Option<i32>,
    pub front: String,
    pub back: String,
    pub status: i32,
}

impl Flashcard {
    pub fn new_error_variant() -> Flashcard {
        Flashcard {
            id: None,
            front: String::from("Error"),
            back: String::from("Error"),
            status: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StudySet {
    pub id: Option<i32>,
    pub name: String,
    #[allow(dead_code)]
    pub folders: Vec<Folder>,
}

impl StudySet {
    pub fn new(name: String) -> StudySet {
        StudySet {
            id: None,
            name,
            folders: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Folder {
    pub id: Option<i32>,
    pub name: String,
    #[allow(dead_code)]
    pub flashcards: Vec<Flashcard>,
}

impl Folder {
    pub fn new(name: String) -> Folder {
        Folder {
            id: None,
            name,
            flashcards: Vec::new(),
        }
    }
}

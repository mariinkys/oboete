CREATE TABLE IF NOT EXISTS studysets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    desired_retention REAL NOT NULL,
    studyset_id INTEGER NOT NULL,
    FOREIGN KEY (studyset_id) REFERENCES studysets(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS flashcards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    front TEXT NOT NULL,
    back TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,

    fsrs_state TEXT, -- Serialized MemoryState
    due_date INTEGER, -- Days since epoch
    last_reviewed INTEGER, -- Days since epoch

    folder_id INTEGER NOT NULL,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE
);
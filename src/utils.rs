#[derive(Debug, Clone)]
pub struct OboeteError {
    pub message: String,
}

impl From<sqlx::Error> for OboeteError {
    fn from(err: sqlx::Error) -> Self {
        OboeteError {
            message: err.to_string(),
        }
    }
}

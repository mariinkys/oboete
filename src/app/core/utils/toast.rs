use std::time::Duration;

use cosmic::widget::Toast;

#[derive(Debug, Clone)]
pub struct OboeteToast {
    pub message: String,
}

impl OboeteToast {
    pub fn new<T>(message: T) -> Self
    where
        T: ToString,
    {
        Self {
            message: message.to_string(),
        }
    }
}

impl From<OboeteToast> for Toast<crate::app::Message> {
    fn from(toast: OboeteToast) -> Self {
        Toast::new(toast.message).duration(Duration::from_secs(5))
    }
}

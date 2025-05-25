use crate::animation::easing::Easing;
use std::fmt::Display;

pub struct Information {
    category: String,
    data: String,
    details: Option<String>,
}

impl Information {
    pub fn new(category: String, data: String, details: Option<String>) -> Self {
        Self {
            category,
            data,
            details,
        }
    }
}

impl Display for Information {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.category, self.data)?;
        if let Some(details) = &self.details {
            write!(f, " ({details})")
        } else {
            Ok(())
        }
    }
}

pub trait Meta {
    fn meta(&self, easing: &dyn Easing) -> Vec<Information>;
}

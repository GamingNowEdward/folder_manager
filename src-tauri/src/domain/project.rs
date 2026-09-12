use uuid::Uuid;

use crate::domain::folder::Folder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub folders: Vec<Folder>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            folders: Vec::new(),
        }
    }
}

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub path: String,
}

impl Folder {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            path: path.into(),
        }
    }
}

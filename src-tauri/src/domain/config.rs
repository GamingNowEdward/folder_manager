use std::collections::HashSet;

use uuid::Uuid;

use crate::domain::project::Project;

pub const CONFIG_VERSION: u32 = 2;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Workspace {
    pub current_project_id: String,
    pub projects: Vec<Project>,
}

impl Workspace {
    pub fn normalize(&mut self) {
        let mut project_ids: HashSet<String> = HashSet::new();
        for project in &mut self.projects {
            if project.id.is_empty() || !project_ids.insert(project.id.clone()) {
                project.id = Uuid::new_v4().to_string();
                project_ids.insert(project.id.clone());
            }

            let mut folder_ids: HashSet<String> = HashSet::new();
            for folder in &mut project.folders {
                if folder.id.is_empty() || !folder_ids.insert(folder.id.clone()) {
                    folder.id = Uuid::new_v4().to_string();
                    folder_ids.insert(folder.id.clone());
                }
            }
        }

        if !self
            .projects
            .iter()
            .any(|project| project.id == self.current_project_id)
        {
            self.current_project_id.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::folder::Folder;

    #[test]
    fn normalize_assigns_missing_ids() {
        let mut workspace = Workspace {
            current_project_id: String::new(),
            projects: vec![Project {
                id: String::new(),
                name: "工作".to_string(),
                folders: vec![Folder {
                    id: String::new(),
                    name: "A".to_string(),
                    path: "C:\\A".to_string(),
                }],
            }],
        };

        workspace.normalize();

        assert!(!workspace.projects[0].id.is_empty());
        assert!(!workspace.projects[0].folders[0].id.is_empty());
        assert!(workspace.current_project_id.is_empty());
    }

    #[test]
    fn normalize_replaces_duplicate_ids() {
        let mut workspace = Workspace {
            current_project_id: String::new(),
            projects: vec![
                Project {
                    id: "dup".to_string(),
                    name: "A".to_string(),
                    folders: Vec::new(),
                },
                Project {
                    id: "dup".to_string(),
                    name: "B".to_string(),
                    folders: Vec::new(),
                },
            ],
        };

        workspace.normalize();

        assert_ne!(workspace.projects[0].id, workspace.projects[1].id);
        assert_eq!(workspace.projects[0].id, "dup");
    }

    #[test]
    fn normalize_keeps_valid_current_project() {
        let mut workspace = Workspace {
            current_project_id: "p1".to_string(),
            projects: vec![Project {
                id: "p1".to_string(),
                name: "A".to_string(),
                folders: Vec::new(),
            }],
        };

        workspace.normalize();

        assert_eq!(workspace.current_project_id, "p1");
        assert_eq!(workspace.projects[0].id, "p1");
    }

    #[test]
    fn normalize_clears_unknown_current_project() {
        let mut workspace = Workspace {
            current_project_id: "missing".to_string(),
            projects: vec![Project {
                id: "p1".to_string(),
                name: "A".to_string(),
                folders: Vec::new(),
            }],
        };

        workspace.normalize();

        assert!(workspace.current_project_id.is_empty());
    }
}

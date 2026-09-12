use std::path::PathBuf;
use std::sync::Mutex;

use tauri::AppHandle;

use crate::domain::config::Workspace;
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::json_store;

pub struct ConfigService {
    path: PathBuf,
    state: Mutex<Workspace>,
}

impl ConfigService {
    pub fn initialize(app: &AppHandle) -> AppResult<Self> {
        let path = json_store::resolve_config_path(app)?;
        Self::open(path)
    }

    fn open(path: PathBuf) -> AppResult<Self> {
        let workspace = json_store::read_workspace(&path)?.unwrap_or_default();
        Ok(Self {
            path,
            state: Mutex::new(workspace),
        })
    }

    pub fn load(&self) -> AppResult<Workspace> {
        self.state
            .lock()
            .map_err(|error| AppError::Lock(error.to_string()))
            .map(|workspace| workspace.clone())
    }

    pub fn save(&self, mut workspace: Workspace) -> AppResult<()> {
        workspace.normalize();
        {
            let mut current = self
                .state
                .lock()
                .map_err(|error| AppError::Lock(error.to_string()))?;
            *current = workspace.clone();
        }
        json_store::write_workspace(&self.path, &workspace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project::Project;

    #[test]
    fn save_normalizes_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let service = ConfigService::open(path.clone()).unwrap();

        let project = Project::new("工作");
        let workspace = Workspace {
            current_project_id: project.id.clone(),
            projects: vec![project],
        };

        service.save(workspace.clone()).unwrap();

        assert_eq!(service.load().unwrap(), workspace);
        assert!(path.exists());
    }

    #[test]
    fn save_fills_missing_ids() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let service = ConfigService::open(path).unwrap();

        let workspace = Workspace {
            current_project_id: String::new(),
            projects: vec![Project {
                id: String::new(),
                name: "A".to_string(),
                folders: Vec::new(),
            }],
        };

        service.save(workspace).unwrap();

        let loaded = service.load().unwrap();
        assert!(!loaded.projects[0].id.is_empty());
    }
}

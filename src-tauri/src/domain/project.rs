use uuid::Uuid;

use crate::domain::folder::Folder;
use crate::error::{AppError, AppResult};

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

pub fn find_project_by_id<'a>(projects: &'a [Project], id: &str) -> Option<&'a Project> {
    projects.iter().find(|project| project.id == id)
}

pub fn find_project_mut<'a>(projects: &'a mut [Project], id: &str) -> Option<&'a mut Project> {
    projects.iter_mut().find(|project| project.id == id)
}

pub fn is_project_name_taken(projects: &[Project], name: &str, except_id: Option<&str>) -> bool {
    projects
        .iter()
        .any(|project| project.name == name && Some(project.id.as_str()) != except_id)
}

/// 创建项目并追加到列表；重名视为冲突。
pub fn create_project(projects: &mut Vec<Project>, name: &str) -> AppResult<Project> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidProjectName);
    }
    if is_project_name_taken(projects, trimmed, None) {
        return Err(AppError::ProjectNameTaken(trimmed.to_string()));
    }
    let project = Project::new(trimmed);
    projects.push(project.clone());
    Ok(project)
}

/// 重命名项目；返回更新后的项目。
pub fn rename_project(projects: &mut [Project], id: &str, name: &str) -> AppResult<Project> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidProjectName);
    }
    if is_project_name_taken(projects, trimmed, Some(id)) {
        return Err(AppError::ProjectNameTaken(trimmed.to_string()));
    }
    let project =
        find_project_mut(projects, id).ok_or_else(|| AppError::ProjectNotFound(id.into()))?;
    project.name = trimmed.to_string();
    Ok(project.clone())
}

/// 移除项目；返回被移除的项目。
pub fn remove_project(projects: &mut Vec<Project>, id: &str) -> AppResult<Project> {
    let index = projects
        .iter()
        .position(|project| project.id == id)
        .ok_or_else(|| AppError::ProjectNotFound(id.to_string()))?;
    Ok(projects.remove(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn projects() -> Vec<Project> {
        vec![
            Project {
                id: "p1".to_string(),
                name: "工作".to_string(),
                folders: Vec::new(),
            },
            Project {
                id: "p2".to_string(),
                name: "私人".to_string(),
                folders: Vec::new(),
            },
        ]
    }

    #[test]
    fn creates_project_with_generated_id() {
        let mut list = projects();
        let project = create_project(&mut list, "  新项目  ").unwrap();

        assert_eq!(project.name, "新项目");
        assert!(!project.id.is_empty());
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn rejects_duplicate_and_empty_project_names() {
        let mut list = projects();

        assert!(matches!(
            create_project(&mut list, "工作"),
            Err(AppError::ProjectNameTaken(_))
        ));
        assert!(matches!(
            create_project(&mut list, "   "),
            Err(AppError::InvalidProjectName)
        ));
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn renames_project_keeping_its_own_name() {
        let mut list = projects();

        assert_eq!(
            rename_project(&mut list, "p1", "工作").unwrap().name,
            "工作"
        );
        assert_eq!(
            rename_project(&mut list, "p1", "事业").unwrap().name,
            "事业"
        );
        assert!(matches!(
            rename_project(&mut list, "p1", "私人"),
            Err(AppError::ProjectNameTaken(_))
        ));
    }

    #[test]
    fn rename_reports_missing_project() {
        let mut list = projects();

        assert!(matches!(
            rename_project(&mut list, "missing", "A"),
            Err(AppError::ProjectNotFound(_))
        ));
    }

    #[test]
    fn removes_project() {
        let mut list = projects();
        let removed = remove_project(&mut list, "p1").unwrap();

        assert_eq!(removed.name, "工作");
        assert_eq!(list.len(), 1);
        assert!(matches!(
            remove_project(&mut list, "p1"),
            Err(AppError::ProjectNotFound(_))
        ));
    }
}

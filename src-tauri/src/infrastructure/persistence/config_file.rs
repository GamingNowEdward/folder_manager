use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::config::{Workspace, CONFIG_VERSION};
use crate::domain::folder::{now_ms, Folder};
use crate::domain::project::Project;
use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
struct LegacyFolder {
    #[serde(default)]
    name: String,
    #[serde(default)]
    path: String,
}

#[derive(Debug, Deserialize)]
struct LegacyProject {
    #[serde(default)]
    name: String,
    #[serde(default)]
    folders: Vec<LegacyFolder>,
}

#[derive(Debug, Deserialize)]
struct LegacyConfig {
    #[serde(default)]
    current_project: String,
    #[serde(default)]
    projects: Vec<LegacyProject>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderFile {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub path: String,
    /// 添加时间（Unix 毫秒）。v2 及更早没有这个字段，迁移时按数组顺序补齐。
    #[serde(default)]
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFile {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub folders: Vec<FolderFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigFile {
    pub version: u32,
    #[serde(default)]
    pub current_project: String,
    #[serde(default)]
    pub projects: Vec<ProjectFile>,
}

impl ConfigFile {
    pub fn from_workspace(workspace: &Workspace) -> Self {
        Self {
            version: CONFIG_VERSION,
            current_project: workspace.current_project_id.clone(),
            projects: workspace.projects.iter().map(project_to_file).collect(),
        }
    }

    fn into_workspace(self) -> Workspace {
        Workspace {
            current_project_id: self.current_project,
            projects: self
                .projects
                .into_iter()
                .map(|project| Project {
                    id: project.id,
                    name: project.name,
                    folders: project
                        .folders
                        .into_iter()
                        .map(|folder| Folder {
                            id: folder.id,
                            name: folder.name,
                            path: folder.path,
                            created_at: folder.created_at,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

fn project_to_file(project: &Project) -> ProjectFile {
    ProjectFile {
        id: project.id.clone(),
        name: project.name.clone(),
        folders: project
            .folders
            .iter()
            .map(|folder| FolderFile {
                id: folder.id.clone(),
                name: folder.name.clone(),
                path: folder.path.clone(),
                created_at: folder.created_at,
            })
            .collect(),
    }
}

pub fn parse_config(value: Value) -> AppResult<Workspace> {
    let version = value.get("version").and_then(Value::as_u64);
    match version {
        None | Some(1) => {
            let legacy: LegacyConfig = serde_json::from_value(value)
                .map_err(|error| AppError::ParseConfig(error.to_string()))?;
            let mut workspace = migrate_legacy(legacy);
            backfill_created_at(&mut workspace, now_ms());
            Ok(workspace)
        }
        // v2 没有 created_at、v3 可能有缺失项：补齐后再进入业务层。
        Some(found) if found == 2 || found == CONFIG_VERSION as u64 => {
            let file: ConfigFile = serde_json::from_value(value)
                .map_err(|error| AppError::ParseConfig(error.to_string()))?;
            let mut workspace = file.into_workspace();
            workspace.normalize();
            backfill_created_at(&mut workspace, now_ms());
            Ok(workspace)
        }
        Some(found) => Err(AppError::UnsupportedVersion(found)),
    }
}

/// 为缺少添加时间的文件夹补时间戳。
///
/// 只要一个项目里出现 `created_at <= 0`（v2 迁移或手工改配置），就**按数组顺序重建整个项目**：
/// 第一个最早、最后一个最新，保证迁移后「按添加日期」的结果与用户原本看到的顺序一致。
/// 时间值本身不参与「自定义顺序」，所以重建不会改变界面顺序。
fn backfill_created_at(workspace: &mut Workspace, base_ms: i64) {
    for project in &mut workspace.projects {
        if !project.folders.iter().any(|folder| folder.created_at <= 0) {
            continue;
        }
        let len = project.folders.len() as i64;
        for (index, folder) in project.folders.iter_mut().enumerate() {
            folder.created_at = base_ms - (len - 1 - index as i64);
        }
    }
}

fn migrate_legacy(legacy: LegacyConfig) -> Workspace {
    let mut workspace = Workspace::default();
    for project in legacy.projects {
        let mut migrated = Project::new(project.name);
        migrated.folders = project
            .folders
            .into_iter()
            // created_at 留 0，交给 `backfill_created_at` 按数组顺序统一补齐
            .map(|folder| Folder {
                id: String::new(),
                name: folder.name,
                path: folder.path,
                created_at: 0,
            })
            .collect();
        workspace.projects.push(migrated);
    }
    workspace.current_project_id = workspace
        .projects
        .iter()
        .find(|project| project.name == legacy.current_project)
        .map(|project| project.id.clone())
        .unwrap_or_default();
    workspace.normalize();
    workspace
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn migrates_unversioned_config() {
        let value = json!({
            "current_project": "工作",
            "projects": [
                { "name": "工作", "folders": [{ "name": "A", "path": "C:\\A" }] },
                { "name": "私人", "folders": [] }
            ]
        });

        let workspace = parse_config(value).unwrap();

        assert_eq!(workspace.projects.len(), 2);
        assert_eq!(workspace.projects[0].name, "工作");
        assert!(!workspace.projects[0].id.is_empty());
        assert!(!workspace.projects[0].folders[0].id.is_empty());
        assert_eq!(workspace.current_project_id, workspace.projects[0].id);
    }

    #[test]
    fn migrates_version_one_config() {
        let value = json!({
            "version": 1,
            "current_project": "工作",
            "projects": [{ "name": "工作", "folders": [] }]
        });

        let workspace = parse_config(value).unwrap();

        assert_eq!(workspace.current_project_id, workspace.projects[0].id);
    }

    #[test]
    fn backfills_created_at_in_array_order_when_migrating_v2() {
        let value = json!({
            "version": 2,
            "current_project": "p1",
            "projects": [{
                "id": "p1",
                "name": "工作",
                "folders": [
                    { "id": "f1", "name": "B", "path": "C:\\B" },
                    { "id": "f2", "name": "A", "path": "C:\\A" },
                    { "id": "f3", "name": "C", "path": "C:\\C" }
                ]
            }]
        });

        let workspace = parse_config(value).unwrap();
        let times: Vec<i64> = workspace.projects[0]
            .folders
            .iter()
            .map(|folder| folder.created_at)
            .collect();

        assert!(times.iter().all(|time| *time > 0), "迁移后必须有添加时间");
        assert!(
            times[0] < times[1] && times[1] < times[2],
            "数组顺序必须与添加时间一致: {times:?}"
        );
    }

    #[test]
    fn keeps_existing_created_at_for_v3() {
        let value = json!({
            "version": 3,
            "current_project": "p1",
            "projects": [{
                "id": "p1",
                "name": "工作",
                "folders": [
                    { "id": "f1", "name": "B", "path": "C:\\B", "created_at": 111 },
                    { "id": "f2", "name": "A", "path": "C:\\A", "created_at": 222 }
                ]
            }]
        });

        let workspace = parse_config(value).unwrap();

        assert_eq!(workspace.projects[0].folders[0].created_at, 111);
        assert_eq!(workspace.projects[0].folders[1].created_at, 222);
    }

    #[test]
    fn backfills_missing_created_at_in_v3_without_losing_order() {
        let value = json!({
            "version": 3,
            "current_project": "p1",
            "projects": [{
                "id": "p1",
                "name": "工作",
                "folders": [
                    { "id": "f1", "name": "B", "path": "C:\\B" },
                    { "id": "f2", "name": "A", "path": "C:\\A", "created_at": 222 }
                ]
            }]
        });

        let workspace = parse_config(value).unwrap();
        let times: Vec<i64> = workspace.projects[0]
            .folders
            .iter()
            .map(|folder| folder.created_at)
            .collect();

        assert!(times.iter().all(|time| *time > 0));
        assert!(times[0] < times[1], "缺失时间后仍要保留数组顺序");
    }

    #[test]
    fn keeps_v2_ids_and_current_project() {
        let value = json!({
            "version": 2,
            "current_project": "p1",
            "projects": [
                { "id": "p1", "name": "工作", "folders": [{ "id": "f1", "name": "A", "path": "C:\\A" }] }
            ]
        });

        let workspace = parse_config(value).unwrap();

        assert_eq!(workspace.current_project_id, "p1");
        assert_eq!(workspace.projects[0].id, "p1");
        assert_eq!(workspace.projects[0].folders[0].id, "f1");
    }

    #[test]
    fn fills_missing_v2_ids() {
        let value = json!({
            "version": 2,
            "current_project": "",
            "projects": [{ "name": "工作", "folders": [{ "name": "A", "path": "C:\\A" }] }]
        });

        let workspace = parse_config(value).unwrap();

        assert!(!workspace.projects[0].id.is_empty());
        assert!(!workspace.projects[0].folders[0].id.is_empty());
    }

    #[test]
    fn clears_unknown_current_project() {
        let value = json!({ "version": 2, "current_project": "missing", "projects": [] });

        let workspace = parse_config(value).unwrap();

        assert!(workspace.current_project_id.is_empty());
    }

    #[test]
    fn rejects_unsupported_version() {
        let value = json!({ "version": 99, "projects": [] });

        assert!(matches!(
            parse_config(value),
            Err(AppError::UnsupportedVersion(99))
        ));
    }

    #[test]
    fn roundtrips_workspace_through_file_model() {
        let workspace = parse_config(json!({
            "current_project": "工作",
            "projects": [{ "name": "工作", "folders": [{ "name": "A", "path": "C:\\A" }] }]
        }))
        .unwrap();

        let file = ConfigFile::from_workspace(&workspace);
        let value = serde_json::to_value(file).unwrap();
        let reparsed = parse_config(value).unwrap();

        assert_eq!(reparsed, workspace);
    }
}

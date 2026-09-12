use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tauri::AppHandle;

use crate::domain::config::Workspace;
use crate::domain::folder::{self, AddFolderOutcome, Folder, FolderInput};
use crate::domain::project::{self, Project};
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::json_store;

/// 配置变更事件名。前端通过 Tauri event 监听它并重新 hydrate Pinia store。
pub const WORKSPACE_CHANGED_EVENT: &str = "workspace-changed";

/// 变更事件载荷。只带修订号与来源，前端收到后调用 `load_config` 拉取权威快照。
#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkspaceChanged {
    pub revision: u64,
    pub source: &'static str,
}

/// 配置变更事件的可选出口。`ConfigService` 归属 application 层，
/// 不直接依赖 Tauri，因此用窄接口 + 组合根注入。
pub trait ChangeSink: Send + Sync {
    fn on_workspace_changed(&self, payload: &WorkspaceChanged);
}

/// 通过 Tauri app handle 把事件广播给前端窗口。
pub struct AppEventHandle(AppHandle);

impl AppEventHandle {
    pub fn new(app: AppHandle) -> Self {
        Self(app)
    }
}

impl ChangeSink for AppEventHandle {
    fn on_workspace_changed(&self, payload: &WorkspaceChanged) {
        use tauri::Emitter;
        if let Err(error) = self.0.emit(WORKSPACE_CHANGED_EVENT, payload) {
            eprintln!("[folder-manager] 广播配置变更事件失败: {error}");
        }
    }
}

/// 配置的唯一权威状态：内存快照 + config.json。
/// Tauri command 与本地 HTTP API 共用同一个实例，因此共用同一套业务规则与持久化。
pub struct ConfigService {
    path: PathBuf,
    state: Mutex<Workspace>,
    revision: AtomicU64,
    sink: Option<Box<dyn ChangeSink>>,
}

impl ConfigService {
    pub fn initialize(app: &AppHandle, sink: Option<Box<dyn ChangeSink>>) -> AppResult<Self> {
        let path = json_store::resolve_config_path(app)?;
        Self::open_with_sink(path, sink)
    }

    /// 打开指定路径的配置（测试与无 Tauri 上下文场景）。
    #[cfg(test)]
    pub(crate) fn open(path: PathBuf) -> AppResult<Self> {
        Self::open_with_sink(path, None)
    }

    fn open_with_sink(path: PathBuf, sink: Option<Box<dyn ChangeSink>>) -> AppResult<Self> {
        let workspace = json_store::read_workspace(&path)?.unwrap_or_default();
        Ok(Self {
            path,
            state: Mutex::new(workspace),
            revision: AtomicU64::new(1),
            sink,
        })
    }

    /// 注入变更事件出口（测试使用；组合根通过 `initialize` 直接传入）。
    #[cfg(test)]
    pub fn with_sink(mut self, sink: Box<dyn ChangeSink>) -> Self {
        self.sink = Some(sink);
        self
    }

    /// 共享句柄：Tauri managed state 与 HTTP API state 指向同一个实例。
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Acquire)
    }

    pub fn config_path(&self) -> &std::path::Path {
        &self.path
    }

    /// 读取当前工作区快照（只读，不改动任何状态）。
    pub fn load(&self) -> AppResult<Workspace> {
        self.state
            .lock()
            .map_err(|error| AppError::Lock(error.to_string()))
            .map(|workspace| workspace.clone())
    }

    /// 快照式全量替换（`save_config` 路径）。保留既有语义：规范化后落盘。
    pub fn save(&self, workspace: Workspace) -> AppResult<()> {
        self.replace_workspace(workspace)?;
        self.notify("snapshot");
        Ok(())
    }

    pub fn create_project(&self, name: &str) -> AppResult<Project> {
        let created = {
            let mut workspace = self.lock_mut()?;
            let created = project::create_project(&mut workspace.projects, name)?;
            if workspace.current_project_id.is_empty() {
                workspace.current_project_id = created.id.clone();
            }
            created
        };
        self.persist(WORKSPACE_CHANGED_EVENT)?;
        Ok(created)
    }

    pub fn update_project(&self, project_id: &str, name: &str) -> AppResult<Project> {
        let updated = {
            let mut workspace = self.lock_mut()?;
            project::rename_project(&mut workspace.projects, project_id, name)?
        };
        self.persist(WORKSPACE_CHANGED_EVENT)?;
        Ok(updated)
    }

    pub fn delete_project(&self, project_id: &str) -> AppResult<Project> {
        let removed = {
            let mut workspace = self.lock_mut()?;
            let removed = project::remove_project(&mut workspace.projects, project_id)?;
            if workspace.current_project_id == removed.id {
                workspace.current_project_id = workspace
                    .projects
                    .first()
                    .map(|project| project.id.clone())
                    .unwrap_or_default();
            }
            removed
        };
        self.persist(WORKSPACE_CHANGED_EVENT)?;
        Ok(removed)
    }

    pub fn get_project(&self, project_id: &str) -> AppResult<Project> {
        let workspace = self.load()?;
        project::find_project_by_id(&workspace.projects, project_id)
            .cloned()
            .ok_or_else(|| AppError::ProjectNotFound(project_id.to_string()))
    }

    /// 向项目添加文件夹；同一 path 幂等（命中已有项时不再写盘、不再广播）。
    pub fn add_folder(&self, project_id: &str, input: FolderInput) -> AppResult<AddFolderOutcome> {
        let outcome = {
            let mut workspace = self.lock_mut()?;
            let project = project::find_project_mut(&mut workspace.projects, project_id)
                .ok_or_else(|| AppError::ProjectNotFound(project_id.to_string()))?;
            folder::add_folder(&mut project.folders, input)?
        };
        if outcome.created() {
            self.persist(WORKSPACE_CHANGED_EVENT)?;
        }
        Ok(outcome)
    }

    pub fn remove_folder_by_id(&self, project_id: &str, folder_id: &str) -> AppResult<Folder> {
        let removed = {
            let mut workspace = self.lock_mut()?;
            let project = project::find_project_mut(&mut workspace.projects, project_id)
                .ok_or_else(|| AppError::ProjectNotFound(project_id.to_string()))?;
            folder::remove_folder_by_id(&mut project.folders, folder_id)?
        };
        self.persist(WORKSPACE_CHANGED_EVENT)?;
        Ok(removed)
    }

    pub fn remove_folder_by_path(&self, project_id: &str, path: &str) -> AppResult<Folder> {
        let removed = {
            let mut workspace = self.lock_mut()?;
            let project = project::find_project_mut(&mut workspace.projects, project_id)
                .ok_or_else(|| AppError::ProjectNotFound(project_id.to_string()))?;
            folder::remove_folder_by_path(&mut project.folders, path)?
        };
        self.persist(WORKSPACE_CHANGED_EVENT)?;
        Ok(removed)
    }

    /// 规范化 → 更新内存 → 落盘。不广播（快照式保存由调用方决定是否广播）。
    fn replace_workspace(&self, mut workspace: Workspace) -> AppResult<()> {
        workspace.normalize();
        {
            let mut current = self.lock_mut()?;
            *current = workspace.clone();
        }
        self.revision.fetch_add(1, Ordering::AcqRel);
        json_store::write_workspace(&self.path, &workspace)
    }

    /// 把内存状态落盘，成功后广播变更。写盘失败时不广播，前端也就不会读到半成品。
    fn persist(&self, source: &'static str) -> AppResult<()> {
        let workspace = self.load()?;
        self.revision.fetch_add(1, Ordering::AcqRel);
        json_store::write_workspace(&self.path, &workspace)?;
        self.notify(source);
        Ok(())
    }

    fn notify(&self, source: &'static str) {
        if let Some(sink) = &self.sink {
            sink.on_workspace_changed(&WorkspaceChanged {
                revision: self.revision(),
                source,
            });
        }
    }

    /// 只改内存、不落盘；调用方随后必须 `persist()`。
    fn lock_mut(&self) -> AppResult<std::sync::MutexGuard<'_, Workspace>> {
        self.state
            .lock()
            .map_err(|error| AppError::Lock(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[derive(Default)]
    struct CountingSink {
        calls: AtomicUsize,
    }

    impl CountingSink {
        fn calls(&self) -> usize {
            self.calls.load(Ordering::Acquire)
        }
    }

    impl ChangeSink for CountingSink {
        fn on_workspace_changed(&self, _payload: &WorkspaceChanged) {
            self.calls.fetch_add(1, Ordering::AcqRel);
        }
    }

    struct SharedSink(Arc<CountingSink>);

    impl ChangeSink for SharedSink {
        fn on_workspace_changed(&self, payload: &WorkspaceChanged) {
            self.0.on_workspace_changed(payload);
        }
    }

    fn service_in(dir: &tempfile::TempDir) -> ConfigService {
        ConfigService::open(dir.path().join("config.json")).unwrap()
    }

    fn service_with_sink(dir: &tempfile::TempDir) -> (ConfigService, Arc<CountingSink>) {
        let sink = Arc::new(CountingSink::default());
        let service = service_in(dir).with_sink(Box::new(SharedSink(sink.clone())));
        (service, sink)
    }

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

    #[test]
    fn mutations_persist_and_bump_revision() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let service = ConfigService::open(path.clone()).unwrap();
        let before = service.revision();

        let project = service.create_project("Agent 项目").unwrap();
        assert_eq!(service.load().unwrap().current_project_id, project.id);

        let outcome = service
            .add_folder(&project.id, FolderInput::new("src", "C:\\work\\src"))
            .unwrap();
        assert!(outcome.created());

        let removed = service
            .remove_folder_by_id(&project.id, outcome.folder().id.as_str())
            .unwrap();
        assert_eq!(removed.name, "src");

        service.delete_project(&project.id).unwrap();

        assert!(service.load().unwrap().projects.is_empty());
        assert!(service.revision() > before);
        let on_disk = json_store::read_workspace(&path).unwrap().unwrap();
        assert!(on_disk.projects.is_empty());
    }

    #[test]
    fn add_folder_is_idempotent_and_does_not_rewrite() {
        let dir = tempfile::tempdir().unwrap();
        let service = service_in(&dir);
        let project = service.create_project("P").unwrap();
        let input = FolderInput::new("src", "C:\\work\\src");

        let first = service.add_folder(&project.id, input.clone()).unwrap();
        let revision = service.revision();
        let second = service.add_folder(&project.id, input).unwrap();

        assert!(first.created());
        assert!(!second.created());
        assert_eq!(first.folder().id, second.folder().id);
        assert_eq!(service.revision(), revision);
        assert_eq!(service.get_project(&project.id).unwrap().folders.len(), 1);
    }

    #[test]
    fn notifies_sink_after_persisting() {
        let dir = tempfile::tempdir().unwrap();
        let (service, sink) = service_with_sink(&dir);

        let project = service.create_project("P").unwrap();
        service.delete_project(&project.id).unwrap();

        assert_eq!(sink.calls(), 2);
    }

    #[test]
    fn does_not_notify_when_operation_fails() {
        let dir = tempfile::tempdir().unwrap();
        let (service, sink) = service_with_sink(&dir);

        assert!(service.delete_project("missing").is_err());

        assert_eq!(sink.calls(), 0);
    }

    #[test]
    fn delete_current_project_selects_first_remaining() {
        let dir = tempfile::tempdir().unwrap();
        let service = service_in(&dir);
        let first = service.create_project("A").unwrap();
        let second = service.create_project("B").unwrap();

        service.delete_project(&first.id).unwrap();

        assert_eq!(service.load().unwrap().current_project_id, second.id);
    }

    #[test]
    fn add_folder_reports_missing_project() {
        let dir = tempfile::tempdir().unwrap();
        let service = service_in(&dir);

        assert!(matches!(
            service.add_folder("missing", FolderInput::new("src", "C:\\work\\src")),
            Err(AppError::ProjectNotFound(_))
        ));
    }
}

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
    /// 当前对外可见的工作区（落盘成功后才会变化）。
    state: Mutex<Workspace>,
    /// 最近一次成功落盘的工作区；写盘失败时用它回滚 `state`。
    committed: Mutex<Workspace>,
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
            committed: Mutex::new(workspace.clone()),
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
        self.transact("snapshot", |current| {
            let mut next = workspace.clone();
            next.normalize();
            *current = next;
            Ok(Mutation::Commit(()))
        })
    }

    pub fn create_project(&self, name: &str) -> AppResult<Project> {
        self.transact(WORKSPACE_CHANGED_EVENT, |workspace| {
            let created = project::create_project(&mut workspace.projects, name)?;
            if workspace.current_project_id.is_empty() {
                workspace.current_project_id = created.id.clone();
            }
            Ok(Mutation::Commit(created))
        })
    }

    pub fn update_project(&self, project_id: &str, name: &str) -> AppResult<Project> {
        self.transact(WORKSPACE_CHANGED_EVENT, |workspace| {
            let updated = project::rename_project(&mut workspace.projects, project_id, name)?;
            Ok(Mutation::Commit(updated))
        })
    }

    pub fn delete_project(&self, project_id: &str) -> AppResult<Project> {
        self.transact(WORKSPACE_CHANGED_EVENT, |workspace| {
            let removed = project::remove_project(&mut workspace.projects, project_id)?;
            if workspace.current_project_id == removed.id {
                workspace.current_project_id = workspace
                    .projects
                    .first()
                    .map(|project| project.id.clone())
                    .unwrap_or_default();
            }
            Ok(Mutation::Commit(removed))
        })
    }

    pub fn get_project(&self, project_id: &str) -> AppResult<Project> {
        let workspace = self.load()?;
        project::find_project_by_id(&workspace.projects, project_id)
            .cloned()
            .ok_or_else(|| AppError::ProjectNotFound(project_id.to_string()))
    }

    /// 向项目添加文件夹；同一 path 幂等（命中已有项时不写盘、不递增 revision、不广播）。
    pub fn add_folder(&self, project_id: &str, input: FolderInput) -> AppResult<AddFolderOutcome> {
        self.transact(WORKSPACE_CHANGED_EVENT, |workspace| {
            let project = project::find_project_mut(&mut workspace.projects, project_id)
                .ok_or_else(|| AppError::ProjectNotFound(project_id.to_string()))?;
            match folder::add_folder(&mut project.folders, input)? {
                outcome if outcome.created() => Ok(Mutation::Commit(outcome)),
                // 幂等命中：不产生任何可观察变更
                outcome => Ok(Mutation::Noop(outcome)),
            }
        })
    }

    /// 按 id 移除「文件夹引用」（只改配置，**不会删除真实目录**）。
    pub fn remove_folder_reference_by_id(
        &self,
        project_id: &str,
        folder_id: &str,
    ) -> AppResult<Folder> {
        self.transact(WORKSPACE_CHANGED_EVENT, |workspace| {
            let project = project::find_project_mut(&mut workspace.projects, project_id)
                .ok_or_else(|| AppError::ProjectNotFound(project_id.to_string()))?;
            Ok(Mutation::Commit(folder::remove_folder_by_id(
                &mut project.folders,
                folder_id,
            )?))
        })
    }

    /// 按 path 移除「文件夹引用」（只改配置，**不会删除真实目录**）。
    pub fn remove_folder_reference_by_path(
        &self,
        project_id: &str,
        path: &str,
    ) -> AppResult<Folder> {
        self.transact(WORKSPACE_CHANGED_EVENT, |workspace| {
            let project = project::find_project_mut(&mut workspace.projects, project_id)
                .ok_or_else(|| AppError::ProjectNotFound(project_id.to_string()))?;
            Ok(Mutation::Commit(folder::remove_folder_by_path(
                &mut project.folders,
                path,
            )?))
        })
    }

    /// 唯一的写事务入口：**持锁** 完成「读取 → mutation → 落盘 → 提交内存状态 →
    /// 递增 revision → 广播」。
    ///
    /// 这样保证：
    /// - 不会有并发 mutation 互相覆盖（不存在「读旧快照 → 释放锁 → 写回」的窗口）；
    /// - 只有写盘成功才提交内存状态，所以 `load()` 读到的永远与磁盘一致；
    /// - `revision` 只在成功提交后递增，因此单调不倒退，也不会为失败的操作递增；
    /// - 失败时回滚到最近一次成功落盘的状态，并且不广播 `workspace-changed`。
    fn transact<T>(
        &self,
        source: &'static str,
        mutate: impl FnOnce(&mut Workspace) -> AppResult<Mutation<T>>,
    ) -> AppResult<T> {
        let mut guard = self
            .state
            .lock()
            .map_err(|error| AppError::Lock(error.to_string()))?;

        match mutate(&mut guard)? {
            Mutation::Noop(value) => Ok(value),
            Mutation::Commit(value) => {
                let candidate = guard.clone();
                if let Err(error) = self.persist_locked(&candidate) {
                    // 回滚到最近一次成功落盘的状态，保证内存与磁盘一致
                    let committed = self
                        .committed
                        .lock()
                        .map_err(|lock| AppError::Lock(lock.to_string()))?;
                    *guard = committed.clone();
                    return Err(error);
                }
                {
                    let mut committed = self
                        .committed
                        .lock()
                        .map_err(|lock| AppError::Lock(lock.to_string()))?;
                    *committed = candidate;
                }
                drop(guard);
                self.revision.fetch_add(1, Ordering::AcqRel);
                self.notify(source);
                Ok(value)
            }
        }
    }

    /// 在持有锁的前提下把工作区写盘（失败时由调用方回滚内存状态）。
    fn persist_locked(&self, workspace: &Workspace) -> AppResult<()> {
        json_store::write_workspace(&self.path, workspace)
    }

    fn notify(&self, source: &'static str) {
        if let Some(sink) = &self.sink {
            sink.on_workspace_changed(&WorkspaceChanged {
                revision: self.revision(),
                source,
            });
        }
    }
}

/// 一次 mutation 的结果：`Commit` 需要落盘 + 广播，`Noop` 不改动任何可观察状态。
enum Mutation<T> {
    Commit(T),
    Noop(T),
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
            .remove_folder_reference_by_id(&project.id, outcome.folder().id.as_str())
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

    /// 让后续 `write_workspace` 必然失败：把临时文件 `config.json.tmp` 的路径占成目录，
    /// `write_atomically` 的第一步 `fs::write` 就会失败。
    fn block_persistence(dir: &tempfile::TempDir) {
        let temp_path = dir.path().join("config.json.tmp");
        let _ = std::fs::remove_file(&temp_path);
        std::fs::create_dir(&temp_path).unwrap();
    }

    fn unblock_persistence(dir: &tempfile::TempDir) {
        let _ = std::fs::remove_dir(dir.path().join("config.json.tmp"));
    }

    #[test]
    fn failed_persistence_keeps_memory_disk_and_revision_consistent() {
        let dir = tempfile::tempdir().unwrap();
        let (service, sink) = service_with_sink(&dir);
        let project = service.create_project("P").unwrap();
        let revision_before = service.revision();

        block_persistence(&dir);
        let result = service.add_folder(&project.id, FolderInput::new("src", "C:\\work\\src"));

        // 1) 调用方拿到错误（HTTP 层会映射成 500）
        assert!(result.is_err(), "写盘失败必须向上返回错误");
        // 2) 内存状态没有被提交（否则 GET 会返回磁盘上没有的数据）
        assert!(
            service.get_project(&project.id).unwrap().folders.is_empty(),
            "写盘失败后内存状态必须回滚"
        );
        // 3) revision 不为失败的操作递增
        assert_eq!(service.revision(), revision_before);
        // 4) 不广播 workspace-changed（只有前面 create_project 那一次）
        assert_eq!(sink.calls(), 1);

        // 5) 内存与磁盘内容一致
        let on_disk = json_store::read_workspace(&dir.path().join("config.json"))
            .unwrap()
            .unwrap();
        assert_eq!(on_disk, service.load().unwrap());

        // 恢复写盘能力后可以正常提交，磁盘与内存再次一致
        unblock_persistence(&dir);
        let outcome = service
            .add_folder(&project.id, FolderInput::new("src", "C:\\work\\src"))
            .unwrap();
        assert!(outcome.created());
        let on_disk = json_store::read_workspace(&dir.path().join("config.json"))
            .unwrap()
            .unwrap();
        assert_eq!(on_disk, service.load().unwrap());
        assert!(service.revision() > revision_before);
    }

    #[test]
    fn failed_snapshot_save_keeps_previous_state() {
        let dir = tempfile::tempdir().unwrap();
        let service = service_in(&dir);
        let original = service.create_project("原始").unwrap();
        let revision_before = service.revision();

        block_persistence(&dir);
        let mut incoming = service.load().unwrap();
        incoming.projects.push(Project::new("来自前端"));

        assert!(service.save(incoming).is_err());

        let after = service.load().unwrap();
        let expected = Workspace {
            current_project_id: original.id.clone(),
            projects: vec![original],
        };
        assert_eq!(after, expected);
        assert_eq!(service.revision(), revision_before);

        // 磁盘上也没有出现「来自前端」这个项目
        unblock_persistence(&dir);
        let on_disk = json_store::read_workspace(&dir.path().join("config.json"))
            .unwrap()
            .unwrap();
        assert_eq!(on_disk, after);
        assert!(on_disk.projects.iter().all(|it| it.name != "来自前端"));
    }

    #[test]
    fn concurrent_adds_do_not_lose_updates() {
        let dir = tempfile::tempdir().unwrap();
        let service = Arc::new(service_in(&dir));
        let project = service.create_project("并发").unwrap();
        let revision_before = service.revision();

        let threads: Vec<_> = (0..16)
            .map(|index| {
                let service = Arc::clone(&service);
                let project_id = project.id.clone();
                std::thread::spawn(move || {
                    service
                        .add_folder(
                            &project_id,
                            FolderInput::new(
                                format!("folder-{index}"),
                                format!("C:\\work\\folder-{index}"),
                            ),
                        )
                        .unwrap()
                        .created()
                })
            })
            .collect();

        let created: Vec<bool> = threads
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();

        assert!(created.iter().all(|it| *it), "每个不同路径都应创建成功");
        let stored = service.get_project(&project.id).unwrap();
        assert_eq!(stored.folders.len(), 16);
        let mut ids: Vec<&str> = stored.folders.iter().map(|it| it.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 16, "id 不能重复");
        assert_eq!(service.revision(), revision_before + 16);
        // 磁盘与内存完全一致（没有旧快照覆盖新数据）
        let on_disk = json_store::read_workspace(&dir.path().join("config.json"))
            .unwrap()
            .unwrap();
        assert_eq!(on_disk, service.load().unwrap());
    }

    #[test]
    fn concurrent_adds_of_same_path_stay_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let service = Arc::new(service_in(&dir));
        let project = service.create_project("幂等").unwrap();

        let threads: Vec<_> = (0..16)
            .map(|_| {
                let service = Arc::clone(&service);
                let project_id = project.id.clone();
                std::thread::spawn(move || {
                    let outcome = service
                        .add_folder(&project_id, FolderInput::new("same", "C:\\work\\same"))
                        .unwrap();
                    (outcome.created(), outcome.folder().id.clone())
                })
            })
            .collect();

        let results: Vec<(bool, String)> = threads
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();

        assert_eq!(
            results.iter().filter(|(created, _)| *created).count(),
            1,
            "并发添加同一路径只能创建一次"
        );
        let stored = service.get_project(&project.id).unwrap();
        assert_eq!(stored.folders.len(), 1);
        let ids: Vec<&str> = results.iter().map(|(_, id)| id.as_str()).collect();
        assert!(ids.iter().all(|id| *id == stored.folders[0].id));
    }

    #[test]
    fn concurrent_mixed_mutations_keep_revision_monotonic_and_consistent() {
        let dir = tempfile::tempdir().unwrap();
        let service = Arc::new(service_in(&dir));
        let project = service.create_project("混合").unwrap();
        let target = service
            .add_folder(&project.id, FolderInput::new("target", "C:\\work\\target"))
            .unwrap()
            .into_folder();

        let mut handles = Vec::new();
        for index in 0..8 {
            let service = Arc::clone(&service);
            let project_id = project.id.clone();
            handles.push(std::thread::spawn(move || {
                // 交替添加 / 删除，制造读写交错
                let folder_id = service
                    .add_folder(
                        &project_id,
                        FolderInput::new(format!("f{index}"), format!("C:\\work\\f{index}")),
                    )
                    .unwrap()
                    .into_folder()
                    .id;
                service
                    .remove_folder_reference_by_id(&project_id, &folder_id)
                    .unwrap();
            }));
        }
        {
            let service = Arc::clone(&service);
            let project_id = project.id.clone();
            let target_id = target.id.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..8 {
                    // 读取与写入并发，读取不应看到半成品状态
                    let workspace = service.load().unwrap();
                    assert!(workspace
                        .projects
                        .iter()
                        .any(|project| project.id == project_id));
                    let _ = service
                        .remove_folder_reference_by_id(&project_id, &target_id)
                        .ok();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // revision 单调递增且磁盘 == 内存
        let revision = service.revision();
        assert!(revision >= 9);
        let on_disk = json_store::read_workspace(&dir.path().join("config.json"))
            .unwrap()
            .unwrap();
        assert_eq!(on_disk, service.load().unwrap());
        // 某个线程删除了 target，另一个线程的重复删除只能是 FolderNotFound
        assert!(service
            .get_project(&project.id)
            .unwrap()
            .folders
            .iter()
            .all(|folder| folder.id != target.id));
    }

    #[test]
    fn corrupt_config_is_quarantined_and_service_stays_usable() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        std::fs::write(&config_path, "{ this is not json").unwrap();

        let service = ConfigService::open(config_path).unwrap();

        assert!(service.load().unwrap().projects.is_empty());
        let project = service.create_project("恢复").unwrap();
        assert_eq!(service.get_project(&project.id).unwrap().name, "恢复");
        let quarantined = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .any(|entry| entry.file_name().to_string_lossy().contains(".corrupt-"));
        assert!(quarantined, "损坏的配置应被隔离而不是被覆盖");
    }
}

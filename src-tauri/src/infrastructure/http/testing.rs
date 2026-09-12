//! HTTP API 测试夹具：临时 config.json + 共享 ConfigService + 直接驱动 router。
//!
//! 测试全程使用 `tempfile::tempdir()`，不会读写用户真实的 `config.json`。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use tempfile::TempDir;
use tower::ServiceExt;

use crate::application::config_service::ConfigService;
use crate::domain::config::Workspace;
use crate::domain::folder::Folder;
use crate::domain::project::Project;
use crate::infrastructure::http::api::{router, ApiState};
use crate::infrastructure::persistence::json_store;

pub struct SampleWorkspace {
    pub workspace: Workspace,
}

pub struct Fixture {
    pub temp: TempDir,
    pub workspace: SampleWorkspace,
    pub service: Arc<ConfigService>,
    pub router: Router,
}

impl Fixture {
    /// 建一个带两个项目、两个真实临时目录的配置。
    pub fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let work_dir = temp.path().join("work-src");
        let docs_dir = temp.path().join("private-docs");
        std::fs::create_dir(&work_dir).unwrap();
        std::fs::create_dir(&docs_dir).unwrap();

        let mut work = Project::new("工作");
        work.folders
            .push(Folder::new("src", work_dir.to_string_lossy()));
        let mut personal = Project::new("私人");
        personal
            .folders
            .push(Folder::new("docs", docs_dir.to_string_lossy()));

        let workspace = Workspace {
            current_project_id: work.id.clone(),
            projects: vec![work, personal],
        };

        let config_path = temp.path().join("config.json");
        json_store::write_workspace(&config_path, &workspace).unwrap();
        let service = ConfigService::open(config_path).unwrap().shared();
        let router = router(ApiState::new(service.clone(), env!("CARGO_PKG_VERSION")));

        Self {
            temp,
            workspace: SampleWorkspace { workspace },
            service,
            router,
        }
    }

    pub fn config_path(&self) -> PathBuf {
        self.temp.path().join("config.json")
    }

    pub fn current_project_id(&self) -> &str {
        &self.workspace.workspace.current_project_id
    }

    /// 新建一个真实目录，返回 (项目 id, 目录路径)。
    pub fn new_directory(&self, name: &str) -> (String, PathBuf) {
        let dir = self.temp.path().join(name);
        std::fs::create_dir(&dir).unwrap();
        (self.current_project_id().to_string(), dir)
    }

    pub async fn request(&self, request: Request<Body>) -> (StatusCode, serde_json::Value) {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = if bytes.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
        };
        (status, body)
    }

    pub async fn get(&self, uri: &str) -> (StatusCode, serde_json::Value) {
        let request = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        self.request(request).await
    }

    pub async fn delete(&self, uri: &str) -> (StatusCode, serde_json::Value) {
        let request = Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        self.request(request).await
    }

    pub async fn post_json(
        &self,
        uri: &str,
        payload: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        self.send_json("POST", uri, payload).await
    }

    pub async fn put_json(
        &self,
        uri: &str,
        payload: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        self.send_json("PUT", uri, payload).await
    }

    async fn send_json(
        &self,
        method: &str,
        uri: &str,
        payload: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();
        self.request(request).await
    }
}

impl Default for Fixture {
    fn default() -> Self {
        Self::new()
    }
}

/// 断言 API 没有删除真实文件夹。
pub fn assert_directory_survives(path: &Path) {
    assert!(path.is_dir(), "目录被删除了: {}", path.display());
}

/// 测试期望的规范化路径：`fs::canonicalize` 在 Windows 上返回 `\\?\C:\...`，
/// 而 API 会去掉扩展长度前缀后再写进配置。
pub fn expected_canonical(path: &Path) -> String {
    let canonical = path.canonicalize().unwrap();
    let text = canonical.to_string_lossy().to_string();
    text.strip_prefix(r"\\?\")
        .map(|rest| rest.to_string())
        .unwrap_or(text)
}

/// 标准查询参数百分号编码（保留 RFC 3986 的 unreserved 字符）。
/// 手工拼请求 URI 时必须用它，否则空格 / `#` / `%` 会让 URI 失效或被截断。
pub fn encode_query_value(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

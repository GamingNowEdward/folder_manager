use std::collections::BTreeMap;

use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Path, Query, Request, State};
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{middleware, Json, Router};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::application::config_service::ConfigService;
use crate::domain::folder::{self, Folder, FolderInput};
use crate::domain::project::Project;
use crate::infrastructure::http::cors;
use crate::infrastructure::http::error::{ApiError, ApiResult, API_NAME, API_VERSION};
use crate::infrastructure::paths;

/// 请求体上限：本地接口不需要大 payload，同时避免被塞入超大 JSON。
const MAX_BODY_BYTES: usize = 64 * 1024;

#[derive(Clone)]
pub struct ApiState {
    service: std::sync::Arc<ConfigService>,
    version: &'static str,
}

impl ApiState {
    pub fn new(service: std::sync::Arc<ConfigService>, version: &'static str) -> Self {
        Self { service, version }
    }
}

/// 组装 API 路由（测试直接对它发起请求，`server` 负责把它绑定到 127.0.0.1）。
pub fn router(state: ApiState) -> Router {
    let allowed_origins = cors::allowed_origins_from_env();
    let api = Router::new()
        .route("/", get(capabilities))
        .route("/health", get(health))
        .route("/projects", get(list_projects).post(create_project))
        .route(
            "/projects/{project_id}",
            get(get_project).put(update_project).delete(delete_project),
        )
        .route(
            "/projects/{project_id}/folders",
            post(add_folder).delete(delete_folder_from_query),
        )
        .route(
            "/projects/{project_id}/folders/{folder_id}",
            delete(delete_folder_by_id),
        )
        .route("/folders", get(search_folders))
        .route("/folders/tree", get(folder_tree))
        .layer(middleware::from_fn(log_api_error))
        .layer(axum::extract::DefaultBodyLimit::max(MAX_BODY_BYTES));

    Router::new()
        .nest("/api", api)
        .fallback(not_found)
        .layer(cors::layer(&allowed_origins))
        .with_state(state)
}

async fn not_found(uri: Uri) -> Response {
    ApiError::not_found("NOT_FOUND", format!("接口不存在: {}", uri.path())).into_response()
}

/// 统一错误日志：只记录出错的 method / path / status，
/// 不记录 header、token 与请求体，避免敏感信息进日志。
async fn log_api_error(request: Request, next: middleware::Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let response = next.run(request).await;
    let status = response.status();
    if status.is_server_error() {
        eprintln!("[folder-manager-api] error {method} {path} -> {status}");
    } else if status.is_client_error() {
        eprintln!("[folder-manager-api] warn {method} {path} -> {status}");
    }
    response
}

/// 与 `axum::Json` 相同，但拒绝时也返回统一的错误结构。
pub struct ApiJson<T>(pub T);

impl<S, T> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(request, state).await {
            Ok(Json(value)) => Ok(Self(value)),
            Err(rejection) => Err(map_json_rejection(rejection)),
        }
    }
}

fn map_json_rejection(rejection: JsonRejection) -> ApiError {
    match rejection {
        JsonRejection::MissingJsonContentType(_) => ApiError::bad_request(
            "INVALID_CONTENT_TYPE",
            "请求需要 Content-Type: application/json",
        ),
        JsonRejection::JsonDataError(error) => {
            ApiError::bad_request("INVALID_ARGUMENT", format!("请求体字段不合法: {error}"))
        }
        JsonRejection::JsonSyntaxError(_) => {
            ApiError::bad_request("MALFORMED_JSON", "请求体不是合法 JSON")
        }
        other => ApiError::bad_request("INVALID_ARGUMENT", other.body_text()),
    }
}

// ---------------------------------------------------------------- 视图模型

#[derive(Debug, Serialize)]
pub struct FolderView {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectView {
    pub id: String,
    pub name: String,
    pub folders: Vec<FolderView>,
}

#[derive(Debug, Serialize)]
pub struct ProjectsResponse {
    pub current_project_id: String,
    pub revision: u64,
    pub projects: Vec<ProjectView>,
}

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub project: ProjectView,
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub ok: bool,
    pub removed: RemovedView,
}

#[derive(Debug, Serialize)]
pub struct RemovedView {
    pub kind: &'static str,
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub version: &'static str,
    pub api_version: &'static str,
    pub revision: u64,
}

#[derive(Debug, Serialize)]
pub struct CapabilitiesResponse {
    pub name: &'static str,
    pub version: &'static str,
    pub api_version: &'static str,
    pub base_url: &'static str,
    pub capabilities: [&'static str; 6],
}

#[derive(Debug, Serialize)]
pub struct FolderSearchResponse {
    pub query: String,
    pub matches: Vec<FolderMatchView>,
}

#[derive(Debug, Serialize)]
pub struct FolderMatchView {
    pub project_id: String,
    pub project_name: String,
    pub folder: FolderView,
}

#[derive(Debug, Serialize)]
pub struct FolderTreeResponse {
    pub current_project_id: String,
    pub revision: u64,
    pub projects: Vec<ProjectView>,
}

impl From<&Folder> for FolderView {
    fn from(folder: &Folder) -> Self {
        Self {
            id: folder.id.clone(),
            name: folder.name.clone(),
            path: folder.path.clone(),
        }
    }
}

impl From<&Project> for ProjectView {
    fn from(project: &Project) -> Self {
        Self {
            id: project.id.clone(),
            name: project.name.clone(),
            folders: project.folders.iter().map(FolderView::from).collect(),
        }
    }
}

// ---------------------------------------------------------------- 请求模型

#[derive(Debug, Deserialize)]
pub struct ProjectNameRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct AddFolderRequest {
    pub path: String,
    #[serde(default)]
    pub name: Option<String>,
}

// ---------------------------------------------------------------- 处理器

async fn capabilities(State(state): State<ApiState>) -> Json<CapabilitiesResponse> {
    Json(CapabilitiesResponse {
        name: API_NAME,
        version: state.version,
        api_version: API_VERSION,
        base_url: "/api",
        capabilities: [
            "list_projects",
            "create_project",
            "delete_project",
            "add_folder",
            "remove_folder",
            "search_folder",
        ],
    })
}

async fn health(State(state): State<ApiState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        version: state.version,
        api_version: API_VERSION,
        revision: state.service.revision(),
    })
}

async fn list_projects(State(state): State<ApiState>) -> ApiResult<Json<ProjectsResponse>> {
    let workspace = state.service.load().map_err(internal)?;
    Ok(Json(ProjectsResponse {
        current_project_id: workspace.current_project_id,
        revision: state.service.revision(),
        projects: workspace.projects.iter().map(ProjectView::from).collect(),
    }))
}

async fn get_project(
    State(state): State<ApiState>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<ProjectResponse>> {
    validate_id("project_id", &project_id)?;
    let project = state.service.get_project(&project_id).map_err(reject)?;
    Ok(Json(ProjectResponse {
        project: ProjectView::from(&project),
    }))
}

async fn create_project(
    State(state): State<ApiState>,
    ApiJson(request): ApiJson<ProjectNameRequest>,
) -> ApiResult<Response> {
    let name = request.name.trim();
    validate_name("name", name)?;
    let project = state.service.create_project(name).map_err(reject)?;
    Ok((
        StatusCode::CREATED,
        Json(ProjectResponse {
            project: ProjectView::from(&project),
        }),
    )
        .into_response())
}

async fn update_project(
    State(state): State<ApiState>,
    Path(project_id): Path<String>,
    ApiJson(request): ApiJson<ProjectNameRequest>,
) -> ApiResult<Json<ProjectResponse>> {
    validate_id("project_id", &project_id)?;
    let name = request.name.trim();
    validate_name("name", name)?;
    let project = state
        .service
        .update_project(&project_id, name)
        .map_err(reject)?;
    Ok(Json(ProjectResponse {
        project: ProjectView::from(&project),
    }))
}

async fn delete_project(
    State(state): State<ApiState>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<DeleteResponse>> {
    validate_id("project_id", &project_id)?;
    let project = state.service.delete_project(&project_id).map_err(reject)?;
    Ok(Json(DeleteResponse {
        ok: true,
        removed: RemovedView {
            kind: "project",
            id: project.id,
            name: project.name,
            path: None,
        },
    }))
}

async fn add_folder(
    State(state): State<ApiState>,
    Path(project_id): Path<String>,
    ApiJson(request): ApiJson<AddFolderRequest>,
) -> ApiResult<Response> {
    validate_id("project_id", &project_id)?;

    // 1) path 必须存在；2) 必须是目录；3) 规范化（解析 . / .. 与符号链接）
    let resolved = paths::resolve_existing_directory(&request.path).map_err(reject)?;
    let path = resolved.to_string_lossy().to_string();
    // 4) 目录名作为默认 name
    let name = match request.name.as_deref().map(str::trim) {
        Some(provided) if !provided.is_empty() => provided.to_string(),
        _ => paths::directory_name(&resolved).ok_or_else(|| {
            ApiError::bad_request("INVALID_PATH", format!("无法从路径推断名称: {path}"))
        })?,
    };

    // 5) 幂等：同一项目内相同 path 不重复创建
    let outcome = state
        .service
        .add_folder(&project_id, FolderInput::new(name, path))
        .map_err(reject)?;

    let (status, folder) = if outcome.created() {
        (StatusCode::CREATED, outcome.into_folder())
    } else {
        (StatusCode::OK, outcome.folder().clone())
    };
    Ok((status, Json(FolderView::from(&folder))).into_response())
}

async fn delete_folder_by_id(
    State(state): State<ApiState>,
    Path((project_id, folder_id)): Path<(String, String)>,
) -> ApiResult<Json<DeleteResponse>> {
    validate_id("project_id", &project_id)?;
    validate_id("folder_id", &folder_id)?;
    let folder = state
        .service
        .remove_folder_by_id(&project_id, &folder_id)
        .map_err(reject)?;
    Ok(Json(DeleteResponse {
        ok: true,
        removed: removed_folder(folder),
    }))
}

/// `DELETE /api/projects/{project_id}/folders?path=...`
async fn delete_folder_from_query(
    State(state): State<ApiState>,
    Path(project_id): Path<String>,
    Query(query): Query<BTreeMap<String, String>>,
) -> ApiResult<Json<DeleteResponse>> {
    validate_id("project_id", &project_id)?;
    let path = query
        .get("path")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ApiError::bad_request(
                "INVALID_ARGUMENT",
                "缺少 path 查询参数；也可改用 DELETE /api/projects/{project_id}/folders/{folder_id}",
            )
        })?;
    // 只做路径形状校验：不会因为目录被移动/删除而拒绝移除配置记录
    let path = paths::normalize_stored_path(path).map_err(reject)?;
    let folder = state
        .service
        .remove_folder_by_path(&project_id, &path)
        .map_err(reject)?;
    Ok(Json(DeleteResponse {
        ok: true,
        removed: removed_folder(folder),
    }))
}

fn removed_folder(folder: Folder) -> RemovedView {
    RemovedView {
        kind: "folder",
        id: folder.id,
        name: folder.name,
        path: Some(folder.path),
    }
}

async fn search_folders(
    State(state): State<ApiState>,
    Query(query): Query<BTreeMap<String, String>>,
) -> ApiResult<Json<FolderSearchResponse>> {
    let needle = query
        .get("path")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    let workspace = state.service.load().map_err(internal)?;
    let normalized_needle = folder::normalize_path_for_comparison(&needle);

    let mut matches = Vec::new();
    for project in &workspace.projects {
        for candidate in &project.folders {
            if needle.is_empty()
                || folder::normalize_path_for_comparison(&candidate.path)
                    .contains(&normalized_needle)
            {
                matches.push(FolderMatchView {
                    project_id: project.id.clone(),
                    project_name: project.name.clone(),
                    folder: FolderView::from(candidate),
                });
            }
        }
    }

    Ok(Json(FolderSearchResponse {
        query: needle,
        matches,
    }))
}

async fn folder_tree(State(state): State<ApiState>) -> ApiResult<Json<FolderTreeResponse>> {
    let workspace = state.service.load().map_err(internal)?;
    Ok(Json(FolderTreeResponse {
        current_project_id: workspace.current_project_id,
        revision: state.service.revision(),
        projects: workspace.projects.iter().map(ProjectView::from).collect(),
    }))
}

// ---------------------------------------------------------------- 校验与映射

/// 领域错误 → HTTP 错误。业务逻辑（domain / application）不感知 HTTP。
fn reject(error: crate::error::AppError) -> ApiError {
    ApiError::from(error)
}

/// 读取/持久化失败属于服务端问题；日志里要能定位到具体原因。
fn internal(error: crate::error::AppError) -> ApiError {
    eprintln!("[folder-manager-api] error 读取配置失败: {error}");
    ApiError::internal(error.to_string())
}

fn validate_name(field: &str, value: &str) -> ApiResult<()> {
    if value.is_empty() {
        return Err(ApiError::bad_request(
            "INVALID_ARGUMENT",
            format!("{field} 不能为空"),
        ));
    }
    if value.chars().count() > 200 {
        return Err(ApiError::bad_request(
            "INVALID_ARGUMENT",
            format!("{field} 过长（上限 200 字符）"),
        ));
    }
    Ok(())
}

/// project_id / folder_id 严格校验：非空、限长、仅允许 id 字符集。
/// 即使 config.json 被手工改坏，也不会把任意字符串带进业务逻辑。
pub fn validate_id(field: &str, value: &str) -> ApiResult<()> {
    if value.is_empty() || value.len() > 128 {
        return Err(ApiError::bad_request(
            "INVALID_ID",
            format!("{field} 非法：长度必须是 1..=128"),
        ));
    }
    if !value
        .chars()
        .all(|it| it.is_ascii_alphanumeric() || it == '-' || it == '_')
    {
        return Err(ApiError::bad_request(
            "INVALID_ID",
            format!("{field} 非法：只允许字母、数字、'-' 与 '_'"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::http::testing::{
        assert_directory_survives, encode_query_value, expected_canonical, Fixture,
    };
    use axum::body::Body;
    use axum::http::{Method, Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn read_workspace(path: &std::path::Path) -> crate::domain::config::Workspace {
        crate::infrastructure::persistence::json_store::read_workspace(path)
            .unwrap()
            .unwrap()
    }

    #[tokio::test]
    async fn health_reports_ok_and_version() {
        let fixture = Fixture::new();
        let (status, body) = fixture.get("/api/health").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["ok"], true);
        assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(body["api_version"], "1");
        assert!(body["revision"].as_u64().is_some());
    }

    #[tokio::test]
    async fn capabilities_are_self_describing() {
        let fixture = Fixture::new();
        let (status, body) = fixture.get("/api").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["name"], API_NAME);
        assert_eq!(body["api_version"], "1");
        let capabilities = body["capabilities"].as_array().unwrap();
        for expected in [
            "list_projects",
            "create_project",
            "delete_project",
            "add_folder",
            "remove_folder",
            "search_folder",
        ] {
            assert!(
                capabilities.iter().any(|item| item == expected),
                "缺少 capability: {expected}"
            );
        }
    }

    #[tokio::test]
    async fn lists_projects_from_existing_config() {
        let fixture = Fixture::new();
        let (status, body) = fixture.get("/api/projects").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["current_project_id"], fixture.current_project_id());
        let projects = body["projects"].as_array().unwrap();
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0]["name"], "工作");
        assert_eq!(projects[0]["folders"][0]["name"], "src");
        assert!(projects[0]["folders"][0]["id"].is_string());
    }

    #[tokio::test]
    async fn gets_single_project_and_reports_missing() {
        let fixture = Fixture::new();
        let project_id = fixture.workspace.workspace.projects[1].id.clone();

        let (status, body) = fixture.get(&format!("/api/projects/{project_id}")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["project"]["name"], "私人");

        let (status, body) = fixture.get("/api/projects/does-not-exist").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "PROJECT_NOT_FOUND");
        assert!(body["error"]["message"]
            .as_str()
            .is_some_and(|it| it.contains("does-not-exist")));
    }

    #[tokio::test]
    async fn creates_project_and_persists_it() {
        let fixture = Fixture::new();

        let (status, body) = fixture
            .post_json("/api/projects", serde_json::json!({ "name": "Agent 项目" }))
            .await;

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body["project"]["name"], "Agent 项目");
        let created_id = body["project"]["id"].as_str().unwrap().to_string();
        assert!(!created_id.is_empty());

        let on_disk = read_workspace(&fixture.config_path());
        assert!(on_disk.projects.iter().any(|it| it.id == created_id));

        let (status, body) = fixture.get("/api/projects").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["projects"].as_array().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn rejects_duplicate_project_name_with_conflict() {
        let fixture = Fixture::new();

        let (status, body) = fixture
            .post_json("/api/projects", serde_json::json!({ "name": "工作" }))
            .await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["error"]["code"], "PROJECT_NAME_TAKEN");
    }

    #[tokio::test]
    async fn rejects_blank_project_name() {
        let fixture = Fixture::new();

        let (status, body) = fixture
            .post_json("/api/projects", serde_json::json!({ "name": "   " }))
            .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "INVALID_ARGUMENT");
    }

    #[tokio::test]
    async fn renames_project() {
        let fixture = Fixture::new();
        let project_id = fixture.current_project_id().to_string();

        let (status, body) = fixture
            .put_json(
                &format!("/api/projects/{project_id}"),
                serde_json::json!({ "name": "事业" }),
            )
            .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["project"]["name"], "事业");

        let on_disk = read_workspace(&fixture.config_path());
        assert_eq!(on_disk.projects[0].name, "事业");
    }

    #[tokio::test]
    async fn deletes_project_without_touching_directories() {
        let fixture = Fixture::new();
        let project_id = fixture.current_project_id().to_string();
        let folder_path =
            std::path::PathBuf::from(&fixture.workspace.workspace.projects[0].folders[0].path);

        let (status, body) = fixture.delete(&format!("/api/projects/{project_id}")).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["ok"], true);
        assert_eq!(body["removed"]["kind"], "project");
        assert_directory_survives(&folder_path);

        let on_disk = read_workspace(&fixture.config_path());
        assert_eq!(on_disk.projects.len(), 1);
        assert_ne!(on_disk.current_project_id, project_id);
    }

    #[tokio::test]
    async fn adds_folder_with_directory_name_as_default() {
        let fixture = Fixture::new();
        let (project_id, dir) = fixture.new_directory("agent-target");

        let (status, body) = fixture
            .post_json(
                &format!("/api/projects/{project_id}/folders"),
                serde_json::json!({ "path": dir.to_string_lossy() }),
            )
            .await;

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body["name"], "agent-target");
        assert_eq!(body["path"], expected_canonical(&dir));
        // 写进配置的路径不能带 Windows 扩展长度前缀（\\?\）
        assert!(!body["path"].as_str().unwrap().starts_with(r"\\?\"));
        assert!(body["id"].as_str().is_some_and(|it| !it.is_empty()));

        let on_disk = read_workspace(&fixture.config_path());
        let project = on_disk
            .projects
            .iter()
            .find(|it| it.id == project_id)
            .unwrap();
        assert_eq!(project.folders.len(), 2);
        assert_directory_survives(&dir);
    }

    #[tokio::test]
    async fn add_folder_accepts_trailing_separator_and_custom_name() {
        let fixture = Fixture::new();
        let (project_id, dir) = fixture.new_directory("custom-name");

        let (status, body) = fixture
            .post_json(
                &format!("/api/projects/{project_id}/folders"),
                serde_json::json!({
                    "path": format!("{}\\", dir.to_string_lossy()),
                    "name": "Agent Picked"
                }),
            )
            .await;

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body["name"], "Agent Picked");
    }

    #[tokio::test]
    async fn duplicate_folder_path_is_idempotent() {
        let fixture = Fixture::new();
        let (project_id, dir) = fixture.new_directory("dup-target");
        let url = format!("/api/projects/{project_id}/folders");
        let payload = serde_json::json!({ "path": dir.to_string_lossy() });

        let (first_status, first_body) = fixture.post_json(&url, payload).await;
        assert_eq!(first_status, StatusCode::CREATED);

        // 大小写与结尾分隔符不同，也必须识别为同一路径
        let (second_status, second_body) = fixture
            .post_json(
                &url,
                serde_json::json!({ "path": format!("{}\\", dir.to_string_lossy().to_uppercase()) }),
            )
            .await;

        assert_eq!(second_status, StatusCode::OK);
        assert_eq!(second_body["id"], first_body["id"]);

        let on_disk = read_workspace(&fixture.config_path());
        let project = on_disk
            .projects
            .iter()
            .find(|it| it.id == project_id)
            .unwrap();
        // 只新增了一条记录（幂等命中不会重复写入）
        let names: Vec<&str> = project.folders.iter().map(|it| it.name.as_str()).collect();
        assert_eq!(names, vec!["src", "dup-target"]);
    }

    #[tokio::test]
    async fn add_folder_rejects_missing_project() {
        let fixture = Fixture::new();
        let dir = fixture.temp.path().join("anydir");
        std::fs::create_dir(&dir).unwrap();

        let (status, body) = fixture
            .post_json(
                "/api/projects/missing-project/folders",
                serde_json::json!({ "path": dir.to_string_lossy() }),
            )
            .await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "PROJECT_NOT_FOUND");
    }

    #[tokio::test]
    async fn add_folder_rejects_missing_path() {
        let fixture = Fixture::new();
        let project_id = fixture.current_project_id().to_string();
        let missing = fixture.temp.path().join("nope");

        let (status, body) = fixture
            .post_json(
                &format!("/api/projects/{project_id}/folders"),
                serde_json::json!({ "path": missing.to_string_lossy() }),
            )
            .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "PATH_NOT_FOUND");
    }

    #[tokio::test]
    async fn add_folder_rejects_path_that_is_not_a_directory() {
        let fixture = Fixture::new();
        let project_id = fixture.current_project_id().to_string();
        let file = fixture.temp.path().join("file.txt");
        std::fs::write(&file, "x").unwrap();

        let (status, body) = fixture
            .post_json(
                &format!("/api/projects/{project_id}/folders"),
                serde_json::json!({ "path": file.to_string_lossy() }),
            )
            .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "PATH_NOT_DIRECTORY");
    }

    #[tokio::test]
    async fn add_folder_rejects_relative_path() {
        let fixture = Fixture::new();
        let project_id = fixture.current_project_id().to_string();

        let (status, body) = fixture
            .post_json(
                &format!("/api/projects/{project_id}/folders"),
                serde_json::json!({ "path": "..\\..\\windows\\system32" }),
            )
            .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "INVALID_PATH");
    }

    #[tokio::test]
    async fn add_folder_resolves_traversal_instead_of_escaping() {
        let fixture = Fixture::new();
        let (project_id, dir) = fixture.new_directory("inner");
        let traversing = format!("{}\\..", dir.to_string_lossy());

        let (status, body) = fixture
            .post_json(
                &format!("/api/projects/{project_id}/folders"),
                serde_json::json!({ "path": traversing }),
            )
            .await;

        assert_eq!(status, StatusCode::CREATED);
        let stored = body["path"].as_str().unwrap();
        assert!(!stored.contains(".."));
        let resolved = expected_canonical(fixture.temp.path());
        assert_eq!(stored, resolved);
        assert!(!stored.starts_with(r"\\?\"));
    }

    #[tokio::test]
    async fn add_folder_rejects_duplicate_name_in_same_project() {
        let fixture = Fixture::new();
        let project_id = fixture.current_project_id().to_string();
        let (_, dir) = fixture.new_directory("other");

        let (status, body) = fixture
            .post_json(
                &format!("/api/projects/{project_id}/folders"),
                serde_json::json!({ "path": dir.to_string_lossy(), "name": "src" }),
            )
            .await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["error"]["code"], "FOLDER_NAME_TAKEN");
    }

    #[tokio::test]
    async fn deletes_folder_by_id_without_deleting_directory() {
        let fixture = Fixture::new();
        let project = &fixture.workspace.workspace.projects[0];
        let folder_id = project.folders[0].id.clone();
        let folder_path = std::path::PathBuf::from(&project.folders[0].path);
        let project_id = project.id.clone();

        let (status, body) = fixture
            .delete(&format!("/api/projects/{project_id}/folders/{folder_id}"))
            .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["ok"], true);
        assert_eq!(body["removed"]["id"], folder_id);
        assert_directory_survives(&folder_path);

        let on_disk = read_workspace(&fixture.config_path());
        let stored = on_disk
            .projects
            .iter()
            .find(|it| it.id == project_id)
            .unwrap();
        assert!(stored.folders.iter().all(|it| it.id != folder_id));
    }

    #[tokio::test]
    async fn deletes_folder_by_path_query() {
        let fixture = Fixture::new();
        let project = &fixture.workspace.workspace.projects[0];
        let folder_id = project.folders[0].id.clone();
        let folder_path = std::path::PathBuf::from(&project.folders[0].path);
        let project_id = project.id.clone();

        // axum 会自行解码查询参数，所以这里用标准百分号编码把路径放进 URI
        let encoded = encode_query_value(&project.folders[0].path);
        let (status, body) = fixture
            .delete(&format!(
                "/api/projects/{project_id}/folders?path={encoded}"
            ))
            .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["removed"]["id"], folder_id);
        assert_directory_survives(&folder_path);
    }

    #[tokio::test]
    async fn delete_folder_reports_missing_folder_and_project() {
        let fixture = Fixture::new();
        let project_id = fixture.current_project_id().to_string();

        let (status, body) = fixture
            .delete(&format!("/api/projects/{project_id}/folders/nope"))
            .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "FOLDER_NOT_FOUND");

        let (status, body) = fixture.delete("/api/projects/missing/folders/nope").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "PROJECT_NOT_FOUND");

        let (status, body) = fixture
            .delete(&format!("/api/projects/{project_id}/folders"))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "INVALID_ARGUMENT");

        let (status, body) = fixture
            .delete(&format!(
                "/api/projects/{project_id}/folders?path=C%3A%5Cnot%5Cthere"
            ))
            .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "FOLDER_NOT_FOUND");
    }

    #[tokio::test]
    async fn rejects_invalid_project_and_folder_ids() {
        let fixture = Fixture::new();

        for bad in ["..", "a%20b", "x".repeat(129).as_str()] {
            let (status, body) = fixture.get(&format!("/api/projects/{bad}")).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "id = {bad}");
            assert_eq!(body["error"]["code"], "INVALID_ID", "id = {bad}");
        }

        let project_id = fixture.current_project_id().to_string();
        let (status, body) = fixture
            .delete(&format!("/api/projects/{project_id}/folders/..%2F.."))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "INVALID_ID");
    }

    #[tokio::test]
    async fn searches_folders_by_path() {
        let fixture = Fixture::new();

        let (status, body) = fixture.get("/api/folders?path=work").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["matches"].as_array().unwrap().len(), 1);
        assert_eq!(body["matches"][0]["folder"]["name"], "src");
        assert_eq!(body["matches"][0]["project_name"], "工作");

        let (status, body) = fixture.get("/api/folders").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["matches"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn returns_folder_tree() {
        let fixture = Fixture::new();

        let (status, body) = fixture.get("/api/folders/tree").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["projects"].as_array().unwrap().len(), 2);
        assert_eq!(body["current_project_id"], fixture.current_project_id());
    }

    #[tokio::test]
    async fn unknown_route_uses_the_shared_error_shape() {
        let fixture = Fixture::new();

        let (status, body) = fixture.get("/api/nope").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "NOT_FOUND");
    }

    #[tokio::test]
    async fn malformed_json_uses_the_shared_error_shape() {
        let fixture = Fixture::new();
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/projects")
            .header("content-type", "application/json")
            .body(Body::from("{ not json"))
            .unwrap();

        let response = fixture.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "MALFORMED_JSON");
    }

    #[tokio::test]
    async fn missing_content_type_uses_the_shared_error_shape() {
        let fixture = Fixture::new();
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/projects")
            .body(Body::from(r#"{"name":"x"}"#))
            .unwrap();

        let response = fixture.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "INVALID_CONTENT_TYPE");
    }

    #[tokio::test]
    async fn api_mutations_are_visible_through_the_shared_service() {
        let fixture = Fixture::new();

        let (status, body) = fixture
            .post_json("/api/projects", serde_json::json!({ "name": "共享状态" }))
            .await;
        assert_eq!(status, StatusCode::CREATED);
        let project_id = body["project"]["id"].as_str().unwrap().to_string();

        // Tauri 命令与 HTTP API 共用同一个 ConfigService 实例
        let shared = fixture.service.load().unwrap();
        assert!(shared.projects.iter().any(|it| it.id == project_id));
    }

    #[tokio::test]
    async fn rejects_invalid_json_field_types() {
        let fixture = Fixture::new();

        let (status, body) = fixture
            .post_json("/api/projects", serde_json::json!({ "name": 42 }))
            .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "INVALID_ARGUMENT");
    }
}

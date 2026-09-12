//! API contract tests：启动**真实 HTTP server**，用真实 HTTP 请求 + JSON 解码断言，
//! 防止 handler / router / service 之间出现接口漂移。
//!
//! 与 `api.rs` 里的单元测试互补：那些用 `Router::oneshot` 直接驱动路由，
//! 这里覆盖 listener、真实 socket、header、Content-Type 与状态码。

use std::net::SocketAddr;
use std::path::PathBuf;

use reqwest::StatusCode;

use crate::infrastructure::http::error::{API_NAME, API_VERSION};
use crate::infrastructure::http::server::{self, ServerHandle};
use crate::infrastructure::http::testing::Fixture;

struct LiveApi {
    fixture: Fixture,
    #[allow(dead_code)]
    handle: ServerHandle,
    base_url: String,
    client: reqwest::Client,
}

impl LiveApi {
    fn start() -> Self {
        let fixture = Fixture::new();
        // 端口 0：由系统分配，避免与其它用例或用户环境冲突
        let handle = server::start(fixture.service.clone(), env!("CARGO_PKG_VERSION"))
            .unwrap_or_else(|error| panic!("API 启动失败: {error}"));
        let addr: SocketAddr = handle.address();
        let base_url = format!("http://{addr}/api");
        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("构建 http client 失败");

        Self {
            fixture,
            handle,
            base_url,
            client,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    async fn get(&self, path: &str) -> (StatusCode, serde_json::Value) {
        let response = self.client.get(self.url(path)).send().await.unwrap();
        let status = response.status();
        let body = response.json().await.unwrap_or(serde_json::Value::Null);
        (status, body)
    }

    async fn post(
        &self,
        path: &str,
        payload: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let response = self
            .client
            .post(self.url(path))
            .json(&payload)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let body = response.json().await.unwrap_or(serde_json::Value::Null);
        (status, body)
    }

    async fn put(&self, path: &str, payload: serde_json::Value) -> (StatusCode, serde_json::Value) {
        let response = self
            .client
            .put(self.url(path))
            .json(&payload)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let body = response.json().await.unwrap_or(serde_json::Value::Null);
        (status, body)
    }

    async fn delete(&self, path: &str) -> (StatusCode, serde_json::Value) {
        let response = self.client.delete(self.url(path)).send().await.unwrap();
        let status = response.status();
        let body = response.json().await.unwrap_or(serde_json::Value::Null);
        (status, body)
    }
}

/// 完整跑一遍 Agent 的典型调用序列。
#[tokio::test]
async fn agent_workflow_over_real_http() {
    let api = LiveApi::start();

    // GET /api —— 能力自描述
    let (status, body) = api.get("").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], API_NAME);
    assert_eq!(body["api_version"], API_VERSION);
    assert_eq!(body["server_version"], env!("CARGO_PKG_VERSION"));
    assert!(body["base_url"]
        .as_str()
        .is_some_and(|it| it == api.base_url.as_str()));
    assert_eq!(body["security"]["deletes_real_folders"], false);
    let capability_ids: Vec<&str> = body["capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    assert!(capability_ids.contains(&"add_folder"));

    // GET /api/health
    let (status, body) = api.get("/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
    assert!(body["revision"].as_u64().is_some());

    // POST /api/projects
    let (status, body) = api
        .post(
            "/projects",
            serde_json::json!({ "name": "Agent HTTP 项目" }),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let project_id = body["project"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["project"]["folders"].as_array().unwrap().len(), 0);

    // 创建后能在列表里看到
    let (status, body) = api.get("/projects").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"] == project_id));

    // GET /api/projects/{id}
    let (status, body) = api.get(&format!("/projects/{project_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["project"]["name"], "Agent HTTP 项目");

    // POST /api/projects/{id}/folders
    let dir = api.fixture.temp.path().join("http-target");
    std::fs::create_dir(&dir).unwrap();
    let (status, folder) = api
        .post(
            &format!("/projects/{project_id}/folders"),
            serde_json::json!({ "path": dir.to_string_lossy() }),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let folder_id = folder["id"].as_str().unwrap().to_string();
    assert_eq!(folder["name"], "http-target");

    // 幂等：同一路径再发一次返回 200 与同一 id
    let (status, again) = api
        .post(
            &format!("/projects/{project_id}/folders"),
            serde_json::json!({ "path": dir.to_string_lossy().to_uppercase() }),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(again["id"], folder_id);

    // GET /api/folders?path=...
    let encoded =
        crate::infrastructure::http::testing::encode_query_value(dir.to_string_lossy().as_ref());
    let (status, body) = api.get(&format!("/folders?path={encoded}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["matches"].as_array().unwrap().len(), 1);
    assert_eq!(body["matches"][0]["folder"]["id"], folder_id);

    // GET /api/folders/tree
    let (status, body) = api.get("/folders/tree").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["projects"].is_array());

    // DELETE /api/projects/{id}/folders/{folder_id}
    let (status, body) = api
        .delete(&format!("/projects/{project_id}/folders/{folder_id}"))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
    assert_eq!(body["removed"]["id"], folder_id);
    // 真实目录必须还在
    assert!(dir.is_dir(), "API 不能删除真实目录");

    // DELETE /api/projects/{id}/folders?path=...
    let (status, folder) = api
        .post(
            &format!("/projects/{project_id}/folders"),
            serde_json::json!({ "path": dir.to_string_lossy() }),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let stored_path = folder["path"].as_str().unwrap().to_string();
    let encoded = crate::infrastructure::http::testing::encode_query_value(&stored_path);
    let (status, body) = api
        .delete(&format!("/projects/{project_id}/folders?path={encoded}"))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["removed"]["id"], folder["id"]);
    assert!(dir.is_dir(), "按路径删除同样不能删除真实目录");

    // PUT /api/projects/{id}
    let (status, body) = api
        .put(
            &format!("/projects/{project_id}"),
            serde_json::json!({ "name": "改名后的项目" }),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["project"]["name"], "改名后的项目");

    // DELETE /api/projects/{id}
    let (status, body) = api.delete(&format!("/projects/{project_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["removed"]["kind"], "project");

    // 删除后确实不存在了
    let (status, body) = api.get(&format!("/projects/{project_id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "PROJECT_NOT_FOUND");
}

/// 真实 HTTP 下也必须写进 config.json（而不是只改内存）。
#[tokio::test]
async fn http_mutations_are_persisted_to_config_file() {
    let api = LiveApi::start();

    let (_, body) = api
        .post("/projects", serde_json::json!({ "name": "落盘项目" }))
        .await;
    let project_id = body["project"]["id"].as_str().unwrap().to_string();

    let on_disk =
        crate::infrastructure::persistence::json_store::read_workspace(&api.fixture.config_path())
            .unwrap()
            .unwrap();

    assert!(on_disk.projects.iter().any(|it| it.id == project_id));
}

/// 错误响应在真实 HTTP 下也必须是统一结构。
#[tokio::test]
async fn http_errors_keep_the_unified_shape() {
    let api = LiveApi::start();
    let project_id = api.fixture.current_project_id().to_string();

    let checks: Vec<(StatusCode, serde_json::Value, &str)> = vec![
        {
            let (status, body) = api.get("/projects/missing-project").await;
            (status, body, "PROJECT_NOT_FOUND")
        },
        {
            let (status, body) = api.get("/nope").await;
            (status, body, "NOT_FOUND")
        },
        {
            let (status, body) = api
                .delete(&format!("/projects/{project_id}/folders/missing"))
                .await;
            (status, body, "FOLDER_NOT_FOUND")
        },
        {
            let (status, body) = api
                .post(
                    &format!("/projects/{project_id}/folders"),
                    serde_json::json!({ "path": "relative\\dir" }),
                )
                .await;
            (status, body, "INVALID_PATH")
        },
        {
            let (status, body) = api
                .post("/projects", serde_json::json!({ "name": "工作" }))
                .await;
            (status, body, "PROJECT_NAME_TAKEN")
        },
    ];

    for (status, body, expected_code) in checks {
        assert!(
            body.get("error").is_some_and(|it| it.is_object()),
            "错误结构必须是 {{error:{{code,message}}}}: {body}"
        );
        assert_eq!(body["error"]["code"], expected_code);
        assert!(body["error"]["message"]
            .as_str()
            .is_some_and(|it| !it.is_empty()));
        let expected_status = match expected_code {
            "PROJECT_NOT_FOUND" | "FOLDER_NOT_FOUND" | "NOT_FOUND" => StatusCode::NOT_FOUND,
            "PROJECT_NAME_TAKEN" => StatusCode::CONFLICT,
            _ => StatusCode::BAD_REQUEST,
        };
        assert_eq!(status, expected_status, "code = {expected_code}");
    }
}

/// API 不是文件系统接口：不存在任何读文件 / 写文件 / 列目录的 endpoint。
#[tokio::test]
async fn api_exposes_no_filesystem_endpoints() {
    let api = LiveApi::start();

    for path in [
        "/file",
        "/files",
        "/fs",
        "/read-file",
        "/write-file",
        "/directories",
        "/browse",
    ] {
        let (status, body) = api.get(path).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "path = {path}");
        assert_eq!(body["error"]["code"], "NOT_FOUND", "path = {path}");
    }

    // 真实存在的文件路径也不能被"读取"成资源
    let file = api.fixture.temp.path().join("secret.txt");
    std::fs::write(&file, "top secret").unwrap();
    let encoded =
        crate::infrastructure::http::testing::encode_query_value(file.to_string_lossy().as_ref());
    let (status, body) = api.get(&format!("/folders?path={encoded}")).await;
    assert_eq!(status, StatusCode::OK);
    // 只返回已管理的文件夹元数据，绝不返回文件内容
    assert!(body["matches"].as_array().unwrap().is_empty());
    assert!(!body.to_string().contains("top secret"));
}

/// 并发 HTTP 请求不会丢更新。
#[tokio::test]
async fn concurrent_http_adds_do_not_lose_updates() {
    let api = LiveApi::start();
    let (_, body) = api
        .post("/projects", serde_json::json!({ "name": "并发" }))
        .await;
    let project_id = body["project"]["id"].as_str().unwrap().to_string();

    let mut dirs: Vec<PathBuf> = Vec::new();
    for index in 0..8 {
        let dir = api.fixture.temp.path().join(format!("conc-{index}"));
        std::fs::create_dir(&dir).unwrap();
        dirs.push(dir);
    }

    let mut handles = Vec::new();
    for dir in dirs {
        let client = api.client.clone();
        let url = api.url(&format!("/projects/{project_id}/folders"));
        handles.push(tokio::spawn(async move {
            client
                .post(url)
                .json(&serde_json::json!({ "path": dir.to_string_lossy() }))
                .send()
                .await
                .unwrap()
                .status()
        }));
    }
    for handle in handles {
        assert_eq!(handle.await.unwrap(), StatusCode::CREATED);
    }

    let (status, body) = api.get(&format!("/projects/{project_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["project"]["folders"].as_array().unwrap().len(),
        8,
        "并发写入不能丢记录"
    );

    // 磁盘与接口返回一致
    let on_disk =
        crate::infrastructure::persistence::json_store::read_workspace(&api.fixture.config_path())
            .unwrap()
            .unwrap();
    let stored = on_disk
        .projects
        .iter()
        .find(|it| it.id == project_id)
        .unwrap();
    assert_eq!(stored.folders.len(), 8);
}

/// server 句柄关闭后端口必须释放（真实 socket 级别）。
#[tokio::test]
async fn real_server_releases_port_on_shutdown() {
    let fixture = Fixture::new();
    let handle = server::start(fixture.service.clone(), "test").unwrap();
    let addr = handle.address();

    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    assert!(client
        .get(format!("http://{addr}/api/health"))
        .send()
        .await
        .is_ok());

    handle.shutdown();
    drop(handle);

    let released = (0..80).any(|_| {
        if std::net::TcpListener::bind(addr).is_ok() {
            true
        } else {
            std::thread::sleep(std::time::Duration::from_millis(25));
            false
        }
    });
    assert!(released, "shutdown 后端口 {addr} 仍被占用");
}

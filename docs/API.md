# Folder Manager 本地 HTTP API

Folder Manager 在启动时会同时在本机回环地址上开启一个轻量 REST API，
让外部 **AI Agent / Coding Agent / 自动化脚本** 可以读取、添加、移除项目与文件夹。

> **重要**：API 只操作 Folder Manager 自己的项目配置（`config.json`）。
> 它 **永远不会删除、创建或修改 Windows 文件系统中的真实文件夹**。

- 与 UI 完全同步：API 修改后前端会自动刷新（见 [状态同步](#状态同步)）。
- 与 Tauri command 共用同一套 Rust 业务逻辑与持久化（同一个 `ConfigService` 实例）。

---

## 1. 地址与端口

| 项目 | 值 |
| --- | --- |
| 监听地址 | `127.0.0.1`（**只允许本机访问**，不监听 `0.0.0.0`） |
| 默认端口 | `17890` |
| Base URL | `http://127.0.0.1:17890/api` |
| 传输 | HTTP/1.1，JSON（`Content-Type: application/json`） |
| 认证 | 无 Token；安全模型见 [安全边界](#6-安全边界) |

### 修改端口

```powershell
# PowerShell（临时）
$env:FOLDER_MANAGER_API_PORT = "18000"
.\folder-manager.exe

# cmd
set FOLDER_MANAGER_API_PORT=18000
folder-manager.exe
```

- 环境变量：`FOLDER_MANAGER_API_PORT`（必须是 `1..=65535`）。
- **端口冲突处理**：如果端口被占用，应用会自动尝试后续最多 10 个端口并打印实际监听地址；
  如果全部失败，只记录日志 —— **主界面照常使用，只是 API（含 `/api/health`）不可用**。
- 实际监听地址会打印到应用日志：`[folder-manager-api] Folder Manager API 监听 http://127.0.0.1:17890/api`。

### 确认 API 是否可用

```bash
curl http://127.0.0.1:17890/api/health
```

---

## 2. 快速开始

```bash
# 1) 健康检查
curl http://127.0.0.1:17890/api/health

# 2) 读取当前所有项目与文件夹
curl http://127.0.0.1:17890/api/projects

# 3) 创建项目
curl -X POST http://127.0.0.1:17890/api/projects \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Test Project\"}"

# 4) 把 Windows 文件夹加入项目（PROJECT_ID 用上一步返回的 id）
curl -X POST http://127.0.0.1:17890/api/projects/PROJECT_ID/folders \
  -H "Content-Type: application/json" \
  -d "{\"path\":\"C:\\\\work\\\\project\"}"

# 5) 从 Folder Manager 中移除文件夹（不会删除真实目录）
curl -X DELETE \
  http://127.0.0.1:17890/api/projects/PROJECT_ID/folders/FOLDER_ID
```

---

## 3. 数据模型

```jsonc
// Folder
{
  "id": "df0f1a1e-...",        // 稳定 id（uuid），删除时使用
  "name": "src",               // 显示名，默认取目录名
  "path": "C:\\work\\project\\src"
}

// Project
{
  "id": "1c2b3a4d-...",        // 稳定 id（uuid）
  "name": "Project A",
  "folders": [ /* Folder[] */ ]
}
```

- `id` 稳定且唯一，Agent 应该缓存 `id` 而不是依赖列表顺序。
- `path` 为规范化后的绝对路径（统一使用 `\` 分隔符）。
- 保存位置：release 版在 exe 同目录的 `config.json`；debug 版在系统应用数据目录。

---

## 4. Endpoint 一览

| 方法 | 路径 | 说明 | 成功状态码 |
| --- | --- | --- | --- |
| GET | `/api` | API 能力自描述 | 200 |
| GET | `/api/health` | 健康检查 + 当前修订号 | 200 |
| GET | `/api/projects` | 所有项目及其文件夹 | 200 |
| POST | `/api/projects` | 创建项目 | 201 |
| GET | `/api/projects/{project_id}` | 单个项目 | 200 |
| PUT | `/api/projects/{project_id}` | 重命名项目 | 200 |
| DELETE | `/api/projects/{project_id}` | 删除项目（含其文件夹配置） | 200 |
| POST | `/api/projects/{project_id}/folders` | 添加文件夹 | 201 / 200（已存在） |
| DELETE | `/api/projects/{project_id}/folders/{folder_id}` | 按 id 移除文件夹 | 200 |
| DELETE | `/api/projects/{project_id}/folders?path=...` | 按路径移除文件夹 | 200 |
| GET | `/api/folders?path=...` | 按路径搜索文件夹 | 200 |
| GET | `/api/folders/tree` | 完整项目/文件夹结构 | 200 |

---

## 5. 接口详情

### GET `/api`

能力自描述，Agent 可用它自动发现可用操作与安全边界。

```bash
curl http://127.0.0.1:17890/api
```

```json
{
  "name": "Folder Manager API",
  "api_version": "1",
  "server_version": "1.1.0",
  "base_url": "http://127.0.0.1:17890/api",
  "capabilities": [
    {
      "id": "list_projects",
      "method": "GET",
      "path": "/api/projects",
      "description": "列出所有项目及其文件夹"
    },
    {
      "id": "create_project",
      "method": "POST",
      "path": "/api/projects",
      "description": "创建项目"
    },
    {
      "id": "delete_project",
      "method": "DELETE",
      "path": "/api/projects/{project_id}",
      "description": "删除项目记录（不删除真实文件夹）"
    },
    {
      "id": "add_folder",
      "method": "POST",
      "path": "/api/projects/{project_id}/folders",
      "description": "把已存在的绝对路径目录加入项目（相同 path 幂等）"
    },
    {
      "id": "remove_folder",
      "method": "DELETE",
      "path": "/api/projects/{project_id}/folders/{folder_id}",
      "description": "从项目中移除文件夹引用（不删除真实文件夹）"
    },
    {
      "id": "search_folder",
      "method": "GET",
      "path": "/api/folders?path=",
      "description": "按路径子串搜索文件夹"
    }
  ],
  "security": {
    "bind_address": "127.0.0.1 (loopback only)",
    "authentication": "none",
    "deletes_real_folders": false,
    "path_policy": "absolute existing directory path; canonicalized; relative paths rejected",
    "max_body_bytes": 65536
  }
}
```

- `capabilities` 覆盖除 `/api` 自身与 `/api/health` 之外的全部 endpoint（共 6 项）。
- `base_url` 由请求的 `Host` 头生成，端口顺延后依然返回可直接使用的地址。
- `security` 让 Agent 无需读文档就能知道「不能删真实目录、没有认证」。
- 这些字段只增不改，Agent 可以长期依赖。

### GET `/api/health`

```bash
curl http://127.0.0.1:17890/api/health
```

```json
{
  "ok": true,
  "server_version": "1.1.0",
  "api_version": "1",
  "revision": 12
}
```

`revision` 是配置的修订号，每次配置被**成功**修改（UI 或 API）都会 +1；
写盘失败的修改不会递增。Agent 可以低成本轮询 `/api/health`，
只在 `revision` 变化时才重新拉取 `/api/projects`。

### GET `/api/projects`

```bash
curl http://127.0.0.1:17890/api/projects
```

```json
{
  "current_project_id": "1c2b3a4d-...",
  "revision": 12,
  "projects": [
    {
      "id": "1c2b3a4d-...",
      "name": "Project A",
      "folders": [
        { "id": "df0f...", "name": "src", "path": "C:\\work\\project\\src" }
      ]
    }
  ]
}
```

### GET `/api/projects/{project_id}`

```bash
curl http://127.0.0.1:17890/api/projects/1c2b3a4d-...
```

```json
{
  "project": { "id": "1c2b3a4d-...", "name": "Project A", "folders": [] }
}
```

不存在 → `404 PROJECT_NOT_FOUND`；`project_id` 含非法字符 → `400 INVALID_ID`。

### POST `/api/projects`

创建项目。项目名不能为空、最长 200 字符，且不能与已有项目重名。

```bash
curl -X POST http://127.0.0.1:17890/api/projects \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"My Project\"}"
```

`201 Created`：

```json
{
  "project": { "id": "9f8e...", "name": "My Project", "folders": [] }
}
```

| 情况 | 状态码 | `error.code` |
| --- | --- | --- |
| `name` 为空或过长 | 400 | `INVALID_ARGUMENT` |
| `name` 已存在 | 409 | `PROJECT_NAME_TAKEN` |
| 请求体不是合法 JSON | 400 | `MALFORMED_JSON` |
| 缺少 `Content-Type: application/json` | 400 | `INVALID_CONTENT_TYPE` |

> 如果第一个项目是通过 API 创建的，它会被自动设为「当前项目」（`current_project_id`）。

### PUT `/api/projects/{project_id}`

重命名项目（只改名字）。

```bash
curl -X PUT http://127.0.0.1:17890/api/projects/9f8e... \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Renamed\"}"
```

```json
{ "project": { "id": "9f8e...", "name": "Renamed", "folders": [] } }
```

### DELETE `/api/projects/{project_id}`

从 Folder Manager 中删除项目 **记录**（含该项目的文件夹配置）。

```bash
curl -X DELETE http://127.0.0.1:17890/api/projects/9f8e...
```

```json
{
  "ok": true,
  "removed": { "kind": "project", "id": "9f8e...", "name": "Renamed" }
}
```

> **不会删除任何真实文件夹。** 如果删除的是当前项目，`current_project_id` 会自动切换到剩余的第一个项目。

### POST `/api/projects/{project_id}/folders`

把一个 Windows 文件夹加入指定项目。

```bash
curl -X POST http://127.0.0.1:17890/api/projects/9f8e.../folders \
  -H "Content-Type: application/json" \
  -d "{\"path\":\"C:\\\\work\\\\project\\\\src\"}"
```

请求字段：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `path` | 是 | **绝对路径**，必须存在且是目录 |
| `name` | 否 | 显示名，默认取目录名；为空时同样取目录名 |

处理步骤（由 Rust 业务逻辑执行）：

1. 校验 `path` 非空、是绝对路径、存在、且是目录；
2. 规范化路径（解析 `.` / `..` / 符号链接，统一为绝对路径）；
3. 默认名 = 目录名；
4. 生成稳定 `folder_id`；
5. 追加到指定项目；
6. 持久化到 `config.json`；
7. 广播 `workspace-changed` 事件，前端自动刷新。

`201 Created`：

```json
{ "id": "a1b2...", "name": "src", "path": "C:\\work\\project\\src" }
```

**幂等性（重要）**：如果该项目里已经存在同一个 `path`（忽略大小写、忽略结尾 `\`），
API **不会创建重复项**，而是直接返回已存在的文件夹，状态码为 `200 OK`（不是 201）。
Agent 可以安全地重复调用本接口。

| 情况 | 状态码 | `error.code` |
| --- | --- | --- |
| `path` 不存在 | 400 | `PATH_NOT_FOUND` |
| `path` 存在但不是目录 | 400 | `PATH_NOT_DIRECTORY` |
| `path` 是相对路径（如 `..\x`、`work\src`） | 400 | `INVALID_PATH` |
| `path` 在检查后被移除/无权限，无法解析 | 400 | `PATH_UNRESOLVED` |
| `project_id` 不存在 | 404 | `PROJECT_NOT_FOUND` |
| 同项目已有相同 `path` | **200** | —（返回已存在项） |
| 同项目已有同名但不同 `path` 的文件夹 | 409 | `FOLDER_NAME_TAKEN` |

**路径等价规则**：以下写法都指向同一个目录，添加时会被识别为同一条记录（不会重复）：

| 写法 | 是否等价 |
| --- | --- |
| `C:\work\project\src` | 基准 |
| `c:\WORK\project\src` | ✅ 忽略大小写 |
| `C:/work/project/src` | ✅ 统一分隔符 |
| `C:\work\project\src\` | ✅ 忽略结尾分隔符 |
| `C:\work\project\src\.` | ✅ canonicalize 解析 |
| `C:\work\project\other\..\src` | ✅ canonicalize 解析 |
| `\\?\C:\work\project\src` | ✅ 去掉扩展长度前缀 |
| `\\?\UNC\server\share\src` | ✅ 归一为 `\\server\share\src` |
| 指向同一目标的 symlink / junction | ✅ canonicalize 解析为同一真实路径 |

存储进 `config.json` 的始终是 `canonicalize` 之后的真实绝对路径（大写盘符、`\` 分隔符、
不含扩展长度前缀）。当前只支持**本机可访问的目录**（本地磁盘、映射盘与 UNC 网络路径）；
网络路径的存在性检查与规范化依赖系统调用，慢速网络可能让请求变慢。

### DELETE `/api/projects/{project_id}/folders/{folder_id}`

按 id 把文件夹从项目中移除。**只删除 `config.json` 中的记录**，磁盘目录与其内容完全不动。

```bash
curl -X DELETE \
  http://127.0.0.1:17890/api/projects/9f8e.../folders/a1b2...
```

```json
{
  "ok": true,
  "removed": {
    "kind": "folder",
    "id": "a1b2...",
    "name": "src",
    "path": "C:\\work\\project\\src"
  }
}
```

### DELETE `/api/projects/{project_id}/folders?path=...`

当 Agent 不知道 `folder_id` 时，可以用路径移除。

```bash
# C:\work\project\src  →  URL 编码后为 C%3A%5Cwork%5Cproject%5Csrc
curl -X DELETE \
  "http://127.0.0.1:17890/api/projects/9f8e.../folders?path=C%3A%5Cwork%5Cproject%5Csrc"
```

- 路径匹配忽略大小写、结尾 `\` 与扩展长度前缀（与添加时的等价规则一致）。
- 这里只做路径形状校验，**不要求目录当前仍然存在**（目录被移动/删除后依然可以清理配置记录）。
- 既没有 `folder_id` 也没有 `path` → `400 INVALID_ARGUMENT`。
- 路径不存在于该项目 → `404 FOLDER_NOT_FOUND`。

> **删除不存在的东西统一返回 `404 FOLDER_NOT_FOUND`**：两个删除 endpoint 行为一致，
> 都是**非幂等**的。Agent 重试删除时应当把 404 当作「已经是目标状态」处理，而不是失败。

### GET `/api/folders?path=...`

按路径片段（子串，忽略大小写）搜索所有项目中的文件夹。省略 `path` 返回全部文件夹。

```bash
curl "http://127.0.0.1:17890/api/folders?path=C%3A%5Cwork"
```

```json
{
  "query": "C:\\work",
  "matches": [
    {
      "project_id": "9f8e...",
      "project_name": "My Project",
      "folder": { "id": "a1b2...", "name": "src", "path": "C:\\work\\project\\src" }
    }
  ]
}
```

### GET `/api/folders/tree`

返回完整结构，便于 Agent 一次性获取上下文（内容与 `/api/projects` 相同）。

```bash
curl http://127.0.0.1:17890/api/folders/tree
```

```json
{
  "current_project_id": "9f8e...",
  "revision": 13,
  "projects": [ /* Project[] */ ]
}
```

---

## 6. 状态码与错误格式

所有错误响应使用 **同一种结构**：

```json
{
  "error": {
    "code": "PROJECT_NOT_FOUND",
    "message": "项目不存在: 9f8e..."
  }
}
```

| 状态码 | 含义 |
| --- | --- |
| 200 OK | 查询成功；添加已存在的文件夹（幂等命中）；更新 / 删除成功 |
| 201 Created | 成功创建项目 / 文件夹 |
| 400 Bad Request | 参数错误：非法 id、路径不存在/不是目录/非绝对路径、JSON 语法错误、字段缺失或为空 |
| 404 Not Found | 项目 / 文件夹 / 接口不存在 |
| 409 Conflict | 项目重名、文件夹同名 |
| 413 Payload Too Large | 请求体超过 64 KiB |
| 500 Internal Server Error | 读取或写入 `config.json` 失败等服务器内部错误 |

`error.code` 取值：

| code | 状态码 | 说明 |
| --- | --- | --- |
| `INVALID_ARGUMENT` | 400 | 参数缺失、为空或类型不对 |
| `INVALID_ID` | 400 | `project_id` / `folder_id` 含非法字符或过长 |
| `INVALID_PATH` | 400 | `path` 不是绝对路径 |
| `PATH_NOT_FOUND` | 400 | `path` 不存在 |
| `PATH_NOT_DIRECTORY` | 400 | `path` 存在但不是目录 |
| `PATH_UNRESOLVED` | 400 | `path` 无法解析为真实目录（检查后被移除 / 无权限） |
| `MALFORMED_JSON` | 400 | 请求体不是合法 JSON |
| `INVALID_CONTENT_TYPE` | 400 | 缺少 `Content-Type: application/json` |
| `PROJECT_NOT_FOUND` | 404 | 项目不存在 |
| `FOLDER_NOT_FOUND` | 404 | 项目中没有该文件夹（两个删除 endpoint 一致） |
| `NOT_FOUND` | 404 | 接口路径不存在 |
| `PROJECT_NAME_TAKEN` | 409 | 项目重名 |
| `FOLDER_NAME_TAKEN` | 409 | 同项目内文件夹重名 |
| `PAYLOAD_TOO_LARGE` | 413 | 请求体超过 64 KiB |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 |

**字段校验**（`{}`、`{"name":""}`、`{"name":"   "}`、`{"path":""}` 全部返回
`400 INVALID_ARGUMENT`，不会是 500）：

| 字段 | 规则 |
| --- | --- |
| `name`（项目 / 文件夹） | 去空格后非空，最长 200 字符 |
| `path` | 去空格后非空、绝对路径、存在、是目录、不超过 4096 字符 |

---

## 7. 安全边界

### 当前安全模型（Current security model）

- **只监听本机回环地址**（`127.0.0.1`）
- **没有认证**（no authentication）
- 面向**本机可信 Agent**（intended for local trusted agents）

```text
bind      : 127.0.0.1 only (never 0.0.0.0)
auth      : none
audience  : local trusted agents / scripts running as the same Windows user
```

同机任意进程（与本应用同一 Windows 用户）都可以调用。这是本地桌面工具 API 的常规取舍；
需要更强隔离时请用 Windows 防火墙 / 独立用户账户，而不是依赖本 API 自身。

未来若需要 `Authorization: Bearer <token>`，只需在 `src-tauri/src/infrastructure/http/api.rs`
的 `router()` 上挂一层 tower middleware —— handler 只调用 `ConfigService`，
`application` / `domain` 层不需要任何改动（架构已为此预留边界）。

具体边界：

1. **只绑定 `127.0.0.1`**：不监听 `0.0.0.0`，局域网与公网都无法访问。
2. **不能删除真实文件夹**：`DELETE` 只修改 `config.json`；
   代码里不存在 `remove_dir_all` / `remove_dir` 调用（全仓库可 grep 验证），
   `ConfigService` 的删除方法命名为 `remove_folder_reference_*` 以明确语义，
   域层删除只做 `Vec::remove`。
3. **不是文件读写接口**：
   - 只返回 Folder Manager 自己管理的项目/文件夹元数据；
   - 无法通过任何参数读取文件内容、列目录、写文件；没有 `/api/file`、`/api/fs` 之类 endpoint；
   - `path` 仅用于「校验目录是否存在」与「作为字符串写入配置」。
4. **路径处理**：
   - 只接受**绝对路径**，相对路径（含 `..\..\windows`）一律 `400 INVALID_PATH`；
   - `..` / `.` / 符号链接会被 `canonicalize` 解析成真实绝对路径后再写入配置，
     因此不存在"用 `..` 扩大 API 权限"的路径穿越；
   - 长度上限 4096 字符。
5. **id 严格校验**：`project_id` / `folder_id` 长度 1..=128，且只允许 `[A-Za-z0-9_-]`。
6. **请求体上限** 64 KiB：超出返回 `413 PAYLOAD_TOO_LARGE`，不会造成超大分配。
7. **日志**：只记录出错的 HTTP 方法、路径与状态码；不记录 header、token 与请求体。

### CORS

**CORS 不是认证**：它只决定浏览器是否允许跨源读取响应，不能阻止本机进程直接调用 API。
真正的边界是「只监听回环地址」。

默认**不放行任意公网来源**：

- 未配置时：debug 构建允许 `http://localhost` / `http://127.0.0.1` / `tauri://localhost`
  等本机页面来源的任意端口；release 构建默认不放开任何浏览器 origin。
- 非浏览器客户端（curl / Python / Node / Coding Agent）不发送 `Origin`，**不受 CORS 限制**，可直接访问。
- 需要时用环境变量显式配置（逗号分隔，`*` 表示放开全部 —— 不推荐）：

```powershell
$env:FOLDER_MANAGER_API_CORS_ORIGINS = "http://localhost:5173,https://my-tool.local"
```

- 非法 / 无法解析的 `Origin` 头只会被拒绝，不会让服务报错或崩溃。
- 修改 CORS 配置**不会**改变监听地址（始终是 `127.0.0.1`）。

---

## 8. 状态同步

Rust 端只有一个权威配置状态（`ConfigService`），Tauri command 与 HTTP API 共用它：

```text
Vue UI ──invoke(load_config/save_config)──┐
                                          ├─> ConfigService ─> config.json
HTTP API ─────────────────────────────────┘         │
                                                    └─> emit("workspace-changed")
                                                              │
                                          src/app/bootstrap.ts ┘
                                                              └─> projectStore.loadFromDisk()
                                                                  （Pinia 状态刷新 + 状态栏提示）
```

- 任何通过 API 的成功修改都会在**落盘成功后**广播 `workspace-changed` 事件。
- 前端收到事件后重新拉取权威快照，因此已打开的 UI 不会停留在旧状态。
- 事件只在写盘成功时发出，因此前端 reload 读到的内容一定是最新的。
- 幂等命中（重复添加同一路径）不修改数据，因此也不广播事件。
- 前端只在用户操作时提交快照，因此事件驱动刷新与用户操作不会互相覆盖。

### 并发一致性

`ConfigService` 把「读取 → 修改 → 序列化 → 落盘 → 提交内存 → revision+1 → 广播」
放在**同一个互斥锁**内（`transact()`），因此：

- 多个 Agent 同时 add / delete 不会互相覆盖（不存在「读旧快照 → 释放锁 → 写回」的窗口）；
- `revision` 单调递增，且只为成功的修改递增；
- 落盘失败时内存状态回滚，`GET` 返回的内容始终与磁盘一致，并返回 `500`；
- 并发添加同一个 `path` 仍然只有一条记录（幂等 + 事务）。

---

## 9. Agent 使用建议

1. 先 `GET /api` 看能力与安全边界，再 `GET /api/projects` 拿 `id`。
2. 记住 `id`，不要依赖数组顺序；id 是 UUID 字符串。
3. 添加文件夹前不必先查询：本接口幂等，重复调用返回 `200` 与已存在项。
4. 删除是**非幂等**的：删不存在的对象返回 `404 FOLDER_NOT_FOUND` / `404 PROJECT_NOT_FOUND`，
   重试时把 404 视为「已是目标状态」。
5. 需要轮询时用 `GET /api/health` 的 `revision` 字段，避免反复拉取全量数据。
6. 只使用 API 返回的 `id` 作为路径参数，不要自己拼 id。
7. 错误处理：读取 `error.code` 做分支（稳定），`error.message` 只用于展示/日志。
8. 路径统一用绝对 Windows 路径（`C:\work\project`）；URL 里记得百分号编码
   （`C%3A%5Cwork%5Cproject`）。

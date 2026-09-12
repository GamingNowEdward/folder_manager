# Changelog

本项目所有值得记录的变更都写在本文档中。

格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Added

- 本地 HTTP API（默认 `http://127.0.0.1:17890/api`，只绑定回环地址）：外部 AI Agent / Coding Agent / 自动化程序可以读取、创建、删除项目与文件夹。新增 `GET /api`（能力自描述）、`GET /api/health`、`GET /api/projects`、`POST /api/projects`、`GET|PUT|DELETE /api/projects/{id}`、`POST /api/projects/{id}/folders`、`DELETE /api/projects/{id}/folders/{folder_id}`、`DELETE /api/projects/{id}/folders?path=...`、`GET /api/folders?path=...`、`GET /api/folders/tree`；统一错误结构 `{ "error": { "code", "message" } }` 与 REST 状态码（200/201/400/404/409/500）。
- API 与 Tauri command 共用同一套业务逻辑：项目/文件夹的增删改规则下沉到 `domain/{project,folder}.rs`，编排集中在 `application/config_service.rs`，`commands/` 与 `infrastructure/http/` 都只是薄适配层。API 不会绕过配置管理直接改 JSON。
- API 修改配置后通过 Tauri event `workspace-changed` 通知前端，Pinia store 自动重新 hydrate，UI 不会停留在旧状态。
- 环境变量配置：`FOLDER_MANAGER_API_PORT`（默认 17890，端口被占用时自动顺延最多 10 个端口，全部失败只记录日志、不影响 UI）、`FOLDER_MANAGER_API_CORS_ORIGINS`（默认只放开本机来源，支持 `*` 与显式 origin 列表）。
- 路径安全处理：只接受绝对路径，`..` / `.` / 符号链接经 `canonicalize` 解析为真实绝对路径；相对路径与不存在/非目录路径返回 400。
- 幂等添加：同一项目内相同 `path`（忽略大小写与结尾 `\`）不创建重复项，返回 `200 OK` 与已存在文件夹（新建返回 `201 Created`）。
- `docs/API.md`：完整 API 文档（地址/端口、认证与安全模型、全部 endpoint、请求响应 JSON、状态码、错误格式、curl 示例、Agent 使用建议）。
- Rust API 测试：健康检查、项目读写、文件夹增删、错误路径（项目/文件夹不存在、path 不存在/非目录/相对路径）、重复添加幂等、`DELETE` 不删除真实目录、修改后 config 持久化、非法 id、API server 启动与优雅关闭；全部使用临时目录与临时 config。
- 本地一键发布脚本 `npm run release -- patch|minor|major`：自动执行质量门禁、同步版本文件（`package.json` / `Cargo.toml` / `Cargo.lock`）、提交、打 tag 并推送；CHANGELOG 仍由人工撰写并由脚本校验。
- 发布脚本支持将 `[Unreleased]` 自动提升为 `## [x.y.z] - 日期`（并保留新的空 `[Unreleased]`），发布前无需手工修改版本标题。

### Changed

- `lib.rs` 改用 `build()` + `run(callback)`，在 `RunEvent::Exit` 时优雅关闭 HTTP server。
- `ConfigService::initialize` 增加可选的变更事件出口（`ChangeSink`），application 层不直接依赖 Tauri。

## [1.1.0] - 2026-09-12

架构重构：在不破坏现有用户体验与配置兼容性的前提下，将项目从 UI 驱动的功能堆叠重构为业务能力清晰、依赖方向明确、前后端边界稳定的桌面应用架构。

### Added

- Rust 后端分层：`commands/`（薄 IPC 层）、`application/`（用例编排）、`domain/`（模型与迁移规则）、`infrastructure/`（JSON 持久化与 Windows 集成）以及统一错误类型 `error.rs`（`AppError`）。
- 配置 v2 持久化：`version: 2` + stable id（项目 / 文件夹）。
- 旧配置自动迁移：无 `version` 或 `version: 1` 的文件在加载时补齐 id，并将 `current_project` 由名称映射为 id。
- 数据保护：覆盖旧格式文件前生成 `config.json.bak`；损坏或版本不受支持的文件隔离为 `config.json.corrupt-<unix秒>.bak`，不再静默丢弃。
- 原子写入：先写 `config.json.tmp` 再 rename，避免写入中断损坏配置。
- 前端 Feature 化结构：`features/{projects,folders,selection,drag-drop,dialogs}` + `shared/` + `infrastructure/tauri/`。
- 统一 IPC adapters（`config` / `system` / `dialog` / `window`），业务代码不再直接 `invoke()` 或 import `@tauri-apps/*`。
- 交互能力 composable 化：`useFolderSelection`、`useBoxSelection`、`useFolderDragDrop`、`useExternalDrop`、`useDeleteShortcut`。
- 测试基础设施：Vitest（9 个文件 / 46 个用例）；Rust `cargo test`（20 个用例，覆盖迁移、规范化、备份、隔离与读写往返）。
- Lint / Format：ESLint 9（flat config）、Prettier。
- CI：`.github/workflows/ci.yml`（前端 ubuntu-latest；Rust fmt / clippy / test windows-latest）。
- 架构文档：`docs/ARCHITECTURE_REFACTOR.md`（现状分析、问题清单、目标结构、决策记录）。
- 版本与发布自动化：`package.json` 作为唯一版本来源（`tauri.conf.json` 引用之），新增 `scripts/sync-version.mjs` / `scripts/check-version.mjs`；推送 `v*` tag 触发 GitHub Actions 构建并创建 draft Release（MSI + 便携 exe，说明自动截取本文件）。

### Changed

- `App.vue` 从 588 行缩减为组合根（仅渲染 `AppShell` + 全局交互）；拖拽、框选、快捷键、对话框编排、持久化全部下沉。
- 单一 `stores/app.ts` 拆分为 `project.store`（projects + currentProjectId）、`folder.store`（当前项目文件夹操作层）、`status.store`。
- 业务身份由 `name` / 数组 index 迁移为 stable id：选区、拖拽、排序、删除、重命名均按 id 定位，UI index 仅用于渲染与命中测试。
- 组件只通过 props / emit 交互；容器组件与 composable 负责调用 store 与 infrastructure。
- 持久化统一走 `infrastructure/tauri/config.ts`，保留 150ms debounce 与快照式 `load_config` / `save_config` 协议。
- `lib.rs` 不再承载全部实现，仅负责 builder / plugins / setup / state / invoke_handler。

### Fixed

- 损坏或无法解析的 `config.json` 不再被静默替换为空配置。
- 修复配置保存失败日志的乱码。
- 修复以 name 为身份时重命名导致的选区 / 排序引用错位。

### Removed

- 根目录 9 个一次性脚本：`gen.cjs`、`rebuild-app.cjs`、`fix-addcard.cjs`、`fix-blur.cjs`、`fix-ctx.cjs`、`fix-ghost.cjs`、`fix-sidebar.cjs`、`patch-drag.cjs`、`patch-flip.cjs`。
- 死代码：`FlowLayout.vue`、`stores/app.ts`、`moveFolder` / `reorderFolders`、`dropIndicatorIndex`、`.is-drop-target` / `.drop-indicator-end` 样式。

### 兼容性

- 旧版 `config.json` 自动迁移，无需用户手动删除或修改文件。
- Tauri 命令名与快照协议保持不变；release / debug 配置存储位置不变。
- UI 外观、动画（FLIP、项目切换、Acrylic）与交互行为保持不变。

## [1.0.1] - 2026-08-04

### Changed

- 禁用 UI 文本选择（输入框除外）。

### Fixed

- 打开不存在的文件夹时显示"路径不存在"错误提示。

### Docs

- README 补充项目切换动画说明。

## [1.0.0] - 2026-07-27

### Added

- 首个版本：按项目分组管理文件夹，支持双击打开、复制路径、拖拽排序、框选 / Ctrl 多选批量删除、从资源管理器拖入自动添加。
- Windows DWM Acrylic 亚克力半透明界面。
- 便携持久化：release 便携版配置保存在 exe 同目录的 `config.json`。

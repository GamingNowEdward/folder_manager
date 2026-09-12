# Changelog

本项目所有值得记录的变更都写在本文档中。

格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

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

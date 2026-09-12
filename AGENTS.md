# AGENTS.md

## 项目概述
文件夹快速访问管理工具（Folder Manager）：按"项目"分组管理常用文件夹，支持双击打开、复制路径、拖拽排序、框选多选批量删除、从资源管理器拖入自动添加。

- 数据模型：`Project { id, name, folders }` → `Folder { id, name, path }`，stable id 为业务身份，UI index 仅用于渲染。
- 持久化：Rust 端读写 `config.json`（release 构建保存在 exe 同目录，便于便携分发；debug 构建保存在系统应用数据目录）。
- 配置格式：`version: 2`；旧版（无 `version`）自动迁移并补齐 id；覆盖旧格式前生成 `config.json.bak`；损坏文件隔离为 `config.json.corrupt-<时间戳>.bak`。
- 窗口效果：Windows 上通过 DWM API 启用 Acrylic 亚克力背景（`src-tauri/src/infrastructure/windows/acrylic.rs`）。

## 技术栈
- **Tauri v2** 桌面应用：Rust 后端位于 `src-tauri/`，**Vue 3** + **Pinia** 前端位于 `src/`
- Vite 开发服务器固定端口 **1420**（设置 `TAURI_DEV_HOST` 时 HMR 使用 1421）。不要修改。
- 入口：`src/main.ts` → `src/app/bootstrap.ts` → `src/App.vue` → `src/app/AppShell.vue`。
- IPC：`load_config` / `save_config` / `open_folder`（快照式，前端提交全量 `WorkspaceData`）。
- 架构分析与决策记录见 `docs/ARCHITECTURE_REFACTOR.md`；版本变更历史见 `docs/CHANGELOG.md`。

### 目录结构
- `src/app/` — 组合根：`AppShell`、`bootstrap`、`TitleBar/StatusBar/DialogHost`、全局交互（`useGlobalInteractions`）
- `src/features/projects/` — 项目领域：`project.model`、`project.store`、`useProjectActions`、Sidebar/ProjectDialog
- `src/features/folders/` — 文件夹领域：`folder.model`、`folder.store`（当前项目文件夹操作层，状态归 project store）、`useFolderActions`、FolderCard/AddFolderCard/FolderDialog/FolderWorkspace
- `src/features/selection/` — 选择：`useFolderSelection`、`useBoxSelection`
- `src/features/drag-drop/` — 拖拽：`useFolderDragDrop`、`useExternalDrop`、命中测试纯函数
- `src/features/dialogs/` — 对话框编排：`useDialogs`、ConfirmDialog
- `src/shared/` — 跨 feature 的常量、工具、status store
- `src/infrastructure/tauri/` — Tauri IPC adapters（config / system / dialog / window）
- `src/types/index.ts` — 领域类型
- `src-tauri/src/` — `commands/`（薄 IPC 层）、`application/`（编排）、`domain/`（模型与迁移规则）、`infrastructure/`（JSON 持久化、Windows 集成）、`error.rs`

## 环境要求
接手开发需要以下环境（Windows 平台）：

- **Node.js 18+**（Vite 6 要求），`npm install` 恢复前端依赖
- **Rust stable** 工具链（通过 rustup 安装）
- **Visual Studio Build Tools 2022**：勾选"使用 C++ 的桌面开发"工作负载（含 MSVC v143 工具集 + Windows 10/11 SDK）——Rust 在 Windows 编译的硬性依赖
- **WebView2 Runtime**：Windows 11 / 更新过的 Win10 1803+ 已内置
- **7-Zip**：仅运行 `pack-7z.ps1` 打包脚本时需要

提示：首次 Rust 编译需下载并编译全部依赖，耗时数分钟，`src-tauri/target/` 会膨胀至约 7.5GB，属正常现象（已被 git 忽略 / 打包排除）。

## 命令
- `npm run dev` — 仅启动 Vite 开发服务器（不会打开 Tauri 窗口）。
- `npm run tauri dev` — 完整 Tauri 开发模式（Rust + Vite）。
- `npm run typecheck` — `vue-tsc --noEmit`。
- `npm run lint` — ESLint（flat config）全量检查。
- `npm run test` — Vitest 单元测试（纯逻辑 + store）。
- `npm run test:watch` — Vitest 监听模式。
- `npm run format` / `npm run format:check` — Prettier。
- `npm run build` — `typecheck` + `vite build`。
- `npm run tauri build` — 完整原生构建。
- `cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test --all-features`（在 `src-tauri/` 下执行）。
- `.\pack-7z.ps1` — 将源码打包为 `folder-manager-src.7z`（排除构建产物）。

## 测试
- 前端单测使用 Vitest（node 环境，不依赖 DOM），文件名 `*.spec.ts`，与对应 `model/` 或 `store/` 同目录。
- 纯业务规则必须覆盖：selection、reorder、normalization、config mapping、project / folder mutations。
- Rust 测试写在模块内 `#[cfg(test)]`；涉及文件系统的用例使用 `tempfile` 隔离。

## 质量门禁
提交 / PR 前需通过（与 `.github/workflows/ci.yml` 一致）：

- 前端：`npm run typecheck`、`npm run lint`、`npm run test`、`npm run build`
- Rust（`src-tauri/` 下）：`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test --all-features`

## 版本与发布
- 版本唯一来源：`package.json` 的 `version`；`tauri.conf.json` 通过 `"version": "../package.json"` 引用它。
- 发布流程：更新 `docs/CHANGELOG.md`（人工撰写目标版本段落）并提交 → 运行 `npm run release -- patch|minor|major`（自动跑质量门禁、同步版本文件、提交、打 tag、推送）→ GitHub Actions 构建并创建 **draft** Release（说明自动截取自 `docs/CHANGELOG.md`）→ 人工确认后发布。
- `scripts/release.mjs` 要求工作区干净、`main` 分支且与 `origin/main` 同步；支持 `--dry-run`（预演）与 `--skip-checks`（应急跳过门禁）。
- 手动替代路径：`npm version x.y.z --no-git-tag-version` → `npm run version:check` → commit → `git tag vX.Y.Z` → push tag。
- tag 与 `package.json` 版本不一致时 release workflow 会直接失败。

## 风格 / 约定
- `.vue` 文件使用 `<script setup lang="ts">` 风格（与现有文件保持一致）。
- 跨目录 import 使用 `@/` alias（配置见 `vite.config.ts` 与 `tsconfig.json`），同目录使用相对路径。
- 叶子组件只通过 props / emit 交互；容器组件与 composable 负责调用 store 与 infrastructure。
- 组件禁止直接 `invoke()`、读写文件或 import `@tauri-apps/*`；统一走 `src/infrastructure/tauri/`。
- 业务规则放 `features/*/model`（纯函数，必须有单测）；状态编排放 feature store。
- CSS 集中在 `src/assets/styles.css`；组件私有样式写在 `<style scoped>`。
- `dist/`、`src-tauri/target/`、`src-tauri/gen/schemas/` 是构建产物；不要修改或提交。

## Tauri 注意事项
- 命令名与载荷保持兼容（快照式设计）；若变更载荷，需同步 `src/infrastructure/tauri/config.mapper.ts` 与 Rust `infrastructure/persistence/config_file.rs`。
- Vite 会忽略 `src-tauri/**` 的 HMR；修改 Rust 代码需要重启 Tauri 进程。
- release 与 debug 的配置存储位置不同（见"项目概述"），测试数据迁移时注意。

## 分发
- 成品：`npm run tauri build` → `src-tauri\target\release\folder-manager.exe`（便携版，config.json 生成在 exe 旁）及 `src-tauri\target\release\bundle\msi\*.msi`（安装包）。
- 源码：`.\pack-7z.ps1` → `folder-manager-src.7z`（约 1MB 以内），接收方按"环境要求"配置后 `npm install` → `npm run tauri dev` 即可运行。

# AGENTS.md

## 项目概述
文件夹快速访问管理工具（Folder Manager）：按"项目"分组管理常用文件夹，支持双击打开、复制路径、拖拽排序、框选多选批量删除、从资源管理器拖入自动添加。

- 数据模型：`Project → Folder { name, path }`，类型定义见 `src/types/index.ts`
- 持久化：Rust 端读写 `config.json`（release 构建保存在 exe 同目录，便于便携分发；debug 构建保存在系统应用数据目录）
- 窗口效果：Windows 上通过 DWM API 启用 Acrylic 亚克力背景（`src-tauri/src/lib.rs`）

## 技术栈
- **Tauri v2** 桌面应用：Rust 后端位于 `src-tauri/`，**Vue 3** + **Pinia** 前端位于 `src/`
- Vite 开发服务器固定端口 **1420**（设置 `TAURI_DEV_HOST` 时 HMR 使用 1421）。不要修改。
- 入口：`src/main.ts` -> `src/App.vue`。状态仓库：`src/stores/app.ts`。类型：`src/types/index.ts`。

### 目录结构
- `src/components/` — UI 组件：`TitleBar`、`Sidebar`（项目列表）、`FolderCard`（文件夹卡片）、`AddFolderCard`、`StatusBar`、`FlowLayout`
- `src/dialogs/` — `ProjectDialog`、`FolderDialog`、`ConfirmDialog`
- `src/App.vue` — 主交互逻辑：拖拽排序、框选、外部拖入、对话框编排
- `src-tauri/src/lib.rs` — 全部 Rust 命令：`load_config`、`save_config`、`open_folder`
- 根目录 `fix-*.cjs`、`gen.cjs`、`patch-*.cjs`、`rebuild-app.cjs` 是一次性脚本，不属于构建流程

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
- `npm run build` — 先执行 `vue-tsc --noEmit`，再执行 `vite build`（类型检查 + 生产构建）。
- `npm run tauri build` — 完整原生构建。
- `.\pack-7z.ps1` — 将源码打包为 `folder-manager-src.7z`（排除构建产物）。
- 未配置测试或 lint 脚本；`vue-tsc --noEmit` 是唯一的验证命令。

## 风格 / 约定
- `.vue` 文件使用 `<script setup lang="ts">` 风格（与现有文件保持一致）。
- CSS 集中在 `src/assets/styles.css`；无 CSS modules 或预处理器。
- 根目录下的 `fix-*.cjs`、`gen.cjs`、`patch-*.cjs` 是一次性 Node 脚本，不属于构建流程；不要把它们挂到 `package.json`。
- `dist/`、`src-tauri/target/`、`src-tauri/gen/schemas/` 是构建产物；不要修改或提交。

## Tauri 注意事项
- 前端通过 `@tauri-apps/api` 及插件（`dialog`、`clipboard-manager`）与 Rust 通信，使用 `invoke()` 调用——确保载荷可序列化。
- Vite 会忽略 `src-tauri/**` 的 HMR；修改 Rust 代码需要重启 Tauri 进程。
- release 与 debug 的配置存储位置不同（见"项目概述"），测试数据迁移时注意。

## 分发
- 成品：`npm run tauri build` → `src-tauri\target\release\folder-manager.exe`（便携版，config.json 生成在 exe 旁）及 `src-tauri\target\release\bundle\msi\*.msi`（安装包）。
- 源码：`.\pack-7z.ps1` → `folder-manager-src.7z`（约 1MB 以内），接收方按"环境要求"配置后 `npm install` → `npm run tauri dev` 即可运行。

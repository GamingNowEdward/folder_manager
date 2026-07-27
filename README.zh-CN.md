[English](./README.md) | **简体中文**

# Folder Manager

文件夹快速访问管理工具 —— 按"项目"分组管理常用文件夹，一键直达。Windows 桌面应用，亚克力（Acrylic）半透明界面。

## 功能

- **项目分组**：为不同工作/场景建立独立的项目，各自维护一组文件夹
- **快速访问**：双击卡片在资源管理器中打开文件夹，右键或点按钮复制路径
- **多种添加方式**：对话框手动添加 / 浏览选择 / 直接从资源管理器拖入窗口自动添加
- **拖拽排序**：按住卡片拖动即可调整顺序，带 FLIP 平滑动画
- **批量操作**：框选或 Ctrl+ 多选卡片，Delete 键批量删除
- **便携持久化**：配置保存为 `config.json`（便携版在 exe 同目录），无需安装即可迁移

## 技术栈

- [Tauri v2](https://tauri.app)（Rust 后端）+ [Vue 3](https://vuejs.org) + [Pinia](https://pinia.vuejs.org)
- Vite 6 + TypeScript
- Windows DWM Acrylic 背景

## 使用

### 直接运行

从 Release 下载（或自行构建后取用）：

- `folder-manager.exe` — 便携版，直接运行，`config.json` 自动生成在 exe 旁
- `Folder Manager_x.x.x_x64_xx-XX.msi` — Windows 安装包

运行环境：Windows 10 1803+ / Windows 11（WebView2 已内置）。

## 从源码构建

### 环境要求

| 工具 | 版本 | 说明 |
| --- | --- | --- |
| Node.js | 18+ | 前端构建（Vite 6 要求） |
| Rust | stable | 通过 [rustup](https://rustup.rs) 安装 |
| VS Build Tools 2022 | — | 勾选"使用 C++ 的桌面开发"工作负载（MSVC v143 + Windows SDK），Rust 在 Windows 编译的硬性依赖 |
| WebView2 Runtime | — | Win11 / 更新过的 Win10 已内置 |

> 首次 Rust 编译需下载并编译全部依赖，耗时数分钟；`src-tauri/target/` 会增至约 7.5GB，属正常现象（已 git 忽略）。

### 开发

```powershell
npm install
npm run tauri dev
```

### 生产构建

```powershell
npm run tauri build
```

产物：

- `src-tauri\target\release\folder-manager.exe`
- `src-tauri\target\release\bundle\msi\Folder Manager_1.0.0_x64_en-US.msi`

### 打包源码

```powershell
.\pack-7z.ps1
```

生成 `folder-manager-src.7z`（约 200KB），已排除全部构建缓存，可直接发给他人。

## 项目结构

```
├── src/                  # Vue 前端
│   ├── components/       # TitleBar / Sidebar / FolderCard / StatusBar 等
│   ├── dialogs/          # 项目 / 文件夹 / 确认对话框
│   ├── stores/app.ts     # Pinia 状态仓库 + 持久化
│   └── types/            # 类型定义
├── src-tauri/            # Rust 后端（Tauri 命令、配置读写、亚克力窗口）
├── pack-7z.ps1           # 源码打包脚本
└── AGENTS.md             # AI 辅助开发约定
```

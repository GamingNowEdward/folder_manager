# Folder Manager 架构重构记录

> 状态：已完成（Phase 1–10 全部执行完毕，验收结果见 §8）
> 约束：不破坏现有用户体验与 `config.json` 兼容性；不重写 UI；不做过度抽象。

## 0. 已确认的架构决策

| 编号 | 决策 | 理由 |
| --- | --- | --- |
| D1 | 保留快照式 IPC：`load_config` / `save_config` / `open_folder` 命令名与语义不变 | 行为不变、无 async 往返、避免前后端双份实现业务规则；Rust 负责持久化边界规则 |
| D2 | 配置升级为 v2（`version: 2` + stable id），v1 自动迁移；覆盖 v1 文件前生成 `config.json.bak`；损坏文件隔离为 `config.json.corrupt-<时间戳>.bak` | 数据不丢失；ID 身份稳定 |
| D3 | 采用 Feature-oriented + Lightweight Layered 结构，不做 Enterprise DDD | 桌面工具规模有限，抽象成本需可控 |
| D4 | 不引入 repository interface / factory / DI 容器 / event bus | 只有一个 JSON backend，无真实替换需求 |
| D5 | Rust 不设 `project_service` / `folder_service`：领域规则集中在 `domain`（迁移、规范化），编排集中在 `application/config_service` | 快照式 IPC 下不存在按操作粒度的用例，强行拆分会产生死代码 |

## 1. 重构前架构分析

### 1.1 调用链

```text
Vue Component（emit）
    ↓
App.vue（588 行；状态 + 业务 + DOM 算法 + IPC + 快捷键）
    ↓
stores/app.ts（单一 store：业务 + UI 状态 + 持久化 + invoke）
    ↓
invoke('load_config' / 'save_config' / 'open_folder')
    ↓
src-tauri/src/lib.rs（DTO + 领域 + 持久化 + 命令 + Acrylic + builder，142 行）
    ↓
config.json（release: exe 同目录；debug: app_data_dir）
```

旁路直连：

- `App.vue` — `invoke`、`writeText`、`getCurrentWindow`、`onDragDropEvent`
- `dialogs/FolderDialog.vue` — `@tauri-apps/plugin-dialog` 的 `open()`
- `components/TitleBar.vue` — `@tauri-apps/api/window`

### 1.2 持久化流程（重构前）

- 启动：`lib.rs` setup 读取文件；解析失败使用空数据（`.unwrap_or_default()`），随后任何保存都会静默覆盖原文件。
- 前端：`App.vue` `onMounted` 调 `loadFromDisk()`；任意 mutation 触发 150ms debounce 的 `save_config` 全量快照；Rust 端 `fs::write` 直接覆盖，非原子写。
- 无 `version`、无迁移、无备份。

### 1.3 问题清单

| # | 文件 | 职责 | 问题与后果 | 重构方案 |
| --- | --- | --- | --- | --- |
| 1 | `App.vue` | 项目/文件夹对话框增删改分支与重名校验 | 业务规则不可单测；任何对话框调整都要改根组件 | `useProjectActions` / `useFolderActions` + 纯 model |
| 2 | `App.vue` | 拖拽状态机、ghost、实时 reorder、框选、Ctrl 多选、Delete 快捷键 | 核心交互全部汇聚根组件 | `useFolderDragDrop` / `useBoxSelection` / `useFolderSelection` / `useDeleteShortcut` |
| 3 | `App.vue` | 直接 `invoke` / `writeText` | 组件耦合 Tauri，无法 mock | `infrastructure/tauri/{system,window}.ts` |
| 4 | `App.vue` | `getDropTargetIndex` 依赖 `data-folder-index`；`liveReorder` 以 name 定位 | 算法不可测；index / name 被当作业务身份 | 纯函数 `resolveDropTargetIndex` + stable id |
| 5 | `stores/app.ts` | UI 状态（status）+ 业务 + 持久化 + IPC 混用 | 职责三位一体；`_save` 被组件直接调用 | 拆 `project.store` / `folder.store` / `status.store` + `configRepository` |
| 6 | 全局 | 身份基于 `name`（当前项目、去重、删除、选区、拖拽、`:key`） | 重命名 / 重名敏感；index 变化易错 | `ProjectId` / `FolderId` |
| 7 | `stores/app.ts` | 日志字符串 mojibake（`'閰嶇疆淇濆瓨澶辫触:'`） | 调试信息不可读 | 修复为 `'配置保存失败:'` |
| 8 | `src-tauri/src/lib.rs` | DTO = 领域 = 持久化模型 + 命令 + Acrylic + builder | 扩展即 God Module | 按 `commands / application / domain / infrastructure` 拆分 |
| 9 | Rust | 无统一错误类型（全 `String`）、非原子写、无版本/迁移/备份 | 崩溃可能损坏配置；旧/坏配置静默丢弃 | `error.rs` + `json_store` + `domain::config` 迁移与规范化 |
| 10 | `types/index.ts` | 仅 name/path DTO | domain / DTO / 持久化模型无边界 | domain 类型 + `config.mapper.ts` DTO 映射 |

### 1.4 死代码（重构前已确认）

- `components/FlowLayout.vue` — 零引用。
- `styles.css` 中 `.is-drop-target`、`.drop-indicator-end` — 零引用。
- `stores/app.ts` 中 `moveFolder` / `reorderFolders` — 零调用。
- `App.vue` 中 `dropIndicatorIndex`（只写不读）、`.project-dropdown` 判断（元素不存在）。
- `src/utils/` — 空目录。
- 根目录 9 个 `.cjs` 一次性脚本（见 §6）。

## 2. 目标结构

### 2.1 前端

```text
src/
├── main.ts
├── App.vue                         # 仅组合根：AppShell + 全局交互
├── app/
│   ├── bootstrap.ts                # createApp + Pinia + 挂载 + hydrate
│   ├── AppShell.vue                # 布局与容器组件组合
│   ├── components/{TitleBar,StatusBar,DialogHost}.vue
│   └── composables/useGlobalInteractions.ts
├── features/
│   ├── projects/
│   │   ├── components/{Sidebar,ProjectDialog}.vue
│   │   ├── composables/useProjectActions.ts
│   │   ├── model/project.model.ts
│   │   ├── store/project.store.ts
│   │   └── index.ts
│   ├── folders/
│   │   ├── components/{FolderCard,AddFolderCard,FolderDialog,FolderWorkspace}.vue
│   │   ├── composables/{useFolderActions,useDeleteShortcut,useDirectoryPicker}.ts
│   │   ├── model/folder.model.ts
│   │   ├── store/folder.store.ts
│   │   └── index.ts
│   ├── selection/
│   │   ├── composables/{useFolderSelection,useBoxSelection}.ts
│   │   ├── model/selection.model.ts
│   │   └── index.ts
│   ├── drag-drop/
│   │   ├── composables/{useFolderDragDrop,useExternalDrop}.ts
│   │   ├── model/{drop-target,external-drop}.ts
│   │   └── index.ts
│   └── dialogs/
│       ├── components/ConfirmDialog.vue
│       ├── composables/useDialogs.ts
│       └── index.ts
├── shared/
│   ├── stores/status.store.ts
│   ├── utils/{id,path,errors}.ts
│   └── constants/index.ts
├── infrastructure/tauri/
│   ├── commands.ts                 # 命令名常量 + typed invoke
│   ├── config.ts                   # load / scheduleSave / flush
│   ├── config.mapper.ts            # DTO ↔ domain（防御性规范化）
│   ├── system.ts                   # openFolder / copyPath
│   ├── dialog.ts                   # pickDirectory
│   └── window.ts                   # 窗口控制 / 文件拖入监听
├── types/index.ts                  # domain 类型（Project / Folder / ids）
└── assets/styles.css
```

依赖方向：

```text
App / AppShell
    ↓
Feature 容器组件（Sidebar / FolderWorkspace / DialogHost）
    ↓
composable / feature store
    ↓
feature model（纯函数）
    ↓
infrastructure/tauri adapters
    ↓
Tauri IPC
```

规则：

- 叶子组件（FolderCard、AddFolderCard、三个 Dialog）只通过 props / emit 交互。
- 组件不直接 `invoke()`，不直接读写 JSON，不直接调用 `@tauri-apps/*`。
- 跨 feature import 使用 feature 的 `index.ts`；跨 feature 依赖单向（folders → projects、drag-drop → folders/projects）。

### 2.2 Rust

```text
src-tauri/src/
├── main.rs
├── lib.rs                          # 仅 builder / plugins / setup / manage / handler
├── error.rs                        # AppError（统一错误类型，serde 可序列化）
├── commands/
│   ├── mod.rs
│   ├── config.rs                   # load_config / save_config（薄封装）
│   └── system.rs                   # open_folder（薄封装）
├── application/
│   ├── mod.rs
│   └── config_service.rs           # 内存状态 + load/save 编排
├── domain/
│   ├── mod.rs
│   ├── config.rs                   # Workspace + 规范化规则
│   ├── project.rs
│   └── folder.rs
└── infrastructure/
    ├── mod.rs
    ├── persistence/
    │   ├── mod.rs
    │   ├── config_file.rs          # v1/v2 serde DTO + 迁移/解析
    │   └── json_store.rs           # 路径解析、读取、原子写、备份、隔离
    └── windows/
        ├── mod.rs
        ├── acrylic.rs              # DWM Acrylic
        └── shell.rs                # explorer 打开文件夹
```

依赖方向：

```text
lib.rs → commands → application → domain
application → infrastructure::persistence（写盘）
commands::system → infrastructure::windows::shell
domain 不依赖 tauri / windows / filesystem / serde
```

说明：IPC 与持久化共用同一序列化契约（快照式设计使二者等价），因此 serde DTO 归属 `infrastructure::persistence::config_file`，command 只做类型搬运；业务规则（迁移、规范化）在 `domain`。

### 2.3 数据模型

```ts
type ProjectId = string
type FolderId = string

interface Folder  { id: FolderId;  name: string; path: string }
interface Project { id: ProjectId; name: string; folders: Folder[] }
interface WorkspaceData { currentProjectId: ProjectId; projects: Project[] }
```

- 决策：folders 保持嵌套数组（非 `folderIds` map）——对本应用等价且改动最小。
- `UI index` 仅用于渲染与命中测试，不再作为业务身份；`selection / drag / delete / rename / :key` 全部使用 id。

### 2.4 配置兼容

- v1（无 `version`）：`{ current_project: <name>, projects: [{ name, folders: [{ name, path }] }] }`
- v2：`{ version: 2, current_project: <projectId>, projects: [{ id, name, folders: [{ id, name, path }] }] }`
- 迁移：无 `version` / `1` → v1 迁移（补 id，`current_project` 由 name 匹配为 id）；`2` → 规范化补齐缺失 id；`> 2` → 错误并隔离原文件。
- 规范化保守策略：只补 id / 修正失效 `current_project`，不丢弃任何条目。
- 保存：写入前若磁盘文件为旧格式，先生成 `config.json.bak`；写入采用 temp + rename 原子替换。
- 读取损坏文件：重命名为 `config.json.corrupt-<unix秒>.bak` 后以空配置启动，原数据不丢失。

## 3. 前端职责迁移映射

| 重构前位置（App.vue / store） | 重构后位置 |
| --- | --- |
| 项目对话框提交、重名校验、删除确认 | `features/projects/composables/useProjectActions.ts` |
| 文件夹对话框提交、重名校验、删除确认、打开、复制 | `features/folders/composables/useFolderActions.ts` |
| 外部拖入去重与状态提示 | `features/drag-drop/composables/useExternalDrop.ts` + `model/external-drop.ts` |
| `getDropTargetIndex` / `liveReorder` / 拖拽状态机 / ghost | `features/drag-drop/composables/useFolderDragDrop.ts` + `model/drop-target.ts` |
| 框选、Ctrl 多选 | `features/selection/composables/{useBoxSelection,useFolderSelection}.ts` + `model/selection.model.ts` |
| Delete 快捷键 | `features/folders/composables/useDeleteShortcut.ts` |
| 三个对话框的可见性与 confirm 回调 | `features/dialogs/composables/useDialogs.ts` + `app/components/DialogHost.vue` |
| `statusMessage` / `setStatus` | `shared/stores/status.store.ts` |
| `_save` / `loadFromDisk` | `project.store` → `infrastructure/tauri/config.ts` |
| 主区域卡片网格、header、ghost、框选覆盖层 | `features/folders/components/FolderWorkspace.vue` |

必须逐字保留的行为：5px 拖拽阈值；拖拽中实时 reorder + FLIP；非拖拽点击选中；Ctrl 切换；空区框选；Delete 全部 guard；外部拖入按名去重与状态文案；status 4s 复位；150ms 保存防抖；全部中文提示文案；Acrylic（DWMWA 38 = 3）；release/debug 路径差异。

## 4. 测试策略

### TypeScript（Vitest，node 环境）

- `project.model`：创建、重名、重命名、删除。
- `folder.model`：添加、重名、更新、批量删除、按 id 移动到目标 index。
- `selection.model`：toggle、selectOnly、矩形相交。
- `drop-target`：行分组 / 半宽命中 / fallback index。
- `external-drop`：空名跳过、重名跳过、批内去重。
- `config.mapper`：DTO → domain（补 id、清理无效 current）、domain → DTO（带 version）。
- Stores：`project.store` / `folder.store`（mock `configRepository`）。

### Rust（cargo test）

- `domain::config`：id 补齐 / 去重 / 无效 current 清理。
- `persistence::config_file`：v1 迁移、v2 保留 id、未知版本拒绝、往返一致。
- `persistence::json_store`：缺失文件、往返、旧格式备份、损坏隔离、未知版本隔离、无残留临时文件。
- `windows::shell`：路径不存在返回 `PathNotFound`（不发起点进程）。

## 5. Lint / 验证链路

前端（Phase 8 建立后可用）：

```text
npm run typecheck   # vue-tsc --noEmit
npm run lint        # eslint .
npm run test        # vitest run
npm run build       # typecheck + vite build
npm run format      # prettier --write
```

Rust：

```text
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

CI：`.github/workflows/ci.yml`（frontend on ubuntu-latest；cargo fmt/clippy/test on windows-latest）。

## 6. 一次性脚本处理结论

| 脚本 | 结论 | 依据 |
| --- | --- | --- |
| `gen.cjs` | 删除 | 引用旧路径 `c:/opencode/folder_manager_tauri`，生成物已被后续迭代取代 |
| `rebuild-app.cjs` | 删除 | 同上 |
| `fix-addcard.cjs` | 删除 | 目标改动已存在于源码（`v-if="store.currentFolders.length === 0"`） |
| `fix-blur.cjs` | 删除 | 目标改动已存在于源码（`onWindowBlur`） |
| `fix-ctx.cjs` | 删除 | 目标改动已存在于源码（contextmenu 拦截） |
| `fix-ghost.cjs` | 删除 | 目标改动已存在于源码（ghost blur 样式） |
| `fix-sidebar.cjs` | 删除 | 目标改动已存在于源码（action-icon） |
| `patch-drag.cjs` | 删除 | 其引入的 flip 逻辑已被 `patch-flip.cjs` 移除，最终态即当前代码 |
| `patch-flip.cjs` | 删除 | 最终态已存在于源码（TransitionGroup + flip-list） |
| `pack-7z.ps1` | 保留 | 真实分发工具，README 已文档化 |

## 7. 执行阶段与验证记录

| 阶段 | 内容 | 状态 | 验证 |
| --- | --- | --- | --- |
| P1 | 本文档 | ✅ | 文档 |
| P2/P3 | Rust 分层 + 迁移 + 测试 | ✅ | `cargo test`（20 通过）/ `cargo clippy -D warnings` / `cargo fmt --check` |
| P4 | 前端 infrastructure adapters | ✅ | `vue-tsc` / `vitest` |
| P5 | Pinia 拆分 + stable id | ✅ | `vue-tsc` / `vitest` |
| P6 | composables + FolderWorkspace | ✅ | `vue-tsc` |
| P7 | App.vue 组合根 + 删除旧文件 | ✅ | `vue-tsc` / `vite build` |
| P8 | Vitest/ESLint/Prettier + CI | ✅ | `npm run lint` / `npm run test`（46 通过）/ `format:check` |
| P9 | 脚本/死代码清理 + 文档更新 | ✅ | `git status` / 边界 grep |
| P10 | 全量验收 | ✅ | 见 §8 |

### 执行期环境说明

- 本机初始缺少 MSVC 链接器（`link.exe`），`cargo test` 无法编译；在本阶段安装了 Visual Studio Build Tools 2022（VCTools 工作负载）后恢复。
- MSVC 在中文系统下会向 rustc 输出 GBK 链接日志，触发 `linker_messages` 警告；已在 `Cargo.toml` 增加 `[lints.rust] linker_messages = "allow"` 以保持构建输出干净（不影响任何 lint 正确性）。

## 8. 验收结果（Phase 10）

| 命令 | 结果 |
| --- | --- |
| `npm run typecheck` | ✅ exit 0 |
| `npm run lint` | ✅ exit 0（eslint 9 flat config） |
| `npm run test` | ✅ 9 个测试文件 / 46 个测试通过 |
| `npm run build` | ✅ 98 modules transformed；dist 产物正常 |
| `cargo fmt --all -- --check` | ✅ exit 0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | ✅ exit 0 |
| `cargo test --all-features` | ✅ 20 个测试通过（迁移 / 规范化 / 持久化 / 隔离 / shell） |
| `npm run tauri build` | ✅ `target/release/folder-manager.exe` + `bundle/msi/Folder Manager_1.0.0_x64_en-US.msi` |
| release exe 冒烟 | ✅ 启动后 6 秒进程存活（WebView + 配置初始化正常），随后主动结束 |
| 边界校验 | ✅ `src/**` 中 infrastructure 之外无 `invoke(` / `@tauri-apps` 引用 |

### 未自动化验证项（需人工确认）

以下 Tauri 窗口交互无法通过 CLI 自动验证，代码路径已按原行为迁移并在单测中覆盖算法，但仍建议人工过一遍：

- 拖拽排序与 FLIP 动画、拖拽 ghost 样式
- 框选、Ctrl 多选、Delete 批量删除确认流程
- 项目切换过渡动画、卡片 hover、右键复制路径
- 从资源管理器拖入文件夹（外部 drop）
- Acrylic 背景与无边框窗口按钮

### 人工验证步骤

1. 备份当前 `src-tauri/target/release/config.json`（若存在）。
2. 将旧版 `config.json`（无 `version`、含 name/path）放到 exe 同目录，启动 `folder-manager.exe`。
3. 确认项目与文件夹完整、当前项目正确；执行任意改名/排序后关闭，检查 `config.json` 已升级为 `version: 2` 且生成 `config.json.bak`。
4. 依次验证 §7 表格中"未自动化验证项"的交互行为。

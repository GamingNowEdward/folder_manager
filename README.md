**English** | [简体中文](./README.zh-CN.md)

# Folder Manager

A quick-access folder manager — organize your frequently used folders into "projects" and jump to them in one click. A Windows desktop app with an Acrylic translucent UI.

## Features

- **Project groups**: Create separate projects for different workstreams or scenarios, each with its own set of folders
- **Smooth project switching**: Cross-fade transition when switching between projects
- **Quick access**: Double-click a card to open the folder in File Explorer; right-click or use the button to copy its path
- **Multiple ways to add**: Add via dialog, browse to pick a directory, or drag folders straight in from File Explorer
- **Drag to reorder**: Grab a card and drag it to a new position, with smooth FLIP animations
- **Batch operations**: Box-select or Ctrl+click to multi-select cards, then press Delete to remove them all
- **Portable persistence**: Settings are stored in `config.json` (next to the exe in the portable build) — migrate without installing anything

## Tech Stack

- [Tauri v2](https://tauri.app) (Rust backend) + [Vue 3](https://vuejs.org) + [Pinia](https://pinia.vuejs.org)
- Vite 6 + TypeScript
- Windows DWM Acrylic backdrop

## Usage

### Prebuilt binaries

Download from Releases (or build your own):

- `folder-manager.exe` — portable edition; just run it, and `config.json` is created next to the exe
- `Folder Manager_x.x.x_x64_xx-XX.msi` — Windows installer

Runtime requirements: Windows 10 1803+ / Windows 11 (WebView2 is built in).

## Building from source

### Prerequisites

| Tool | Version | Notes |
| --- | --- | --- |
| Node.js | 18+ | Frontend build (required by Vite 6) |
| Rust | stable | Install via [rustup](https://rustup.rs) |
| VS Build Tools 2022 | — | Select the "Desktop development with C++" workload (MSVC v143 + Windows SDK); a hard requirement for compiling Rust on Windows |
| WebView2 Runtime | — | Built into Win11 / up-to-date Win10 |

> The first Rust build downloads and compiles all dependencies and takes several minutes; `src-tauri/target/` grows to about 7.5GB. This is normal (and git-ignored).

### Development

```powershell
npm install
npm run tauri dev
```

### Production build

```powershell
npm run tauri build
```

Output:

- `src-tauri\target\release\folder-manager.exe`
- `src-tauri\target\release\bundle\msi\Folder Manager_1.0.0_x64_en-US.msi`

### Packaging the source

```powershell
.\pack-7z.ps1
```

Produces `folder-manager-src.7z` (~200KB) with all build caches excluded — ready to share.

## Project structure

```
├── src/                  # Vue frontend
│   ├── components/       # TitleBar / Sidebar / FolderCard / StatusBar, etc.
│   ├── dialogs/          # Project / folder / confirm dialogs
│   ├── stores/app.ts     # Pinia store + persistence
│   └── types/            # Type definitions
├── src-tauri/            # Rust backend (Tauri commands, config I/O, acrylic window)
├── pack-7z.ps1           # Source packaging script
└── AGENTS.md             # Conventions for AI-assisted development
```

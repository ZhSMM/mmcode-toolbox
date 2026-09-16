# Mavis Code Toolbox

面向开发者的桌面端工具集合。基于 **Rust + Tauri 2 + Vue 3 + TypeScript + SQLite + Element Plus**。

> 📄 完整设计文档见 [`DESIGN.md`](./DESIGN.md)

## 特性

- ⚡️ **本地优先**:所有工具调用与历史均在本地 SQLite,无网络依赖。
- 🧰 **25 个内置工具**:覆盖编解码 / 文本 / 加解密 / 生成器 / 数据 / Web / 网络 7 大类。
- 🕓 **可历史**:每次执行自动留痕,支持一键回填输入。
- ⭐ **可收藏 + 导入导出**:跨设备同步收藏。
- ⌨️ **全局命令面板**:`Cmd/Ctrl+K` 模糊搜索工具和命令。
- 🌓 **主题切换**:跟随系统 / 浅色 / 深色,一键循环。
- 🧩 **易扩展**:新增工具 = 一个 Rust command + 一个 Vue 组件 + 一条菜单常量。

## 内置工具

| 分类 | 工具 |
|---|---|
| **编解码** | JSON 格式化、Base64、URL 编解码、Hex、Base32/Base58 |
| **文本** | 正则测试、Diff 对比、字符串统计 |
| **加解密** | MD5、SHA-1/256/512、HMAC、AES-256-GCM、JWT 解码 |
| **生成器** | UUID、随机密码、时间戳转换、二维码、随机数/抽样 |
| **数据** | JSON、CSV 工具、SQL 格式化 |
| **Web** | Markdown 预览、颜色转换、Cron 解析 |
| **网络** | CIDR 子网计算、IP 信息 |

## 开发

### 环境要求

- Node.js ≥ 20
- pnpm ≥ 9(或 npm / yarn)
- Rust stable([rustup](https://rustup.rs/))
- Tauri 2 系统依赖:见 [Tauri 官方文档](https://tauri.app/start/prerequisites/)

### 安装与启动

```bash
pnpm install
pnpm tauri:dev
```

首次启动会:

1. 在 OS 应用数据目录创建 `app.db`(Windows: `%APPDATA%\com.minimax.mmcode-toolbox\app.db`)
2. 执行迁移并 seed 25 条工具元数据
3. 弹出桌面窗口

### 生产构建

```bash
pnpm tauri:build
```

产物在 `src-tauri/target/release/bundle/` 下。

## 目录速览

```
src/                    Vue3 SPA (TypeScript)
src-tauri/src/          Rust 后端
src-tauri/migrations/   数据库迁移 SQL
.github/workflows/      CI + Release(自动构建三平台)
DESIGN.md               设计文档
```

## 快捷键

- `Cmd/Ctrl + K` — 全局命令面板
- 顶部主题按钮 — 循环切换 跟随 / 浅 / 深

## 后续扩展(按 DESIGN.md §12)

1. Rust 端:`src-tauri/src/commands/<tool>.rs` 实现纯函数 + `#[tauri::command]`
2. 在 `commands/mod.rs` 注册,在 `lib.rs` 的 `generate_handler!` 数组登记
3. 在 `commands/tools.rs` 的 `TOOLS` 常量数组加一条
4. 前端:`src/api/<tool>.ts` 封装 invoke
5. 在 `src/router/index.ts` 加一条路由
6. 在 `src/components/tools/<Tool>.vue` 复用 `ToolPanel.vue`

## CI / Release

- `push` 到 `master`/`main` 触发 `.github/workflows/ci.yml`(cargo check + test + 前端 type-check + build)
- 推送 `v*` tag 触发 `.github/workflows/release.yml`,自动构建 Windows / macOS(arm64+x86) / Linux 包并发布 draft Release
# Mavis Code Toolbox

面向开发者的桌面端工具集合。基于 **Rust + Tauri 2 + Vue3 + TypeScript + SQLite + Element Plus**。

> 📄 完整设计文档见 [`DESIGN.md`](./DESIGN.md)。

## 特性

- ⚡️ **本地优先**：所有工具调用与历史均在本地 SQLite，无网络依赖。
- 🧰 **统一壳**：左侧菜单归类工具，右侧即时使用。
- 🕓 **可历史**：每次执行自动留痕，支持一键回填输入。
- ⭐ **可收藏**：常用工具钉到首页。
- 🧩 **易扩展**：新增工具 = 一个 Rust command + 一个 Vue 组件 + 一条菜单常量。

## 内置工具

| 大类 | 工具 | 状态 |
|---|---|---|
| 编解码 | JSON 格式化、Base64、URL 编解码、Hex | JSON / Base64 已实现，其余占位 |
| 文本 | 正则测试、Diff 对比、字符串统计 | 占位 |
| 加解密 | MD5、SHA、HMAC | 占位 |
| 生成器 | UUID、随机密码、时间戳、二维码 | 占位 |

## 开发

### 环境要求

- Node.js ≥ 20
- pnpm ≥ 9（也可用 npm / yarn）
- Rust stable（[rustup](https://rustup.rs/)）
- Tauri 2 系统依赖：见 [Tauri 官方文档](https://tauri.app/start/prerequisites/)

### 安装与启动

```bash
pnpm install
pnpm tauri:dev
```

首次启动会：

1. 拉取 Rust 依赖（首次较慢）
2. 在 OS 应用数据目录创建 `app.db`
3. 执行迁移并 seed 14 条工具元数据
4. 弹出桌面窗口

### 生产构建

```bash
pnpm tauri:build
```

产物在 `src-tauri/target/release/bundle/` 下，按平台生成 `.msi / .dmg / .AppImage / .deb`。

## 目录速览

```
src/                    Vue3 SPA
src-tauri/src/          Rust 后端
src-tauri/migrations/   数据库迁移 SQL
DESIGN.md               设计文档
```

## 后续扩展

详见 `DESIGN.md` §12。简言之：写一个 Rust 纯函数 → 加进 `TOOLS` 常量 → 写一个 Vue 组件复用 `ToolPanel` → 注册路由。
# Mavis Code Toolbox — 设计文档

> 面向开发者的桌面端工具集合。基于 **Rust + Tauri 2 + Vue3 + TypeScript + SQLite + Element Plus**。

---

## 1. 目标与定位

一个**常驻桌面**的轻量工具箱，左侧菜单归类工具，右侧即时调用，覆盖开发日常的高频小操作。
核心原则：

- **零启动延迟**：本地工具即开即用，不走网络。
- **隐私优先**：所有数据落地本地 SQLite，不上传。
- **可扩展**：新增一个工具 = 新增一个 Rust command + 一个 Vue 组件 + 一条菜单注册项。
- **可历史**：每次执行自动留痕，支持回填输入。

---

## 2. 技术栈

| 层 | 选型 | 版本 | 用途 |
|---|---|---|---|
| 壳 | Tauri | 2.x | 桌面运行时 + Rust ↔ WebView 桥 |
| 前端框架 | Vue | 3.5+ | SPA |
| 前端语言 | TypeScript | 5.6+ | 类型安全 |
| 构建工具 | Vite | 5.x | 开发服务器与生产打包 |
| UI 库 | Element Plus | 2.8+ | 组件库 |
| 路由 | Vue Router | 4.x | 工具页路由 |
| 状态 | Pinia | 2.x | 全局菜单 / 收藏 / 历史状态 |
| 数据库 | SQLite (rusqlite, bundled) | latest | 工具元数据 / 历史 / 收藏 / 配置 |
| 序列化 | serde / serde_json | latest | Rust ↔ JS 通信 |
| 错误 | thiserror | latest | Rust 统一错误类型 |

---

## 3. 整体架构

```
┌────────────────────────────────────────────────────────┐
│                WebView (Chromium)                       │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Vue3 SPA                                          │  │
│  │  ┌────────────┐  ┌────────────────────────────┐  │  │
│  │  │ SideMenu   │  │ ToolView (router-view)     │  │  │
│  │  │ (ElMenu)   │→ │  ├─ JsonFormatter           │  │  │
│  │  └────────────┘  │  ├─ Base64Tool              │  │  │
│  │                  │  └─ PlaceholderTool ×N      │  │  │
│  │  Pinia stores  ◀─│  │                            │  │  │
│  │  api/*.ts      ──┼─▶ tauri.invoke()  ──┐        │  │  │
│  │                  │  └─ HistoryDrawer    │        │  │  │
│  └───────────────────────────────────────│────────┘  │  │
│                                          │ IPC       │
├──────────────────────────────────────────│──────────┤
│                Rust Process              ▼           │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Tauri Commands  (#[tauri::command])               │  │
│  │   ├─ list_tools / list_categories                  │  │
│  │   ├─ save_history / list_history / clear_history   │  │
│  │   ├─ toggle_favorite / list_favorites             │  │
│  │   ├─ json_format                                   │  │
│  │   └─ base64_encode / base64_decode                 │  │
│  └──────────────────────┬────────────────────────────┘  │
│                         ▼                               │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Service Layer                                     │  │
│  │   ├─ tools (元数据常量)                            │  │
│  │   ├─ json_fmt / base64 (纯函数)                     │  │
│  │   └─ db (rusqlite 连接 + migrate)                 │  │
│  └──────────────────────┬────────────────────────────┘  │
│                         ▼                               │
│                  SQLite (app.db)                        │
└────────────────────────────────────────────────────────┘
```

要点：

1. **菜单元数据用 Rust 常量**，避免每次启动都查表。前端调用 `list_tools()` 拿到菜单树渲染侧边栏。
2. **工具执行 = 纯 Rust 函数 + 一条历史记录**。前端拿到结果后只负责展示，历史由后端统一写入。
3. **错误统一收敛**：`AppError` 通过 `serde` 序列化为 `{ kind, message }`，前端可识别并友好提示。

---

## 4. 目录结构

```
mmcode/
├── DESIGN.md                       # 本文档
├── README.md
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tsconfig.node.json
├── index.html
├── .gitignore
│
├── src/                            # Vue3 前端
│   ├── main.ts                     # 入口：创建 app、注册 Pinia/Router、挂载 Element Plus
│   ├── App.vue                     # 根组件
│   ├── env.d.ts
│   ├── router/index.ts             # 路由表
│   ├── stores/
│   │   └── app.ts                  # 菜单 / 收藏 / 当前工具
│   ├── api/                        # invoke 封装层
│   │   ├── index.ts                # 通用 invoke + 错误归一化
│   │   ├── tools.ts                # list_tools / list_categories
│   │   ├── history.ts              # 历史 CRUD
│   │   ├── favorites.ts            # 收藏
│   │   ├── json.ts                 # json_format
│   │   └── base64.ts               # base64_encode / base64_decode
│   ├── types/
│   │   └── index.ts                # 与 Rust 对齐的类型
│   ├── utils/
│   │   └── format.ts               # 时间 / 截断 / 复制等通用函数
│   ├── components/
│   │   ├── layout/
│   │   │   ├── MainLayout.vue      # 侧边栏 + 内容主框架
│   │   │   └── SideMenu.vue        # Element Plus ElMenu 封装
│   │   ├── common/
│   │   │   ├── ToolPanel.vue       # 工具页通用外壳：标题 + 输入/输出 + 历史侧栏
│   │   │   ├── HistoryDrawer.vue   # 历史抽屉
│   │   │   └── PlaceholderTool.vue # 待实现工具占位
│   │   └── tools/
│   │       ├── JsonFormatter.vue   # 工具 1：JSON 格式化
│   │       └── Base64Tool.vue      # 工具 2：Base64 编解码
│   ├── views/
│   │   ├── Home.vue                # 首页：欢迎 + 最近使用 + 收藏
│   │   └── Settings.vue            # 设置：数据目录 / 清空历史 / 关于
│   └── styles/
│       └── global.css
│
└── src-tauri/                      # Rust 后端
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json
    ├── capabilities/
    │   └── default.json            # Tauri 2 权限声明
    ├── icons/                      # 应用图标（占位，由 build 脚本准备）
    ├── migrations/
    │   └── 001_init.sql            # 建表 SQL
    └── src/
        ├── main.rs                 # 入口
        ├── lib.rs                  # run() 库入口
        ├── error.rs                # AppError
        ├── state.rs                # AppState (db pool)
        ├── db/
        │   ├── mod.rs
        │   ├── pool.rs             # Mutex<Connection>
        │   └── migrate.rs          # 启动时执行 migrations/*.sql
        └── commands/
            ├── mod.rs              # 命令注册
            ├── tools.rs            # list_tools / list_categories
            ├── history.rs          # 历史 CRUD
            ├── favorites.rs        # 收藏
            ├── json_fmt.rs         # JSON 格式化
            └── base64.rs           # Base64 编解码
```

---

## 5. 菜单设计

左侧菜单用 Element Plus 的 `ElMenu`（折叠/手风琴模式），按 **大类 → 工具** 两级组织。

| 大类 (Category) | 工具 (Tool) | tool_id | 路由 | 默认实现 |
|---|---|---|---|---|
| 编解码 (encode) | JSON 格式化 | `json-formatter` | `/tools/json-formatter` | ✅ 完整 |
|  | Base64 编解码 | `base64` | `/tools/base64` | ✅ 完整 |
|  | URL 编解码 | `url-codec` | `/tools/url-codec` | 🟡 占位 |
|  | Hex 转换 | `hex` | `/tools/hex` | 🟡 占位 |
| 文本 (text) | 正则测试 | `regex-tester` | `/tools/regex-tester` | 🟡 占位 |
|  | Diff 对比 | `diff` | `/tools/diff` | 🟡 占位 |
|  | 字符串统计 | `string-stats` | `/tools/string-stats` | 🟡 占位 |
| 加解密 (crypto) | MD5 | `md5` | `/tools/md5` | 🟡 占位 |
|  | SHA-1/256 | `sha` | `/tools/sha` | 🟡 占位 |
|  | HMAC | `hmac` | `/tools/hmac` | 🟡 占位 |
| 生成器 (generator) | UUID 生成 | `uuid` | `/tools/uuid` | 🟡 占位 |
|  | 随机密码 | `password` | `/tools/password` | 🟡 占位 |
|  | 时间戳转换 | `timestamp` | `/tools/timestamp` | 🟡 占位 |
|  | 二维码生成 | `qrcode` | `/tools/qrcode` | 🟡 占位 |

**菜单元数据** 在 Rust 端以常量数组定义（`src-tauri/src/commands/tools.rs`），前端通过 `list_tools()` 拉取，避免散落在 Vue 里。

```rust
pub static TOOLS: &[ToolMeta] = &[
    ToolMeta { tool_id: "json-formatter", name: "JSON 格式化", category: "encode", route: "/tools/json-formatter", icon: "Document", sort: 10, enabled: true },
    // ...
];

pub static CATEGORIES: &[Category] = &[
    Category { id: "encode",    name: "编解码",  icon: "Coin",    sort: 1 },
    Category { id: "text",      name: "文本",    icon: "EditPen", sort: 2 },
    Category { id: "crypto",    name: "加解密",  icon: "Lock",    sort: 3 },
    Category { id: "generator", name: "生成器",  icon: "MagicStick", sort: 4 },
];
```

后续新增工具只需改这个常量 + 新建一个 Vue 组件 + 注册一条路由。

---

## 6. 数据库设计

SQLite 文件位于 OS 标准应用数据目录（macOS: `~/Library/Application Support/<bundleId>/app.db`，Windows: `%APPDATA%/<bundleId>/app.db`，Linux: `~/.local/share/<bundleId>/app.db`），由 Tauri 的 path API 解析。

### 6.1 表结构（`migrations/001_init.sql`）

```sql
-- 工具元数据（菜单用）。预置 14 条，启动时 INSERT OR IGNORE 同步常量表。
CREATE TABLE IF NOT EXISTS tools (
    tool_id     TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    category    TEXT NOT NULL,
    route       TEXT NOT NULL,
    icon        TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    enabled     INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_tools_category ON tools(category);

-- 历史记录。每次工具执行成功/失败都留痕。
CREATE TABLE IF NOT EXISTS history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tool_id     TEXT NOT NULL,
    input       TEXT,                 -- 截断到 16KB
    output      TEXT,                 -- 截断到 64KB
    status      TEXT NOT NULL DEFAULT 'success',  -- success | error
    error_msg   TEXT,
    created_at  INTEGER NOT NULL      -- 毫秒时间戳
);
CREATE INDEX IF NOT EXISTS idx_history_tool ON history(tool_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_history_time ON history(created_at DESC);

-- 收藏的工具 tool_id。
CREATE TABLE IF NOT EXISTS favorites (
    tool_id     TEXT PRIMARY KEY,
    created_at  INTEGER NOT NULL
);

-- 应用配置（KV）。
CREATE TABLE IF NOT EXISTS app_settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  INTEGER NOT NULL
);
```

### 6.2 设计要点

- **截断策略**：`input` 截断到 16KB、`output` 截断到 64KB，避免巨型文本撑爆 SQLite。截断在 Rust 写入前完成，前端不感知。
- **软删除**：历史只支持硬删除 + 按 tool 清空，不做软删除，避免索引膨胀。
- **收藏只存 tool_id**：收藏列表是一个 Set，前端按 tool_id 渲染。
- **KV 配置**：用 `app_settings` 存"是否自动清空 > N 天的历史"等可调项，预留扩展。

---

## 7. IPC 契约

所有命令统一返回 `Result<T, AppError>`。错误序列化形态：

```rust
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("database: {0}")]        Db(String),
    #[error("io: {0}")]              Io(String),
    #[error("invalid input: {0}")]   Invalid(String),
    #[error("internal: {0}")]        Internal(String),
}
```

前端 invoke 包装：

```ts
// src/api/index.ts
export async function invokeCmd<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    // e = { kind: 'invalid input: xxx', message: 'xxx' }
    const err = e as { kind?: string; message?: string };
    throw new Error(err.message ?? err.kind ?? String(e));
  }
}
```

### 7.1 命令清单

| Command | 入参 | 返回 | 说明 |
|---|---|---|---|
| `list_categories` | – | `Category[]` | 菜单大类 |
| `list_tools` | – | `ToolMeta[]` | 全部工具 |
| `list_history` | `{ toolId: string, limit: number }` | `HistoryItem[]` | 单工具最近 N 条 |
| `save_history` | `{ toolId, input, output, status, errorMsg? }` | `i64` | 新增并返回 id |
| `delete_history` | `{ id: number }` | `()` | 单条删除 |
| `clear_history` | `{ toolId?: string }` | `()` | 清空（不传 = 清全部） |
| `list_favorites` | – | `string[]` | 收藏 tool_id 列表 |
| `toggle_favorite` | `{ toolId: string }` | `boolean` | 切换，返回切换后状态 |
| `json_format` | `{ input: string, indent: number, sort_keys: bool, escape_unicode: bool }` | `{ output: string, duration_ms: number }` | JSON 格式化 |
| `json_minify` | 同上 | 同上 | JSON 压缩 |
| `base64_encode` | `{ input: string, url_safe: bool }` | `{ output: string }` | Base64 编码 |
| `base64_decode` | `{ input: string, url_safe: bool }` | `{ output: string }` | Base64 解码 |

### 7.2 类型对齐（前后端共用形态）

```ts
// src/types/index.ts
export interface ToolMeta {
  tool_id: string;
  name: string;
  category: string;
  route: string;
  icon: string | null;
  sort_order: number;
  enabled: boolean;
}
export interface Category {
  id: string;
  name: string;
  icon: string | null;
  sort: number;
}
export interface HistoryItem {
  id: number;
  tool_id: string;
  input: string | null;
  output: string | null;
  status: 'success' | 'error';
  error_msg: string | null;
  created_at: number;       // ms
}
```

Rust 端用 `#[derive(Serialize, ts_rs::TS)]` 或手写 serde，确保字段 snake_case 与前端一致。前端 TypeScript 接口作为单一事实源，Rust 通过 `serde(rename_all = "snake_case")` 对齐。

---

## 8. 关键流程

### 8.1 启动序列

```
main.rs
  ├─ tauri::Builder::default()
  │   ├─ .setup(|app| {
  │   │     let db_path = app.path().app_data_dir()?.join("app.db");
  │   │     let conn = open_db(&db_path)?;
  │   │     migrate::run(&conn)?;          // 执行 migrations/*.sql
  │   │     seed_tools(&conn)?;            // INSERT OR IGNORE 常量 TOOLS
  │   │     app.manage(AppState { db: Mutex::new(conn) });
  │   │     Ok(())
  │   │   })
  │   ├─ .invoke_handler(tauri::generate_handler![
  │   │     commands::list_tools,
  │   │     commands::list_categories,
  │   │     ...
  │   │   ])
  │   └─ .run(tauri::generate_context!())
```

### 8.2 菜单加载

1. Vue `main.ts` 创建应用、挂载。
2. `App.vue` `onMounted` → `appStore.loadMenu()`。
3. `loadMenu` 并行调用 `list_categories()` + `list_tools()`，写入 Pinia store。
4. `SideMenu.vue` 根据 store 渲染分组 + 子项。

### 8.3 工具执行（以 JSON 格式化为例）

```
[JsonFormatter.vue]
  user clicks "格式化"
    → invoke('json_format', { input, indent, sort_keys, escape_unicode })
    → Rust: serde_json::from_str → to_string_pretty
    → save_history({ tool_id, input, output, status: 'success' })
    → return { output, duration_ms }
  渲染输出面板 + 通知"已记录到历史"
  HistoryDrawer 重新拉取该工具最近 10 条
```

失败时 `status='error'`，把 `error_msg` 写入历史，前端用 `ElNotification` 提示。

### 8.4 历史回填

HistoryDrawer 列出某工具历史。点击某条 → 把 `input` 写回主输入区 → 自动重跑 → 覆盖原历史条目的 status 与 output（视为"再次使用"）。

### 8.5 收藏切换

工具页右上角 ☆ 按钮 → `toggle_favorite(tool_id)` → 返回最新状态 → Pinia store 更新 → 首页"我的收藏"区块实时刷新。

---

## 9. 关键模块接口

### 9.1 Rust 工具层（纯函数）

```rust
// src-tauri/src/commands/json_fmt.rs
pub fn format_json(input: &str, indent: usize, sort_keys: bool, escape_unicode: bool) -> Result<String, AppError> {
    let mut value: serde_json::Value = serde_json::from_str(input).map_err(|e| AppError::Invalid(e.to_string()))?;
    if sort_keys { sort_json_keys(&mut value); }
    let formatter = if escape_unicode { serde_json::ser::PrettyFormatter::with_indent(b"    ".repeat(indent).as_slice()) } else { ... };
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    use serde::Serialize;
    value.serialize(&mut ser).map_err(|e| AppError::Internal(e.to_string()))?;
    String::from_utf8(buf).map_err(|e| AppError::Internal(e.to_string()))
}
```

```rust
// src-tauri/src/commands/base64.rs
pub fn encode(input: &str, url_safe: bool) -> String {
    let engine = if url_safe { base64::engine::general_purpose::URL_SAFE_NO_PAD } else { base64::engine::general_purpose::STANDARD };
    base64::Engine::encode(&engine, input.as_bytes())
}
pub fn decode(input: &str, url_safe: bool) -> Result<String, AppError> {
    let engine = if url_safe { base64::engine::general_purpose::URL_SAFE_NO_PAD } else { base64::engine::general_purpose::STANDARD };
    let bytes = base64::Engine::decode(&engine, input).map_err(|e| AppError::Invalid(e.to_string()))?;
    String::from_utf8(bytes).map_err(|e| AppError::Invalid(e.to_string()))
}
```

### 9.2 前端 ToolPanel 契约

```vue
<!-- ToolPanel.vue：所有工具页统一外壳 -->
<template>
  <div class="tool-panel">
    <header class="tool-panel__head">
      <div>
        <h2>{{ title }}</h2>
        <p class="desc">{{ description }}</p>
      </div>
      <div class="actions">
        <el-button @click="historyVisible = true">
          历史 <el-badge :value="historyCount" />
        </el-button>
        <el-button :icon="favorited ? StarFilled : Star" @click="onToggleFav" />
      </div>
    </header>
    <slot />
    <HistoryDrawer v-model:visible="historyVisible" :tool-id="toolId" @pick="onPickHistory" />
  </div>
</template>
```

每个工具组件只关心自己的输入/输出 UI，工具面板统一负责标题、收藏、历史。

---

## 10. 错误处理与可观测

- **统一错误**：所有 command 返回 `Result<T, AppError>`，前端归一化为 `Error`，UI 用 `ElMessage.error` 展示。
- **历史记录** 失败也算历史：保留 `status='error'` 与 `error_msg`，方便用户复盘"为什么这次又错了"。
- **日志**：Rust 端用 `log` + `env_logger`，Tauri 启动时初始化；前端 console 镜像关键 IPC 调用与耗时。

---

## 11. 安全与权限

- Tauri 2 capabilities：默认只允许 `core:default` + 自家 commands。所有 command 必须显式在 `capabilities/default.json` 中允许。
- 前端不直接访问文件系统；所有 I/O 走 Rust command。
- 输入限制：单个工具输入上限 1MB（前端校验 + Rust 兜底），超出返回 `AppError::Invalid`。
- SQL 全参数化，禁止字符串拼接。

---

## 12. 后续扩展指引

新增一个工具的标准流程（10 分钟）：

1. **Rust 命令**：`src-tauri/src/commands/<tool>.rs` 实现纯函数 + `#[tauri::command]` 包装。
2. **注册命令**：`src-tauri/src/commands/mod.rs` 加进 `tauri::generate_handler!`。
3. **加入常量**：`commands/tools.rs` 的 `TOOLS` 数组加一条。
4. **前端 API**：`src/api/<tool>.ts` 封装 invoke。
5. **路由**：`src/router/index.ts` 加一条。
6. **Vue 组件**：`src/components/tools/<Tool>.vue` 复用 `ToolPanel.vue`。
7. **可选**：在 views/Home.vue 加快捷入口卡片。

---

## 13. 当前交付范围（本次脚手架）

- ✅ 完整工程脚手架（可直接 `pnpm tauri dev`）
- ✅ SQLite 迁移 + 元数据 seed
- ✅ 4 大类 / 14 个工具的菜单（其中 12 个为占位）
- ✅ 完整实现 **JSON 格式化**（格式化 + 压缩 + 排序键 + 转 Unicode）
- ✅ 完整实现 **Base64 编解码**（标准 / URL-safe）
- ✅ 历史记录 + 收藏 + 设置页
- 其余 12 个工具为 `PlaceholderTool.vue` 占位（提示"待实现"）

后续按 §12 的步骤继续填充即可。
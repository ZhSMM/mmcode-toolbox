# Mavis Code Toolbox — 设计文档

> 面向开发者的桌面端工具集合。基于 **Rust + Tauri 2 + Vue 3 + TypeScript + SQLite + Element Plus**。
>
> 仓库: https://github.com/ZhSMM/mmcode-toolbox

---

## 1. 目标与定位

一个**常驻桌面**的轻量工具箱,左侧菜单归类工具,右侧即时调用,覆盖开发日常的高频小操作。
核心原则:

- **零启动延迟**:本地工具即开即用,不走网络。
- **隐私优先**:所有数据落地本地 SQLite,不上传。
- **可扩展**:新增工具 = 一条 `TOOLS` 常量 + 一个 Rust command + 一个 Vue 组件 + 一条路由。
- **可历史 + 可收藏**:每次执行留痕,常用工具钉到首页。
- **快捷**:全局命令面板 `Cmd/Ctrl+K` 一步到位。

---

## 2. 技术栈

| 层 | 选型 | 版本 |
|---|---|---|
| 壳 | Tauri | 2.x |
| 前端 | Vue + TypeScript + Vite + Pinia + Vue Router + Element Plus | 3.5 / 5.6 / 5 / 2 / 4 / 2.8 |
| 后端 | Rust + rusqlite (bundled) | stable / 0.32 |
| 数据 | SQLite (本地,app data 目录) | latest |
| 编解码 / 散列 | percent-encoding / hex / md-5 / sha1 / sha2 / hmac / aes-gcm | 各 latest |
| 生成器 | uuid (v4/v7) / rand / qrcode | 1 / 0.8 / 0.14 |
| 数据 | csv / pulldown-cmark / sqlformat | 1 / 0.10 / 0.2 |
| Web 编码 | data-encoding (Base32) / bs58 (Base58) | 2 / 0.5 |
| 网络 | ipnetwork | 0.20 |
| CI | GitHub Actions(tauri-action + release-please) | - |

---

## 3. 整体架构

```
┌────────────────────────────────────────────────────────┐
│                  WebView (Chromium)                    │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Vue3 SPA                                         │  │
│  │  ┌────────────┐  ┌───────────────────────────┐  │  │
│  │  │ SideMenu   │  │ ToolView (router-view)     │  │  │
│  │  │ (ElMenu)   │→ │  ├─ JsonFormatter           │  │  │
│  │  │            │  │  ├─ Base64Tool              │  │  │
│  │  │            │  │  ├─ AesTool / JwtTool       │  │  │
│  │  │            │  │  ├─ ... 25 个               │  │  │
│  │  │            │  │  └─ CommonToolPanel shell   │  │  │
│  │  │ Cmd+K ⌘    │  │  + HistoryDrawer            │  │  │
│  │  └────────────┘  └───────────────────────────┘  │  │
│  │  Pinia stores  ◀─ API layer (invoke wrapper) ──┐  │  │
│  └─────────────────────────────────────────────────│──┘  │
│                                                    │    │
├────────────────────────────────────────────────────│────┤
│                Rust Process                        ▼    │
│  ┌──────────────────────────────────────────────────┐   │
│  │  #[tauri::command] functions                      │   │
│  │   ├─ list_tools / list_categories                 │   │
│  │   ├─ save_history / list_history / clear_history  │   │
│  │   ├─ toggle_favorite / list_favorites             │   │
│  │   ├─ json_format / base64_* / aes_* / jwt_decode  │   │
│  │   ├─ md5 / sha / hmac / uuid / password           │   │
│  │   ├─ qrcode / timestamp / random                  │   │
│  │   ├─ regex / diff / string_stats / url_codec      │   │
│  │   ├─ hex / markdown / csv / color / cron / ...    │   │
│  │   └─ cidr / ip_info / sql_format / base_n         │   │
│  └──────────────────────┬───────────────────────────┘   │
│                         ▼                               │
│            SQLite (app_data_dir/app.db)                  │
└────────────────────────────────────────────────────────┘
```

---

## 4. 目录结构

```
mmcode/
├── DESIGN.md                       # 本文档
├── README.md
├── package.json
├── vite.config.ts
├── tsconfig.json
├── index.html
├── .github/workflows/              # CI + 自动 Release
│   ├── ci.yml
│   └── release.yml
└── src/                            # Vue3 前端
    ├── main.ts
    ├── App.vue
    ├── router/index.ts             # 25 条工具路由
    ├── stores/app.ts
    ├── api/                        # invoke 封装(25 工具)
    ├── types/index.ts
    ├── utils/format.ts
    ├── components/
    │   ├── layout/MainLayout.vue + SideMenu.vue
    │   ├── common/                 # ToolPanel / HistoryDrawer / CommandPalette / PlaceholderTool
    │   └── tools/                  # 25 个工具组件
    ├── views/Home.vue + Settings.vue
    └── styles/global.css           # 含 light/dark 主题变量

src-tauri/                          # Rust 后端
├── Cargo.toml
├── tauri.conf.json
├── capabilities/default.json
├── migrations/001_init.sql
└── src/
    ├── main.rs / lib.rs
    ├── error.rs / state.rs
    ├── db/                        # open_and_migrate + seed_tools
    └── commands/                  # 每个工具一个文件
        ├── mod.rs / tools.rs       # 菜单元数据 + 列表命令
        ├── history.rs / favorites.rs
        ├── json_fmt.rs / base64.rs / aes.rs / jwt.rs
        ├── md5.rs / sha.rs / hmac.rs / uuid.rs / password.rs
        ├── qrcode.rs / timestamp.rs / random.rs
        ├── regex.rs / diff.rs / string_stats.rs / url_codec.rs / hex.rs
        ├── markdown.rs / csv_viewer.rs / color.rs / cron.rs
        └── sql_format.rs / base_n.rs / cidr.rs
```

---

## 5. 工具菜单(25 项 / 7 类)

| 大类 | 工具 (tool_id) |
|---|---|
| **编解码** (encode) | JSON 格式化 · Base64 · URL 编解码 · Hex · Base32/Base58 |
| **文本** (text) | 正则测试 · Diff 对比 · 字符串统计 |
| **加解密** (crypto) | MD5 · SHA-1/256/512 · HMAC · AES-256-GCM · JWT 解码 |
| **生成器** (generator) | UUID(v4/v7/nil) · 随机密码 · 时间戳 · 二维码 · 随机数/抽样 |
| **数据** (data) | JSON · CSV 工具 · SQL 格式化 |
| **Web** (web) | Markdown 预览 · 颜色转换 · Cron 解析 |
| **网络** (net) | CIDR 子网 · IP 信息 |

新菜单字段在 `src-tauri/src/commands/tools.rs` 的 `TOOLS` 常量数组中加一条即可。

---

## 6. 数据库设计

```sql
CREATE TABLE tools (
    tool_id TEXT PRIMARY KEY, name TEXT NOT NULL,
    category TEXT NOT NULL, route TEXT NOT NULL,
    icon TEXT, sort_order INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tool_id TEXT NOT NULL,
    input TEXT, output TEXT,           -- input 截断 16KB,output 64KB
    status TEXT NOT NULL DEFAULT 'success',
    error_msg TEXT, created_at INTEGER NOT NULL
);
CREATE TABLE favorites (
    tool_id TEXT PRIMARY KEY, created_at INTEGER NOT NULL
);
CREATE TABLE app_settings (
    key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at INTEGER NOT NULL
);
```

种子:启动时把 `TOOLS` 常量通过 `INSERT OR IGNORE` 同步进 `tools` 表。

---

## 7. IPC 契约

所有 command 统一返回 `Result<T, AppError>`,`AppError` 序列化为 `{ kind, message }`。
顶层参数 snake_case 自动转 camelCase;结构体内部字段保持 snake_case。

**25+ 条 command 列表(节选)**:

| Command | 入参 | 返回 |
|---|---|---|
| `list_tools` / `list_categories` | – | 菜单 |
| `save_history` / `list_history` / `clear_history` | {tool_id, input/output/status} | 历史 |
| `toggle_favorite` / `list_favorites` | {tool_id} | bool |
| `json_format` / `json_minify` | {input, indent, sort_keys, escape_unicode} | {output, duration_ms} |
| `base64_encode` / `base64_decode` | {input, url_safe} | {output} |
| `aes_encrypt` / `aes_decrypt` / `aes_random_key` | {input, key_hex, nonce_hex?, aad_hex?} | {output} |
| `md5_hash` / `sha_hash` / `hmac_hash` | {input, algorithm} | {hex, base64} |
| `jwt_decode` | {token, verify?, secret?, algorithm?} | {header, payload, signature_hex, verified, error} |
| `regex_test` / `regex_replace` | {pattern, input, flags} | matches / 替换结果 |
| `diff_text` | {left, right} | {hunks, stats} |
| `string_stats` | {input, top_n} | 字符/字节/词频等 |
| `url_encode` / `url_decode` | {input, mode} | {output} |
| `hex_encode` / `hex_decode` / `hex_dump` | {input, mode, uppercase} | {output} |
| `markdown_render` | {input, options} | {html, line_count} |
| `csv_parse` / `csv_extract_column` | {input, delimiter, has_headers} | 表格 + 统计 |
| `color_convert` | {input, from} | {hex, rgb, hsl, luminance, contrast} |
| `cron_next` | {expression, count} | 接下来 N 次 ISO 时间 |
| `sql_format` | {input, indent, uppercase, minify} | {output, line_count} |
| `base32_encode/_decode` / `base58_encode/_decode` | {input, variant?} | {encoded} |
| `cidr_info` / `ip_info` | {input} | {network, mask, hosts, ...} |
| `qrcode_generate` | {input, dark, light} | {svg, modules, version} |
| `uuid_generate` | {count, version, uppercase, with_hyphens} | {items} |
| `password_generate` | {length, count, ...flags} | {passwords, strength, entropy_bits} |
| `timestamp_parse` / `timestamp_to` / `timestamp_now` | {input, unit} | 多种格式 |
| `random_generate` | {mode, min, max, count, items, seed} | {results, description} |

---

## 8. 关键流程

### 启动

```
main → lib.rs
  ├─ env_logger
  ├─ app_data_dir + open_and_migrate (WAL, FK)
  ├─ seed_tools (sync Rust TOOLS → SQLite)
  ├─ app.manage(AppState { db: Mutex<Connection> })
  └─ invoke_handler 注册所有 command
```

### 工具执行

```
用户输入 → 组件 invoke(cmd, req)
  → Rust 纯函数处理
  → save_history(status=success/error)
  → 返回结果 + duration_ms
前端展示 + 刷新历史抽屉
```

### 全局命令面板

```
Cmd/Ctrl+K 触发
  → 模糊搜索 (tools[] + sys[])
  → 上下选择 / Enter 执行
支持命令:跳工具 / 收藏管理 / 切换主题 / 导出收藏 / 刷新菜单
```

---

## 9. 体验增强

- **全局命令面板** (`Cmd/Ctrl+K`):打开后顶部搜索框,过滤项按评分排序;`↑↓` 移动、`Enter` 执行、`Esc` 关闭。
- **主题切换**:顶栏图标循环 跟随系统 / 浅色 / 深色;`localStorage.mmcode-theme` 持久化;CSS 变量驱动,改 html class `theme-light`/`theme-dark`。
- **收藏导入导出**:设置页里导出为 JSON 复制到剪贴板;导入时校验 tool_id,只导入当前存在的项。
- **历史抽屉**:每工具页右上角「历史」按钮,抽屉展示最近 50 条,可一键回填输入。
- **错误历史**:失败的执行也写入历史(`status='error'` + `error_msg`),便于复盘。

---

## 10. 后续扩展(10 分钟一个工具)

1. Rust 端:`src-tauri/src/commands/<tool>.rs` 实现纯函数 + `#[tauri::command]`
2. `commands/mod.rs` 加 `pub mod <tool>;`
3. `lib.rs` 的 `tauri::generate_handler!` 数组加 `commands::<tool>::<fn>`
4. `commands/tools.rs` 的 `TOOLS` 常量加一条
5. 前端:`src/api/<tool>.ts` 封装
6. `src/router/index.ts` 加路由
7. `src/components/tools/<Tool>.vue` 复用 `ToolPanel.vue`

---

## 11. CI / Release

`.github/workflows/ci.yml`:PR + push 时跑 `cargo check/test` + 前端 `type-check/build`。
`.github/workflows/release.yml`:推 `v*` tag 后自动 build 三平台包(msi/exe/dmg/AppImage/deb),发到 draft Release。

---

## 12. 安全

- Tauri 2 capabilities 只放行 `core:default` + 我们用到的 plugin;所有 command 必须显式声明。
- 所有 I/O 走 Rust command,前端不直接访问文件系统。
- 输入上限 1MB(部分 4MB)+ SQL 全部参数化,禁止拼接。
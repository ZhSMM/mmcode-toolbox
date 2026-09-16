# Mavis Code Toolbox — 开发指南

> 本指南面向**扩展本项目**的开发者。阅读后应能独立完成「环境搭建 → 启动 → 新增工具 → 调试 → 测试 → 提交 → 发版」全流程。

---

## 0. TL;DR(60 秒快速跑起来)

```bash
git clone https://github.com/ZhSMM/mmcode-toolbox.git
cd mmcode-toolbox
pnpm install              # 安装前端依赖(约 60s)
pnpm tauri:dev            # 编译 Rust(首次 3-5 分钟)+ 启动 Vite + 弹出窗口
```

> 看到桌面窗口、左侧菜单亮起、点开「JSON 格式化」跑一遍 → 环境 OK。

---

## 1. 环境要求

| 工具 | 最低版本 | 用途 | 安装指引 |
|---|---|---|---|
| Node.js | 20.x | 前端构建 / 依赖管理 | https://nodejs.org |
| pnpm | 9.x | 包管理(快、磁盘省) | `npm install -g pnpm` |
| Rust | stable(1.77+) | Rust 后端编译 | https://rustup.rs |
| Rust target(Windows) | `x86_64-pc-windows-msvc` | Tauri 默认 target | rustup 自动安装 |
| WebView2 | Win10 1803+ 自带 | 渲染前端 | Win10 通常自带 |
| MSVC Build Tools | VS 2022 C++ workload | 链接 Rust Windows 二进制 | https://visualstudio.microsoft.com |

**macOS 用户**:需要 Xcode Command Line Tools(`xcode-select --install`)。
**Linux 用户**:需要 `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`。

---

## 2. 项目结构

```
mmcode/
├── DESIGN.md                 # 设计文档(架构 / 菜单 / DB / IPC)
├── DEVELOPMENT.md            # 你正在看的
├── README.md
├── package.json              # 前端依赖 + scripts
├── vite.config.ts            # Vite 配置(端口 1420 固定给 Tauri)
├── tsconfig.json
├── index.html
├── .gitignore
├── .github/workflows/
│   ├── ci.yml                # PR + push 时跑 cargo test + pnpm build
│   └── release.yml           # 推 v* tag 自动构建三平台 draft Release
├── scripts/
│   └── generate-icons.ps1    # 生成应用图标
│
├── src/                      # 前端(Vue3 + TS)
│   ├── main.ts / App.vue / env.d.ts
│   ├── styles/global.css      # light / dark 主题变量
│   ├── router/index.ts        # 25 条工具路由
│   ├── stores/app.ts          # 菜单 / 收藏 / 当前工具
│   ├── api/                   # invoke 封装(25 个文件,一对一)
│   ├── types/index.ts         # 前后端类型契约
│   ├── utils/
│   │   ├── format.ts          # 时间格式化、复制等通用方法
│   │   └── theme.ts           # 主题应用(localStorage + html class)
│   ├── components/
│   │   ├── layout/
│   │   │   ├── MainLayout.vue # 顶栏 + 主题按钮 + 命令面板入口
│   │   │   └── SideMenu.vue   # 左侧 ElMenu
│   │   ├── common/
│   │   │   ├── ToolPanel.vue  # 工具页通用外壳(标题/收藏/历史/布局)
│   │   │   ├── HistoryDrawer.vue
│   │   │   ├── CommandPalette.vue  # Cmd/Ctrl+K 全局命令面板
│   │   │   └── PlaceholderTool.vue # 占位工具(本项目已无)
│   │   └── tools/              # 25 个工具组件
│   └── views/
│       ├── Home.vue            # 首页(收藏/推荐/全部)
│       └── Settings.vue        # 设置 + 收藏导入导出
│
└── src-tauri/                  # Rust 后端
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── build.rs
    ├── capabilities/default.json
    ├── icons/                  # 应用图标
    ├── migrations/001_init.sql  # 启动建表 SQL
    └── src/
        ├── main.rs / lib.rs    # 入口 + run()
        ├── error.rs            # AppError → { kind, message }
        ├── state.rs            # AppState { db: Mutex<Connection> }
        ├── db/
        │   ├── mod.rs          # open_and_migrate + seed_tools
        │   └── migrate.rs      # 内联 migrations/*.sql 并按序执行
        └── commands/           # 每个工具一个文件
            ├── mod.rs          # 注册所有子模块
            ├── tools.rs        # CATEGORIES + TOOLS 常量 + list_* command
            ├── history.rs / favorites.rs
            ├── json_fmt.rs / base64.rs / aes.rs / jwt.rs
            ├── md5.rs / sha.rs / hmac.rs / uuid.rs / password.rs
            ├── qrcode.rs / timestamp.rs / random.rs
            ├── regex.rs / diff.rs / string_stats.rs / url_codec.rs / hex.rs
            ├── markdown.rs / csv_viewer.rs / color.rs / cron.rs
            └── sql_format.rs / base_n.rs / cidr.rs
```

---

## 3. 启动 / 构建 / 测试

### 3.1 开发态(`pnpm tauri:dev`)

```bash
pnpm tauri:dev
```

这个命令会:
1. 启动 Vite 开发服务器(端口 1420)
2. 用 `cargo run` 启动 Tauri 桌面应用,主窗口加载 `http://127.0.0.1:1420`
3. 前端文件变动 → Vite HMR 自动刷新 WebView
4. Rust 文件变动 → cargo 自动 rebuild 并重启进程

> 想看 Rust 日志:`RUST_LOG=info pnpm tauri:dev`(Linux/macOS)。Windows 下默认会写到 stderr,在 dev 终端可见。

### 3.2 仅前端(无 WebView)

```bash
pnpm dev        # 启动 Vite,浏览器访问 http://127.0.0.1:1420
```

注意:不通过 Tauri 启动时,所有 `invoke()` 调用都会报错(没有 Tauri runtime)。仅用于快速验证 UI 渲染。

### 3.3 类型检查

```bash
pnpm type-check        # vue-tsc --noEmit,严格类型校验
```

### 3.4 Rust 后端单元测试

```bash
cd src-tauri
cargo test --lib                    # 跑所有单元测试
cargo test --lib json_fmt -- --nocapture   # 只跑 json_fmt 测试
cargo test --lib -- --test-threads=1      # 串行跑(默认并行)
```

### 3.5 生产构建

```bash
pnpm tauri:build
```

产物:
- `src-tauri/target/release/bundle/msi/*.msi`(Windows)
- `src-tauri/target/release/bundle/dmg/*.dmg`(macOS)
- `src-tauri/target/release/bundle/deb/*.deb` / `appimage/*.AppImage`(Linux)
- `src-tauri/target/release/bundle/nsis/*.exe`(Windows 另一种打包)

### 3.6 仅构建前端(检查 Vite 产物)

```bash
pnpm build
# 产物在 dist/
```

---

## 4. 数据库与存储

SQLite 文件位置(由 Tauri 的 `app.path().app_data_dir()` 解析):

| OS | 路径 |
|---|---|
| Windows | `%APPDATA%\com.minimax.mmcode-toolbox\app.db` |
| macOS | `~/Library/Application Support/com.minimax.mmcode-toolbox/app.db` |
| Linux | `~/.local/share/com.minimax.mmcode-toolbox/app.db` |

**表结构** 详见 `src-tauri/migrations/001_init.sql` 与 `DESIGN.md` §6。**重置数据**:直接删除 `app.db`(开发态常用)。

> WAL 模式 + NORMAL 同步,写入性能足够日常使用;`foreign_keys = ON` 保证引用完整性。

---

## 5. **核心示例:添加一个新工具**(端到端)

我们以添加**「URL Parser」(URL 解析)** 工具为例,从 0 走到完。

### 5.1 设计 5 分钟

- **tool_id**: `url-parser`(kebab-case,唯一)
- **name**:「URL 解析」
- **category**: web(已有)
- **route**: `/tools/url-parser`
- **icon**: `Link`
- **功能**: 解析 URL 各组件(scheme / host / port / path / query / fragment),query string 解析为 key-value 表

### 5.2 Rust 端:纯函数 + command

新建 `src-tauri/src/commands/url_parser.rs`:

```rust
//! URL 解析:提取各组件,query string 解析为 key-value。

use serde::Serialize;
use url::Url; // 来自 url crate(已是 Tauri 间接依赖;若显式用,Cargo.toml 添 url = "2")

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct UrlParts {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query: String,
    pub fragment: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub query_pairs: Vec<(String, String)>,
}

#[derive(Debug, Serialize)]
pub struct UrlParseResp {
    pub valid: bool,
    pub error: Option<String>,
    pub parts: Option<UrlParts>,
}

#[tauri::command]
pub fn url_parse(input: String) -> AppResult<UrlParseResp> {
    if input.trim().is_empty() {
        return Err(AppError::Invalid("输入为空".into()));
    }
    let parsed = Url::parse(&input).map_err(|e| AppError::Invalid(format!("URL 解析失败: {e}")))?;
    let query_pairs: Vec<(String, String)> = parsed
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    let parts = UrlParts {
        scheme: parsed.scheme().to_string(),
        host: parsed.host_str().unwrap_or("").to_string(),
        port: parsed.port(),
        path: parsed.path().to_string(),
        query: parsed.query().unwrap_or("").to_string(),
        fragment: parsed.fragment().map(|s| s.to_string()),
        username: parsed.username().map(|s| s.to_string()),
        password: parsed.password().map(|s| s.to_string()),
        query_pairs,
    };
    Ok(UrlParseResp { valid: true, error: None, parts: Some(parts) })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_full() {
        let r = url_parse(
            "https://user:pass@example.com:8080/path/to?x=1&y=hello#section".into(),
        ).unwrap();
        let p = r.parts.unwrap();
        assert_eq!(p.scheme, "https");
        assert_eq!(p.host, "example.com");
        assert_eq!(p.port, Some(8080));
        assert_eq!(p.path, "/path/to");
        assert_eq!(p.fragment.as_deref(), Some("section"));
        assert_eq!(p.username.as_deref(), Some("user"));
        assert_eq!(p.query_pairs, vec![("x".into(), "1".into()), ("y".into(), "hello".into())]);
    }

    #[test]
    fn parse_invalid() {
        assert!(url_parse("not a url".into()).is_err());
    }
}
```

### 5.3 注册到 `commands/mod.rs`

```rust
pub mod url_parser;   // ← 新增
```

### 5.4 注册到 `lib.rs` 的 `generate_handler!`

```rust
.invoke_handler(tauri::generate_handler![
    // ... 已有 ...
    commands::url_parser::url_parse,   // ← 新增
])
```

### 5.5 加入菜单常量 `commands/tools.rs`

```rust
ToolMeta { tool_id: "url-parser".into(), name: "URL 解析".into(),
          category: "web".into(), route: "/tools/url-parser".into(),
          icon: Some("Link".into()), sort_order: 64, enabled: true },
```

> **关键**:`sort_order` 决定该分类下的顺序;与现有 `web` 类中的 markdown(61)、color(62)、cron(63) 续接。

### 5.6 前端 API 封装 `src/api/url-parser.ts`

```ts
import { invokeCmd } from './index';

export interface UrlParts {
  scheme: string;
  host: string;
  port: number | null;
  path: string;
  query: string;
  fragment: string | null;
  username: string | null;
  password: string | null;
  query_pairs: [string, string][];
}

export interface UrlParseResp {
  valid: boolean;
  error: string | null;
  parts: UrlParts | null;
}

export function urlParse(input: string) {
  return invokeCmd<UrlParseResp>('url_parse', { input });
}
```

### 5.7 注册路由 `src/router/index.ts`

```ts
import UrlParserTool from '@/components/tools/UrlParserTool.vue';

// 在 children 数组里加:
{ path: 'tools/url-parser', name: 'url-parser',
  component: UrlParserTool, meta: { title: 'URL 解析' } },
```

### 5.8 Vue 组件 `src/components/tools/UrlParserTool.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/url-parser';
import * as historyApi from '@/api/history';
import { copyText, truncate } from '@/utils/format';

const TOOL_ID = 'url-parser';
const input = ref('https://user:pass@example.com:8080/path/to?x=1&y=hello#section');
const result = ref<api.UrlParseResp | null>(null);
const busy = ref(false);

const run = async () => {
  if (!input.value.trim()) { ElMessage.warning('请输入 URL'); return; }
  busy.value = true;
  try {
    result.value = await api.urlParse(input.value);
    await historyApi.saveHistory({
      tool_id: TOOL_ID, input: input.value, output: JSON.stringify(result.value.parts), status: 'success',
    });
  } catch (e: any) {
    const msg = e?.message ?? String(e);
    ElMessage.error(msg);
    await historyApi.saveHistory({
      tool_id: TOOL_ID, input: truncate(input.value, 8000), output: null, status: 'error', error_msg: msg,
    }).catch(() => undefined);
  } finally { busy.value = false; }
};

const copy = async (s: string) => { await copyText(s); ElMessage.success('已复制'); };
const loadFromHistory = (v: string) => { input.value = v; run(); };
</script>

<template>
  <ToolPanel :tool-id="TOOL_ID" title="URL 解析"
             description="解析 URL 的 scheme/host/port/path/query/fragment 等组件"
             @pick-history="loadFromHistory">
    <template #options>
      <div class="tool-panel__options">
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="run">解析</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>输入 URL</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="input" type="textarea" :rows="22" resize="none" spellcheck="false" />
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>解析结果</span></div>
        <div class="tool-panel__pane-body" style="padding:12px;overflow:auto;">
          <div v-if="!result" style="color:#909399;">点击「解析」</div>
          <el-descriptions v-else-if="result.parts" :column="2" border size="small">
            <el-descriptions-item label="scheme">{{ result.parts.scheme }}</el-descriptions-item>
            <el-descriptions-item label="host">
              {{ result.parts.host }}
              <el-button size="small" text @click="copy(result.parts.host)">复制</el-button>
            </el-descriptions-item>
            <el-descriptions-item label="port">{{ result.parts.port ?? '默认' }}</el-descriptions-item>
            <el-descriptions-item label="path">{{ result.parts.path }}</el-descriptions-item>
            <el-descriptions-item label="query" :span="2">
              <code>{{ result.parts.query }}</code>
            </el-descriptions-item>
            <el-descriptions-item label="fragment">{{ result.parts.fragment ?? '—' }}</el-descriptions-item>
            <el-descriptions-item label="userinfo">{{ result.parts.username }}:{{ result.parts.password }}</el-descriptions-item>
            <el-descriptions-item label="query pairs" :span="2">
              <el-table :data="result.parts.query_pairs" size="small" style="width:100%;">
                <el-table-column prop="0" label="key" />
                <el-table-column prop="1" label="value" />
              </el-table>
            </el-descriptions-item>
          </el-descriptions>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
```

### 5.9 验证

```bash
# Rust 端
cd src-tauri && cargo test --lib url_parser   # 跑新工具的单元测试(应该 2 passed)

# 前端
pnpm type-check                              # 类型检查应通过

# 运行时
pnpm tauri:dev                                # 启动,左侧菜单找到「URL 解析」
```

### 5.10 commit

```bash
git add .
git commit -m "feat: URL 解析工具 (Rust + Vue)"
git push origin master
```

---

## 6. IPC 设计规则(踩坑清单)

### 6.1 顶层参数 vs 结构体内部字段

| 位置 | Tauri 转换规则 | 示例 |
|---|---|---|
| 顶层参数 | **snake_case → camelCase**(Tauri 自动) | Rust `tool_id: String` ↔ JS `{ toolId }` |
| 结构体内部字段 | **保持 snake_case**(serde 默认) | Rust `tool_id: String` ↔ JS `{ tool_id: }` |

**错误案例**:
```rust
#[derive(Deserialize)]
pub struct Req { pub tool_id: Option<String> }
// ❌ JS 发 { toolId: undefined } 解析失败
// ✅ JS 发 { tool_id: undefined } 正常(Option 默认 None)
```

**修复**:Rust 端加 `#[derive(Deserialize)]` + JS 端用 snake_case;或加 `#[serde(rename_all = = "camelCase")]`,二选一。**本项目统一用 snake_case**。

### 6.2 `generate_handler!` 必须用完整路径

```rust
// ❌ pub use tools::list_categories;   // 重导出函数,但 __cmd__ 没重导出
//    generate_handler![commands::list_categories]   // 找不到 __cmd__list_categories

// ✅ commands::tools::list_categories   // 完整路径,宏能找到 __cmd__
```

详见 `src-tauri/src/lib.rs` 的注释。

### 6.3 错误统一格式

所有 command 返回 `AppResult<T>`。前端 `invokeCmd` 把 `{ kind, message }` 归一化为 `ApiError`。**不要**自己 throw 字符串。

### 6.4 `Box<dyn Trait>` 的陷阱

`Rng` trait 因为泛型方法不是 dyn-compatible。**不要** `Box<dyn rand::Rng>`。改用 enum holder(见 `src-tauri/src/commands/random.rs`)。

### 6.5 `serde` 默认值

```rust
#[serde(default)]                // Option / 用 Default
#[serde(default = "fn_name")]     // 自定义函数返回默认值
```

---

## 7. 前端架构规则

### 7.1 工具页统一壳 `ToolPanel`

**不要**绕过 `ToolPanel.vue` 自己写页面布局。它的 `#options` slot 放操作按钮,`#default` slot 放两栏布局(输入/输出)。标题、收藏、历史抽屉都由 `ToolPanel` 统一管理。

### 7.2 历史留痕

每个工具页成功时调 `historyApi.saveHistory`,失败时也写(`status='error' + error_msg`)。这样历史抽屉才有意义。

### 7.3 全局命令面板可用命令

如果想新增系统命令(如「刷新菜单」),在 `CommandPalette.vue` 的 `commands` 计算属性里加一项即可。

### 7.4 主题切换

`applyTheme(mode)` 在 `utils/theme.ts`,操作 `html.classList` 添加 `theme-light` 或 `theme-dark`。CSS 变量在 `styles/global.css` 里覆盖。**不要**在 Vue 组件里硬编码颜色,优先用 `var(--panel-bg)` 等。

---

## 8. 调试技巧

### 8.1 Rust 日志

```bash
RUST_LOG=mmcode_toolbox_lib=debug,tauri=info pnpm tauri:dev
```

代码里加 `log::debug!(...)` 即可。

### 8.2 前端日志

`main.ts` 注册后,浏览器 devtools(F12 → Console)+ Tauri WebView 自动暴露。生产构建默认关闭 devtools。

### 8.3 WebView 远程调试

设置里加 `tauri.conf.json`:
```json
"app": { "windows": [{ "devtools": true }] }
```
启动后右键 → 「检查元素」。

### 8.4 IPC 调用追踪

`src/api/index.ts` 里 `invokeCmd` 在 dev 模式会 `console.debug` 每个 invoke。生产环境静默。

---

## 9. 测试规范

### 9.1 Rust 单元测试

每个 tool 的纯函数应该写测试。模式:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        // 加密 → 解密 应相等
    }
    #[test]
    fn invalid_input() {
        // 非法输入应返回 Err
    }
}
```

### 9.2 前端手动测试

每个工具跑 3 类用例:
1. **正常路径**:典型输入
2. **边界**:空串、超大、特殊字符
3. **错误**:明显非法的输入,确认 UI 友好提示

### 9.3 端到端 CI

推 PR 后,`.github/workflows/ci.yml` 会跑:
- `cargo check --all-targets`
- `cargo test --lib`
- `pnpm type-check`
- `pnpm build`

---

## 10. 提交与发版

### 10.1 提交规范

```
feat: 新增 URL 解析工具
fix: 修复 AES 加密时 nonce 长度校验
docs: 更新 DEVELOPMENT.md
refactor: 抽出 applyTheme 到 utils/theme.ts
test: 为 url_parser 添加单元测试
```

### 10.2 发版流程

```bash
# 1. 在 master 上累积改动
git commit -m "..."
git push origin master

# 2. 打 tag(语义化版本)
git tag v0.3.0
git push origin v0.3.0

# 3. GitHub Actions 自动:
#    - 在 macOS(arm64 + x86)/ Ubuntu / Windows 上 tauri build
#    - 上传 .msi / .dmg / .deb / .AppImage 到 draft Release
#    - 通知 release-please 自动生成 changelog PR

# 4. 在 GitHub UI 上检查 draft Release,补 changelog,正式发布
```

### 10.3 升级数据库(迁移)

如果改了 schema:
1. 新建 `src-tauri/migrations/00X_xxx.sql`
2. 在 `db/migrate.rs` 的 `migrations` 数组里加 `("00X_xxx", include_str!("../../migrations/00X_xxx.sql"))`

**注意**:迁移 SQL 必须幂等(`CREATE TABLE IF NOT EXISTS`)或配合 schema 版本号。当前项目用最简方式(每次执行所有 SQL + IF NOT EXISTS),适合小项目。大型项目推荐 `refinery` crate。

---

## 11. 常见问题

### Q: 启动后窗口空白 / 看不到内容
A: 检查 Vite 是否启动。`pnpm tauri:dev` 输出里应该有 `Local: http://127.0.0.1:1420`。访问该 URL 看是否正常。

### Q: Rust 编译报 "failed to resolve `__cmd__xxx`"
A: 见 §6.2,`pub use` 不会重导出 `__cmd__`,用完整路径。

### Q: 添加图标后 `tauri build` 报错
A: 确认图标文件存在(见 `scripts/generate-icons.ps1`)。macOS 需要 `icon.icns`,Linux 需要 `icon.png`。

### Q: SQLite 写不进去
A: 检查 `app_data_dir` 路径是否有写权限。Windows 看 `%APPDATA%\com.minimax.mmcode-toolbox\`。

### Q: 修改 Rust 后没生效
A: Vite 不会自动重启 Rust 进程,等几秒 cargo 自己 rebuild 后窗口会重启。或手动 `Ctrl+C` 重启 `pnpm tauri:dev`。

### Q: `Box<dyn base64::Engine>` 编译错
A: 换 `Box<dyn Engine<Config = X, DecodeEstimate = Y>>` 或用 concrete 类型。本项目 `base64.rs` 用后者。

### Q: 想加 dev 依赖但用户不需要
A: 用 `Cargo.toml` 的 `[dev-dependencies]` 或 `[features]`。

---

## 12. 发布前 Checklist

- [ ] `cargo check --all-targets` 通过
- [ ] `cargo test --lib` 全过(目前 11/11)
- [ ] `pnpm type-check` 通过
- [ ] `pnpm build` 成功
- [ ] 至少在 5 个工具上跑一遍手动测试
- [ ] README / DESIGN.md / DEVELOPMENT.md 同步更新
- [ ] commit message 符合规范
- [ ] tag 用 `vX.Y.Z` 格式

---

## 13. 进阶话题(待补充)

- **插件系统**:把 `commands/` 模块拆成「核心 + 用户插件」,运行时 `inventory` 动态注册。
- **i18n**:用 `vue-i18n` 抽出所有中文字符串。
- **多窗口**:Tauri 支持多窗口,可做"工具栏常驻 + 主窗口详情"双窗口模式。
- **系统托盘**:常驻后台 + 全局快捷键唤起。
- **自动更新**:集成 `tauri-plugin-updater`,发版后用户自动收到更新通知。

需要做哪个直接说。
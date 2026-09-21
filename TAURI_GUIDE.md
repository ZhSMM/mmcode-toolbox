# Tauri 2 深入开发指南

> 本指南讲透 **Tauri 2** 的工作机制、配置体系、安全模型、性能调优与跨平台细节。配套 [`RUST_GUIDE.md`](./RUST_GUIDE.md)(Rust 语法)与 [`DEVELOPMENT.md`](./DEVELOPMENT.md)(项目工作流)使用。

---

## 目录

- **Part 1 基础**
  1. Tauri 是什么?为什么选它?
  2. 架构总览:Rust 后端 + WebView 前端
  3. Tauri 1 vs Tauri 2:关键差异

- **Part 2 IPC 机制**(核心)
  4. `#[tauri::command]` 函数
  5. `invoke()` 调用与参数映射
  6. `emit()` / `listen()` 事件总线
  7. `Channel`:流式数据通道
  8. 自定义协议与资源加载

- **Part 3 应用生命周期**
  9. Builder 链式配置
  10. `setup()` 与 `Manager`
  11. 全局 `State` 注入
  12. 应用 / 窗口事件

- **Part 4 配置与安全**
  13. `tauri.conf.json` 全字段
  14. Capabilities / Permissions 系统
  15. CSP 与 Origin 限制

- **Part 5 窗口 / 托盘 / 菜单**
  16. 窗口创建与控制
  17. 系统托盘
  18. 菜单栏

- **Part 6 插件生态**
  19. 官方插件一览
  20. 自定义插件
  21. 插件的权限与作用域

- **Part 7 性能与发布**
  22. Bundle 体积优化
  23. 启动速度优化
  24. 跨平台打包
  25. 自动更新

- **Part 8 调试与排错**
  26. DevTools 与日志
  27. 常见错误排查

---

# Part 1 — 基础

## 1. Tauri 是什么?

Tauri 让 **Rust** 进程承载一个**系统原生 WebView**(Windows WebView2 / macOS WKWebView / Linux WebKitGTK),前端用任意 JS 框架(Vue / React / Svelte / Solid)写 UI,前后端通过 **IPC** 通信。

对比 Electron:

| 维度 | Electron | Tauri 2 |
|---|---|---|
| 后端 runtime | Node.js(嵌入 Chromium V8) | **Rust 二进制**(不嵌入 JS runtime) |
| 前端 | Chromium(完整打包 ~150MB) | 系统 WebView(0 嵌入) |
| Bundle 体积 | 50-150MB+ | **3-15MB** |
| 内存占用 | 100-300MB | **30-80MB** |
| 启动速度 | 1-3s | **<500ms** |
| 安全模型 | Node API 全暴露 | **能力系统**(默认零权限) |

**适用场景**:
- ✅ 桌面工具类应用(本项目)
- ✅ 需要原生菜单/托盘/快捷键的效率工具
- ✅ 已有 Web 前端,想包成桌面
- ❌ 需要完整 Node 生态的应用(用 Electron)
- ❌ 强 Web 标准依赖(如 PWA 完整支持)

## 2. 架构总览

```
┌───────────────────────────────────────────────────────────────┐
│                        你的桌面应用                              │
│  ┌────────────────────────────────────────────┐                │
│  │         WebView (Chromium / WebKit)         │                │
│  │  ┌──────────────────────────────────────┐  │                │
│  │  │  你的前端(Vue / React / 任何 JS 框架)  │  │                │
│  │  │                                      │  │                │
│  │  │  import { invoke } from '@tauri-apps/api/core' │        │
│  │  └──────────────┬───────────────────────┘  │                │
│  └─────────────────┼──────────────────────────┘                │
│                    │ IPC                                       │
│  ┌─────────────────┼──────────────────────────┐                │
│  │  Tauri 核心 (Rust 进程)                     │                │
│  │  ┌──────────────────────────┐              │                │
│  │  │  #[tauri::command] 函数  │              │                │
│  │  │  - 业务逻辑              │              │                │
│  │  │  - 状态访问              │              │                │
│  │  └──────────────────────────┘              │                │
│  │  ┌──────────────────────────┐              │                │
│  │  │  插件 (fs, dialog, shell) │              │                │
│  │  └──────────────────────────┘              │                │
│  └────────────────────────────────────────────┘                │
└───────────────────────────────────────────────────────────────┘
```

## 3. Tauri 1 vs Tauri 2:关键差异

**1. 配置格式变了**:
```jsonc
// Tauri 1
{
  "tauri": { "allowlist": { "fs": { "readFile": true } } }
}

// Tauri 2
{
  "app": { "windows": [...] },
  "plugins": { "fs": { ... } },        // 插件有自己的配置
  "bundle": { ... }
}
// 权限通过 capabilities/ 目录下的 JSON 文件声明
```

**2. API 路径变了**:
```ts
// Tauri 1
import { invoke, event } from '@tauri-apps/api';

// Tauri 2
import { invoke } from '@tauri-apps/api/core';
import { listen, emit } from '@tauri-apps/api/event';
import { Channel } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
```

**3. 插件独立**:
```toml
# Tauri 1
[dependencies]
tauri = { version = "1", features = ["fs-all", "dialog-all"] }

# Tauri 2
[dependencies]
tauri = "2"
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
```

**4. State 注入简化**:
```rust
// Tauri 1
#[tauri::command]
fn cmd(state: State<'_, MyState>) { ... }

// Tauri 2:同名,签名更严格
#[tauri::command]
fn cmd(state: tauri::State<'_, MyState>) -> AppResult<()> { ... }
```

---

# Part 2 — IPC 机制

## 4. `#[tauri::command]` 函数

### 4.1 基础形式

```rust
#[tauri::command]
pub fn greet(name: String) -> String {
    format!("Hello, {name}!")
}
```

前端:
```ts
import { invoke } from '@tauri-apps/api/core';
const msg = await invoke<string>('greet', { name: 'Alice' });
```

### 4.2 参数映射规则

| 位置 | Rust → JS 命名 | 示例 |
|---|---|---|
| 函数参数 | **snake_case → camelCase** | Rust `tool_id: String` ↔ JS `{ toolId: 'x' }` |
| 结构体内部字段 | **保持 snake_case**(serde 默认) | Rust `tool_id: String` ↔ JS `{ tool_id: 'x' }` |
| 函数返回值 | 通过 Serialize 输出 | `AppResult<T>` 序列化为 `T` 或 `{ kind, message }` |

**为什么两层规则不一样?**
- 函数参数是**调用约定**(Tauri 自己的)
- 字段是**数据格式**(serde 的)

```rust
// ✅ 正确用法
#[derive(Deserialize)]
pub struct Req {
    pub tool_id: String,   // JS: { tool_id: 'x' }
}
#[tauri::command]
pub fn list(req: Req, limit: u32) -> AppResult<Vec<T>> {
//              ^^^^^^^^^  snake_case  ^^^^^   camelCase
}
```

### 4.3 异步 command

```rust
#[tauri::command]
pub async fn fetch_data(url: String) -> AppResult<String> {
    let resp = reqwest::get(&url).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .text().await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(resp)
}
```

Tauri 用 tokio 调度 async command。`async fn` 内部可以用 `.await`。

### 4.4 多参数 command

```rust
// ❌ 不允许多参数(老 API)
#[tauri::command]
fn bad(a: String, b: String) { }

// ✅ 用一个 struct 包
#[derive(Deserialize)]
pub struct Req { pub a: String, pub b: String }
#[tauri::command]
fn good(req: Req) -> AppResult<()> { ... }
```

### 4.5 错误处理

```rust
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("db error")] Db(String),
    #[error("invalid input")] Invalid(String),
}

pub type AppResult<T> = std::result::Result<T, AppError>;

#[tauri::command]
pub fn op(input: String) -> AppResult<String> {
    if input.is_empty() {
        return Err(AppError::Invalid("输入为空".into()));
    }
    Ok(input.to_uppercase())
}
```

前端拿到:
```json
{ "kind": "Invalid", "message": "输入为空" }
```

**最佳实践**:错误要**分类**(`Db` / `Invalid` / `Internal`),前端可按 type 决定行为(输入错→弹 toast;内部错→日志上报)。

### 4.6 本项目实际示例

`src-tauri/src/commands/json_fmt.rs`:

```rust
use crate::error::{AppError, AppResult};
use crate::logging::log_and_run;

#[derive(Debug, Deserialize)]
pub struct JsonReq {
    pub input: String,
    #[serde(default = "default_indent")]
    pub indent: usize,
    #[serde(default)]
    pub sort_keys: bool,
}

#[tauri::command]
pub fn json_format(req: JsonReq) -> AppResult<JsonResp> {
    log_and_run("json_format", "json-formatter", || {
        // 业务逻辑
        Ok(JsonResp { output, duration_ms })
    })
}
```

注册到 `lib.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    commands::tools::list_categories,
    commands::json_fmt::json_format,
    // ...
])
```

## 5. `invoke()` 调用与参数映射

### 5.1 前端 invoke

```ts
import { invoke } from '@tauri-apps/api/core';

// 同步
const result = await invoke<T>('cmd_name', { arg1: 'foo', arg2: 42 });

// 错误处理
try {
  await invoke('cmd');
} catch (e: any) {
  // e = { kind, message }
  console.error(e.kind, e.message);
}
```

### 5.2 TypeScript 类型安全

用 [tauri-typegen](https://github.com/ahkohd/tauri-typegen) 或 [tauri-specta](https://github.com/specta-rs/tauri-specta) 自动生成 TS 类型定义。

手写类型的话,定义对应 Rust struct 的 TS interface:

```ts
// src/types/index.ts
export interface JsonReq {
  input: string;
  indent?: number;
  sort_keys?: boolean;
}
export interface JsonResp {
  output: string;
  duration_ms: number;
}
export function jsonFormat(req: JsonReq) {
  return invoke<JsonResp>('json_format', { req });
}
```

### 5.3 大数据传输

```rust
// ❌ 一次性返回 GB 级数据
#[tauri::command]
pub fn get_big_file() -> AppResult<Vec<u8>> {
    Ok(std::fs::read("big.bin")?)  // IPC 序列化 + 反序列化慢
}

// ✅ 用 Channel 流式
#[tauri::command]
pub async fn stream_big_file(path: String, chunk: Channel<Vec<u8>>) -> AppResult<()> {
    use tokio::fs::File;
    use tokio::io::{AsyncReadExt, BufReader};
    let f = File::open(path).await?;
    let mut r = BufReader::new(f);
    let mut buf = vec![0u8; 64 * 1024];
    while let Ok(n) = r.read(&mut buf).await {
        if n == 0 { break; }
        chunk.send(buf[..n].to_vec()).map_err(...)?;
    }
    Ok(())
}
```

前端:
```ts
import { Channel } from '@tauri-apps/api/core';
const channel = new Channel<Uint8Array>();
channel.onmessage = (chunk) => { /* 处理 */ };
await invoke('stream_big_file', { path, chunk });
```

## 6. `emit()` / `listen()` 事件总线

### 6.1 一次性事件(后端 → 前端)

```rust
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn long_task(app: AppHandle) -> AppResult<()> {
    for i in 0..100 {
        app.emit("task-progress", Progress { percent: i, message: "...".into() })
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Ok(())
}
```

前端:
```ts
import { listen, UnlistenFn } from '@tauri-apps/api/event';

let unlisten: UnlistenFn | null = null;
onMounted(async () => {
  unlisten = await listen<Progress>('task-progress', (e) => {
    console.log(`${e.payload.percent}%`);
  });
  await invoke('long_task');
});
onUnmounted(() => unlisten?.());
```

### 6.2 事件协议设计

推荐用 **3 类事件**:

```
task-started     { task_id, name }
task-progress    { task_id, percent, message }   (多次)
task-done        { task_id, success, error? }   (一次)
```

这样前端用 `task_id` 过滤,多个并发任务互不干扰。

### 6.3 前端 → 后端事件(不常见)

```rust
app.emit_to("background", "job-done", payload)?;  // 跨窗口
app.emit("global-event", payload)?;               // 全应用
```

后端不需要 listen 后端事件,后端要响应前端事件用 command(双向通过 invoke + 事件)。

### 6.4 后端→后端事件

```rust
let (tx, _rx) = tokio::sync::broadcast::channel(100);
// 在多个 task 间共享
tx.send(MyEvent::Updated)?;
```

## 7. `Channel`:流式数据通道

详见 §5.3。Channel 是 Tauri 2 新增的,用于流式传递二进制或大量数据。

特点:
- 一次性创建,多次 send
- 接收方 onmessage
- 适合日志流、进度、文件分块

## 8. 自定义协议与资源加载

### 8.1 Asset Protocol(加载本地文件)

```jsonc
// tauri.conf.json
{
  "app": {
    "security": {
      "assetProtocol": {
        "enable": true,
        "scope": ["$APPDATA/**", "$RESOURCE/**"]
      }
    }
  }
}
```

前端:
```html
<img src="asset://localhost/C:/path/to/image.png" />
```

### 8.2 自定义 URI Scheme

```rust
use tauri::http::Request;
use tauri::http::Response;

pub fn register_protocol(app: &mut App) -> Result<(), Box<dyn Error>> {
    app.register_uri_scheme_protocol("myproto", |_app, req: Request<Vec<u8>>| {
        // 处理 myproto://... 请求
        Response::builder()
            .status(200)
            .body(b"Hello".to_vec())
            .unwrap()
    });
    Ok(())
}
```

---

# Part 3 — 应用生命周期

## 9. Builder 链式配置

```rust
use tauri::{Builder, Manager};

fn main() {
    tauri::Builder::default()
        // 1. 加载配置(默认从 tauri.conf.json)
        // 2. 注册插件
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        // 3. 注册 commands
        .invoke_handler(tauri::generate_handler![
            cmd1,
            cmd2,
        ])
        // 4. setup 钩子(应用启动后、窗口创建前)
        .setup(|app| {
            // 初始化数据库、注册全局 state 等
            let conn = db::open_and_migrate(&app.path().app_data_dir()?)?;
            app.manage(AppState { db: Mutex::new(conn) });
            Ok(())
        })
        // 5. 窗口事件
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // 阻止关闭(例如最小化到托盘)
                window.hide().ok();
                api.prevent_close();
            }
        })
        // 6. 启动
        .run(tauri::generate_context!())
        .expect("启动失败");
}
```

## 10. `setup()` 与 `Manager`

`setup` 在所有窗口创建前调用,**唯一可以 app.manage() 的时机**。

```rust
.setup(|app| {
    // app.path() 拿到路径解析器
    let data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    
    // app.manage() 注入全局 state
    app.manage(AppState { db: Mutex::new(conn) });
    
    // app.get_webview_window("main") 拿到窗口(此时窗口已创建)
    let win = app.get_webview_window("main").unwrap();
    
    // app.emit() 发事件
    app.emit("app-ready", ())?;
    
    Ok(())
})
```

## 11. 全局 `State` 注入

```rust
// 定义
pub struct AppState {
    pub db: Mutex<Connection>,
    pub config: Arc<RwLock<Config>>,
}

// 注入
app.manage(AppState { ... });

// 使用
#[tauri::command]
pub fn query(state: tauri::State<'_, AppState>) -> AppResult<Vec<Row>> {
    let conn = state.db.lock().map_err(...)?;
    // ...
}
```

**约束**:`AppState: Send + Sync + 'static`
- 内部字段用 `Mutex<T: Send>` / `RwLock<T: Send + Sync>` / `tokio::sync::Mutex`
- ❌ 不要用 `RefCell<T>`(单线程)

## 12. 应用 / 窗口事件

```rust
use tauri::{Manager, WindowEvent, RunEvent};

.on_window_event(|window, event| {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            window.hide().ok();
            api.prevent_close();
        }
        WindowEvent::Resized(_) => { /* ... */ }
        WindowEvent::Moved(_) => { /* ... */ }
        WindowEvent::Focused(true) => { /* 窗口获焦 */ }
        _ => {}
    }
})
```

应用事件:
```rust
.build(tauri::generate_context!())
.run(|app, event| match event {
    RunEvent::Ready => println!("应用就绪"),
    RunEvent::ExitRequested { .. } => println!("退出中"),
    _ => {}
});
```

---

# Part 4 — 配置与安全

## 13. `tauri.conf.json` 全字段

```jsonc
{
  "$schema": "https://schema.tauri.app/config/2",
  
  // 产品元数据
  "productName": "My App",
  "version": "1.0.0",
  "identifier": "com.example.myapp",   // 反向域名,唯一
  
  // 构建相关
  "build": {
    "beforeDevCommand": "pnpm dev",       // dev 启动前跑
    "devUrl": "http://localhost:3000",    // dev 时前端地址
    "beforeBuildCommand": "pnpm build",   // build 前跑
    "frontendDist": "../dist"             // 生产前端产物路径
  },
  
  // 应用配置
  "app": {
    "windows": [{
      "label": "main",          // 窗口唯一标识
      "title": "My App",
      "width": 1280,
      "height": 800,
      "minWidth": 960, "minHeight": 600,
      "center": true,
      "resizable": true,
      "decorations": true,      // 标题栏
      "transparent": false,
      "fullscreen": false,
      "resizable": true,
      "visible": true,
      "focus": true
    }],
    
    "security": {
      "csp": "default-src 'self'",         // CSP 策略
      "assetProtocol": {
        "enable": false,
        "scope": []                          // 允许访问的路径
      },
      "freezePrototype": false,
      "pattern": { "use": "brownfield" }   // isolation 模式
    },
    
    "withGlobalTauri": false                // 是否暴露 window.__TAURI__
  },
  
  // 打包
  "bundle": {
    "active": true,
    "targets": "all",                       // 或 ["msi", "dmg", "deb", "appimage"]
    "icon": ["icons/32x32.png", "icons/icon.ico"],
    "category": "DeveloperTool",
    "shortDescription": "...",
    "longDescription": "...",
    "resources": [],                        // 额外资源
    "copyright": "...",
    "licenseFile": "../LICENSE"
  },
  
  // 插件配置(每个插件独立)
  "plugins": {}
}
```

## 14. Capabilities / Permissions 系统

Tauri 2 的**零权限安全模型**:前端**默认什么都做不了**。

### 14.1 工作流

```
1. 你的 command 已经在 Rust 端实现,前端可以直接 invoke(自己的 command 不需要 capability)
2. 但调用插件(fs/dialog/...)需要 permission
3. 在 src-tauri/capabilities/*.json 声明
```

### 14.2 本项目配置

`src-tauri/capabilities/default.json`:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "默认能力 - 允许前端调用本应用定义的所有 commands",
  "windows": ["main"],
  "permissions": [
    "core:default",      // 核心能力(窗口/事件等基础)
    "core:window:default",
    "core:webview:default",
    "core:event:default",
    "core:app:default",
    "core:resources:default",
    "core:menu:default",
    "core:tray:default",
    "dialog:default",     // dialog 插件
    "fs:default"          // fs 插件
  ]
}
```

### 14.3 细粒度权限

```json
// capabilities/main-window.json - 限制 main 窗口只能读 /tmp/data
{
  "identifier": "main-restricted",
  "windows": ["main"],
  "permissions": [
    "fs:default",
    {
      "identifier": "fs:allow-read-text-file",
      "allow": [{ "path": "$APPDATA/data/**" }]
    }
  ]
}
```

### 14.4 多窗口不同权限

```json
// 主窗口 - 完整权限
// 设置窗口 - 只读历史
```

## 15. CSP 与 Origin 限制

### 15.1 CSP

```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; img-src 'self' data: https://*; style-src 'self' 'unsafe-inline'"
    }
  }
}
```

**严格 CSP**:开发时不方便(`'unsafe-eval'` Vite HMR 需要),生产时收紧。

### 15.2 Origin

```json
{
  "app": {
    "security": {
      "freezePrototype": true,
      "pattern": { "use": "isolation" }   // 启用 isolation pattern
    }
  }
}
```

isolation pattern 让前端无法直接访问危险 API,只能通过 invoke → Rust command。

---

# Part 5 — 窗口 / 托盘 / 菜单

## 16. 窗口创建与控制

### 16.1 静态窗口(配置式)

`tauri.conf.json`:
```jsonc
"windows": [
  { "label": "main", "title": "主窗口", "width": 1200, "height": 800 }
]
```

### 16.2 动态窗口(Rust 创建)

```rust
use tauri::{WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
pub fn open_settings(app: tauri::AppHandle) -> AppResult<()> {
    let win = WebviewWindowBuilder::new(
        &app,
        "settings",
        WebviewUrl::App("settings.html".into()),
    )
    .title("设置")
    .inner_size(600.0, 400.0)
    .resizable(false)
    .build()
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}
```

### 16.3 窗口操作

```rust
let win = app.get_webview_window("main").unwrap();

win.set_title("新标题")?;
win.set_size(tauri::LogicalSize::new(800.0, 600.0))?;
win.set_position(tauri::LogicalPosition::new(100, 100))?;
win.show()?;
win.hide()?;
win.close()?;
win.set_focus()?;
win.minimize()?;
win.maximize()?;
win.unmaximize()?;
win.set_fullscreen(true)?;
```

## 17. 系统托盘

```rust
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    w.show().ok();
                    w.set_focus().ok();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    w.show().ok();
                    w.set_focus().ok();
                }
            }
        })
        .build(app)?;
    Ok(())
}
```

## 18. 菜单栏

```rust
use tauri::menu::{Menu, MenuBuilder, Submenu, PredefinedMenuItem};

fn setup_menu(app: &tauri::App) -> tauri::Result<()> {
    let file_menu = Submenu::with_id_and_items(
        app, "file", "文件", true,
        &[&PredefinedMenuItem::quit(app, None)?],
    )?;
    let edit_menu = Submenu::with_id_and_items(
        app, "edit", "编辑", true,
        &[&PredefinedMenuItem::copy(app, None)?, &PredefinedMenuItem::paste(app, None)?],
    )?;
    let menu = Menu::with_items(app, &[&file_menu, &edit_menu])?;
    app.set_menu(menu)?;
    app.on_menu_event(|app, event| {
        // event.id.as_ref() 处理点击
    });
    Ok(())
}
```

---

# Part 6 — 插件生态

## 19. 官方插件一览

| 插件 | 用途 | Cargo 依赖 | npm 依赖 |
|---|---|---|---|
| fs | 文件 I/O | `tauri-plugin-fs` | `@tauri-apps/plugin-fs` |
| dialog | 打开/保存/消息框 | `tauri-plugin-dialog` | `@tauri-apps/plugin-dialog` |
| shell | 调外部命令 | `tauri-plugin-shell` | `@tauri-apps/plugin-shell` |
| http | HTTP 客户端 | `tauri-plugin-http` | `@tauri-apps/plugin-http` |
| notification | 系统通知 | `tauri-plugin-notification` | `@tauri-apps/plugin-notification` |
| store | KV 存储 | `tauri-plugin-store` | `@tauri-apps/plugin-store` |
| updater | 自动更新 | `tauri-plugin-updater` | `@tauri-apps/plugin-updater` |
| process | 进程信息 | `tauri-plugin-process` | `@tauri-apps/plugin-process` |
| window-state | 记住窗口位置 | `tauri-plugin-window-state` | - |
| deep-link | 自定义 URL Scheme | `tauri-plugin-deep-link` | - |
| single-instance | 单实例运行 | `tauri-plugin-single-instance` | - |
| autostart | 开机启动 | `tauri-plugin-autostart` | - |

### 19.1 安装插件(以 fs 为例)

```toml
# Cargo.toml
[dependencies]
tauri-plugin-fs = "2"
```

```rust
// src-tauri/src/lib.rs
.plugin(tauri_plugin_fs::init())
```

```jsonc
// src-tauri/capabilities/default.json
{
  "permissions": ["fs:default"]
}
```

```ts
// 前端
import { readTextFile } from '@tauri-apps/plugin-fs';
const content = await readTextFile('/path/to/file.txt');
```

### 19.2 插件配置

有些插件有自定义配置:
```rust
.plugin(tauri_plugin_fs::Builder::default()
    .build())
```

## 20. 自定义插件

适合提取**可复用的 command + 权限 + 配置**给多个 app。

```rust
// 独立 crate: tauri-plugin-myfeature
use tauri::{plugin::Builder, Runtime, Manager};

pub struct Builder;

impl<R: Runtime> Builder<R> {
    pub fn new() -> Self { Self }
    pub fn build(self) -> tauri::plugin::TauriPlugin<R> {
        tauri::plugin::Builder::new("myfeature")
            .setup(|app, _api| {
                app.manage(MyState::default());
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                commands::my_command,
            ])
            .build()
    }
}

// 自定义权限(在 plugin 内部):
// permissions/default.toml 或 permissions/xxx.toml
```

## 21. 插件的权限与作用域

每个插件有内置权限列表。例如 `tauri-plugin-fs`:
- `fs:default` - 基础读
- `fs:allow-read-text-file` - 读文本
- `fs:allow-write-text-file` - 写文本
- `fs:allow-exists` - 检查存在
- `fs:allow-mkdir` - 建目录
- `fs:allow-remove` - 删除

细粒度 scope:
```json
{
  "permissions": [
    {
      "identifier": "fs:allow-read-text-file",
      "allow": [{ "path": "$APPDATA/**" }]
    }
  ]
}
```

---

# Part 7 — 性能与发布

## 22. Bundle 体积优化

### 22.1 默认体积

Tauri 2 桌面 app 典型大小:

| 平台 | 体积 |
|---|---|
| Windows .msi | 5-8MB |
| macOS .dmg | 4-6MB |
| Linux .deb | 4-7MB |
| Linux .AppImage | 5-8MB |

### 22.2 优化技巧

```toml
# Cargo.toml
[profile.release]
strip = true              # 移除符号表
lto = true                # 链接时优化
codegen-units = 1         # 单 codegen unit,更激进优化
opt-level = "s"           # size 优化
panic = "abort"           # 移除 unwinding 代码
```

```rust
// 避免引入重型依赖
hex = "0.4"               # ✅ 轻量
sha2 = "0.10"             # ✅
// ❌ 避免 crypto crate 装整个 OpenSSL
```

```jsonc
// tauri.conf.json
{
  "bundle": {
    "targets": ["app", "msi"],   // 只构建必要 target
    "resources": []                // 不打包多余资源
  }
}
```

### 22.3 减小前端 bundle

- Vite tree-shaking 默认开启
- 按需引入 Element Plus(我们用了 unplugin-vue-components)
- 字体用 subset
- 图片用 WebP

## 23. 启动速度优化

| 优化 | 影响 |
|---|---|
| 减少 Rust crate 数量 | 链接时间 -20-50% |
| `lto = true` | 链接时间,但启动稍快 |
| 延迟加载非关键 module | 启动 -100-300ms |
| 减少 setup() 同步操作 | 第一帧 -50-200ms |

```rust
.setup(|app| {
    // ❌ 在 setup 同步做大量工作 → 阻塞窗口显示
    let _ = heavy_init();

    // ✅ 后台异步初始化
    let app2 = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        heavy_init_async().await;
        app2.emit("ready", ()).ok();
    });
    Ok(())
})
```

## 24. 跨平台打包

```bash
# 开发
pnpm tauri:dev

# 构建当前平台
pnpm tauri:build

# 指定目标
pnpm tauri:build -- --target x86_64-pc-windows-msvc
pnpm tauri:build -- --target aarch64-apple-darwin
pnpm tauri:build -- --target x86_64-unknown-linux-gnu
```

产物:
- Windows: `src-tauri/target/release/bundle/{msi,nsis}/`
- macOS: `src-tauri/target/release/bundle/{dmg,macos}/`
- Linux: `src-tauri/target/release/bundle/{deb,appimage,rpm}/`

### 24.1 平台特定

**Windows**:需要 MSVC build tools + WebView2 Runtime
**macOS**:需要 Xcode CLT,签名需要开发者证书
**Linux**:需要 webkit2gtk-4.1 + libappindicator 等

## 25. 自动更新

集成 `tauri-plugin-updater`:

```toml
[dependencies]
tauri-plugin-updater = "2"
```

```rust
.plugin(tauri_plugin_updater::Builder::new().build())
```

```jsonc
// tauri.conf.json
{
  "plugins": {
    "updater": {
      "endpoints": ["https://github.com/.../releases/latest/download/{{target}}/{{arch}}/{{app_name}}_{{version}}"],
      "pubkey": "..."   // 生成密钥对
    }
  }
}
```

```ts
import { check } from '@tauri-apps/plugin-updater';
const update = await check();
if (update?.available) {
  await update.downloadAndInstall();
}
```

发布时:
1. 在 GitHub Releases 创建新 release
2. 上传对应平台的安装包(.msi / .dmg / .deb)
3. 用户启动时自动检测更新

---

# Part 8 — 调试与排错

## 26. DevTools 与日志

### 26.1 打开 DevTools

dev 模式默认 F12 或右键 → 检查元素。

生产模式开启:
```jsonc
// tauri.conf.json
{
  "app": { "windows": [{ "label": "main", "devtools": true }] }
}
```

### 26.2 Rust 日志

我们已用 `log` crate + `env_logger`,详见 `LOGGING.md`。

```bash
# 不同级别
RUST_LOG=debug pnpm tauri:dev
RUST_LOG=mmcode_toolbox_lib=trace pnpm tauri:dev
```

### 26.3 前端日志

`src/api/index.ts` 已封装 `invokeCmd` 自动打日志。DevTools Console 看输出。

### 26.4 跨语言跟踪

在 Rust command 里打 `log::debug!("cmd={} input={:?}", cmd, input)`,前端 invoke 之前打 `console.debug`。通过时间戳对齐日志。

## 27. 常见错误排查

### Q: invoke 报错 "command not found"
A: 检查 `lib.rs` 的 `generate_handler!` 数组里有没有注册。

### Q: invoke 报错 "permission denied"
A: 检查 `capabilities/default.json` 里有没有相应权限。

### Q: 窗口空白
A: 打开 DevTools → Console 看错误。常见原因:`csp` 过严 / `frontendDist` 路径错。

### Q: `pnpm tauri:dev` 报 "connection refused"
A: Vite 没起来。检查 `pnpm dev` 是否单独跑通。

### Q: 构建报 "icon not found"
A: 检查 `tauri.conf.json` 的 `bundle.icon` 路径,文件必须真实存在。

### Q: 启动后立刻闪退
A: 看 stderr 终端输出。常见:`panic!` 在 setup hook 里没被捕获。

### Q: Windows 上 `WebView2` 缺失
A: Win10 1803+ 自带。Win7 / 更老的 Win10 要手动装 WebView2 Runtime。

### Q: macOS 上 "应用已损坏"
A: 没签名。开发时右键打开,生产需要 Developer ID 签名。

### Q: Linux 上 .AppImage 跑不起来
A: 需要 fuse + libfuse2。Ubuntu 22.04+ 默认 OK。

### Q: 性能差,启动慢
A: 见 §22 / §23 优化章节。

---

# 附录 A — 完整 command 生命周期

```rust
// 用户在 UI 点击「格式化」按钮
// ↓
// 前端 invoke('json_format', { req: { input, indent, sort_keys } })
// ↓ 序列化为 JSON
// ↓ 跨 IPC 边界(系统消息)
// ↓
// Tauri 路由器匹配到 commands::json_fmt::json_format
// ↓ 反序列化 args 为 JsonReq
// ↓ 调 json_format(req)
// ↓ execute log_and_run 包装
// ↓ 业务逻辑
// ↓ 返回 AppResult<JsonResp>
// ↓ 序列化为 JSON
// ↓ 跨 IPC 边界
// ↓
// 前端 await 返回值
// ↓ 显示输出 / 保存历史
```

---

# 附录 B — Tauri 2 生态工具

| 工具 | 用途 |
|---|---|
| [tauri-action](https://github.com/tauri-apps/tauri-action) | GitHub Actions 集成 |
| [tauri-typegen](https://github.com/ahkohd/tauri-typegen) | Rust command → TS 类型 |
| [tauri-specta](https://github.com/specta-rs/tauri-specta) | 同上,更激进 |
| [cargo-tauri](https://github.com/tauri-apps/tauri) | CLI |
| [Tauri Inspector](https://tauri.app/) | 调试 DevTools |

---

# 附录 C — 本项目 Tauri 配置文件速查

```jsonc
// src-tauri/tauri.conf.json (本项目 v0.2.0)
{
  "productName": "Mavis Code Toolbox",
  "version": "0.2.0",
  "identifier": "com.minimax.mmcode-toolbox",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "devUrl": "http://127.0.0.1:1420",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [{
      "label": "main",
      "title": "Mavis Code Toolbox",
      "width": 1280, "height": 800,
      "minWidth": 960, "minHeight": 600,
      "resizable": true, "center": true,
      "decorations": true, "transparent": false
    }],
    "security": { "csp": null }  // dev 友好,生产前收紧
  },
  "bundle": {
    "active": true, "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.ico"],
    "category": "DeveloperTool"
  }
}
```

`src-tauri/capabilities/default.json`:
```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default", "core:window:default", "core:webview:default",
    "core:event:default", "core:app:default", "core:resources:default",
    "core:menu:default", "core:tray:default",
    "dialog:default", "fs:default"
  ]
}
```

---

# 附录 D — 推荐学习路径

1. **官方教程**:https://tauri.app/start/ (1-2 小时跑通最小 demo)
2. **本项目走读**:按本文 §4.6 + `DEVELOPMENT.md` §5 跑一遍
3. **本指南深入**:本文 Part 2 (IPC) + Part 4 (安全) + Part 7 (性能)
4. **源码级学习**:Tauri 仓库 https://github.com/tauri-apps/tauri(看 `examples/` 目录)
5. **生产实践**:集成 updater / 签名 / 公证 / 自动构建(本文 Part 7)

**Happy hacking!** 任何具体场景想深入,直接问。
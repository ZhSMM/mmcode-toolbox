# Mavis Code Toolbox — 高级开发指南

> 本文专门讲**深度话题**:错误体系、复杂响应、异步、事件、状态、性能。配套 `DEVELOPMENT.md` 使用——前者讲「怎么新增工具」,本文讲「如何把工具做得更好」。

---

## 目录

- [Part 1 — 自定义错误](#part-1--自定义错误)
- [Part 2 — 自定义响应](#part-2--自定义响应)
- [Part 3 — 异步特性](#part-3--异步特性)
- [Part 4 — 事件与流](#part-4--事件与流)
- [Part 5 — 状态管理](#part-5--状态管理)
- [Part 6 — 性能与最佳实践](#part-6--性能与最佳实践)
- [Part 7 — 测试进阶](#part-7--测试进阶)
- [Part 8 — 实战案例](#part-8--实战案例)

---

# Part 1 — 自定义错误

## 1.1 现有 `AppError` 体系

`src-tauri/src/error.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Db(String),
    #[error("IO 错误: {0}")]
    Io(String),
    #[error("输入非法: {0}")]
    Invalid(String),
    #[error("内部错误: {0}")]
    Internal(String),
}
```

**核心设计**:
- `thiserror` 自动派生 `Display + Error`
- 手写 `Serialize`:序列化为 `{ kind: "db", message: "数据库错误: ..." }`
- 前端 `ApiError` 用 `kind` 字段做国际化路由

**优点**:
- 错误分类固定(`kind` 是 enum),前端可按类型决定行为(输入校验→不重试;内部错误→日志上报)
- 字符串内容随语言变化(但目前是中文,见 §1.6 国际化)

## 1.2 添加新的错误种类

假设要加一个「鉴权失败」:

```rust
// src-tauri/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Db(String),

    #[error("IO 错误: {0}")]
    Io(String),

    #[error("输入非法: {0}")]
    Invalid(String),

    #[error("内部错误: {0}")]
    Internal(String),

    // ===== 新增 =====
    #[error("未授权: {0}")]
    Unauthorized(String),

    #[error("调用次数超限,{} 秒后重试", .0)]
    RateLimited(u64),

    #[error("上游服务异常: {0}")]
    Upstream(String),
}

impl AppError {
    pub fn kind(&self) -> &'static str {
        match self {
            AppError::Db(_) => "db",
            AppError::Io(_) => "io",
            AppError::Invalid(_) => "invalid",
            AppError::Internal(_) => "internal",
            AppError::Unauthorized(_) => "unauthorized",
            AppError::RateLimited(_) => "rate_limited",
            AppError::Upstream(_) => "upstream",
        }
    }
}
```

前端 `api/index.ts` 的 `ApiError` 同步扩展:

```ts
export class ApiError extends Error {
  kind: string;
  retryAfter?: number;   // 配合 RateLimited

  constructor(kind: string, message: string, retryAfter?: number) {
    super(message);
    this.kind = kind;
    this.retryAfter = retryAfter;
  }
}

export async function invokeCmd<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e: any) {
    const kind = e?.kind ?? 'unknown';
    const message = e?.message ?? String(e);
    const retryAfter = e?.retryAfter;
    const err = new ApiError(kind, message, retryAfter);
    if (kind === 'rate_limited') {
      err.retryAfter = retryAfter;
    }
    throw err;
  }
}
```

## 1.3 结构化错误(而不是字符串)

把"输入非法"细化成字段:

```rust
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum AppError {
    #[error("validation failed")]
    Invalid(ValidationError),

    #[error("db error: {0}")]
    Db(String),

    // ...
}

#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub field: String,           // "email"
    pub rule: String,           // "format" / "required" / "length"
    pub message: String,        // 详细说明
    pub expected: Option<String>,
    pub actual: Option<String>,
}

// 使用:
return Err(AppError::Invalid(ValidationError {
    field: "email".into(),
    rule: "format".into(),
    message: "邮箱格式不正确".into(),
    expected: Some("*@*.*".into()),
    actual: Some(req.email.clone()),
}));
```

前端拿到:
```json
{
  "kind": "invalid",
  "data": { "field": "email", "rule": "format", "message": "邮箱格式不正确", "expected": "*@*.*", "actual": "abc" }
}
```

可以精确高亮表单的某个字段。

## 1.4 错误链与上下文

使用 `anyhow` 的 `with_context`(在 main.rs 中):

```rust
use anyhow::{Context, Result};

// 仅在 main.rs / 启动期用 anyhow 链式,Rust command 内仍返回 AppError
fn setup_db() -> Result<()> {
    let conn = Connection::open("app.db")
        .context("打开 SQLite 失败")?;
    migrate::run(&conn)
        .context("执行迁移失败")?;
    Ok(())
}
```

## 1.5 错误遥测:收集 + 上报

```rust
// src-tauri/src/error.rs
#[derive(Debug, Serialize)]
pub struct ErrorReport {
    pub kind: String,
    pub message: String,
    pub tool_id: Option<String>,
    pub stack: Option<String>,
    pub occurred_at: i64,
}

#[tauri::command]
pub fn report_error(state: tauri::State<'_, AppState>, report: ErrorReport) -> AppResult<()> {
    log::error!("[error] {:?}", report);
    state.with_db(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS error_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kind TEXT, message TEXT, tool_id TEXT,
                stack TEXT, occurred_at INTEGER
            )",
            [],
        )?;
        conn.execute(
            "INSERT INTO error_log (kind, message, tool_id, stack, occurred_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![report.kind, report.message, report.tool_id, report.stack, report.occurred_at],
        )?;
        Ok(())
    })
}
```

Rust前端全局 error 钩子:

```ts
// main.ts
import { invoke } from '@tauri-apps/api/core';
import { getCurrentToolId } from '@/stores/app';

window.addEventListener('error', async (e) => {
  try {
    await invoke('report_error', {
      report: {
        kind: 'js_error',
        message: String(e.message),
        tool_id: getCurrentToolId(),
        stack: e.error?.stack,
        occurred_at: Date.now(),
      },
    });
  } catch {}
});
```

## 1.6 错误国际化(i18n)

把中文 message 抽到常量:

```rust
// src-tauri/src/i18n.rs
pub fn t(key: &str, locale: &str) -> String {
    match (key, locale) {
        ("invalid.empty", "zh-CN") => "输入为空".into(),
        ("invalid.empty", "en")      => "Input is empty".into(),
        // ...
        _ => key.into(),
    }
}

// command 用法:
return Err(AppError::Invalid(t("invalid.empty", &locale)));
```

或者**让前端负责翻译**(`kind` + 原始数据,前端 i18n):
- `kind = "invalid.empty"` → 前端查表显示
- 这种方式更灵活,后端只输出代码,前端做 UI

---

# Part 2 — 自定义响应

## 2.1 简单 vs 富响应

简单:
```rust
#[derive(Serialize)]
pub struct SumResp { pub result: i64 }
```

富响应(支持前端渐进增强):
```rust
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ToolResp {
    #[serde(rename = "ok")]
    Ok { output: String, duration_ms: u128, bytes: usize },

    #[serde(rename = "err")]
    Err { error: String, hint: Option<String> },
}
```

## 2.2 常用字段模式

### 可选字段
```rust
#[derive(Serialize)]
pub struct StatResp {
    pub mean: f64,
    pub median: Option<f64>,        // 数据太少时无
    pub std_dev: Option<f64>,
}
```

### 时间戳与时长
```rust
pub struct Resp {
    pub created_at: i64,           // 毫秒,前端 new Date(created_at)
    pub duration_ms: u128,         // 性能数据
    pub server_time: String,       // ISO8601 字符串(可直接显示)
}
```

### 列表 + 分页
```rust
#[derive(Deserialize)]
pub struct PageReq { pub page: usize, pub page_size: usize }

#[derive(Serialize)]
pub struct PageResp<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub has_more: bool,
}
```

### 字节流(图片 / 文件内容)

**小**:用 base64 字符串:
```rust
let bytes = std::fs::read(path)?;
Ok(ImageResp { data_b64: base64::Engine::encode(&STANDARD, &bytes), mime: "image/png".into() })
```

前端:
```ts
imgEl.src = `data:${resp.mime};base64,${resp.data_b64}`;
```

**大**:用 Tauri 的二进制流(见 §4.3)。

### Map 与枚举

```rust
#[derive(Serialize)]
pub struct CountersResp {
    pub per_tool: BTreeMap<String, usize>,    // tool_id -> count
    pub level: Severity,                       // enum 自动序列化
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity { Info, Warn, Error }
```

## 2.4 序列化属性大全

```rust
#[derive(Serialize)]
pub struct Demo {
    // 字段重命名
    #[serde(rename = "userId")]
    pub user_id: String,

    // 跳过
    #[serde(skip_serializing)]
    pub password_hash: String,

    // 条件跳过
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    // 默认值(反序列化时)
    #[serde(default)]
    pub count: usize,

    // 展平嵌套
    #[serde(flatten)]
    pub meta: Meta,

    // 自定义序列化
    #[serde(serialize_with = "serialize_duration")]
    pub elapsed: Duration,

    // tag/content 形式(变成 { kind: "X", data: ... })
    #[serde(tag = "kind")]
    pub event: Event,
}
```

## 2.5 大响应:分页 + 流式

对于可能很大的输出(如 100MB 日志),不要一次性返回:

```rust
#[tauri::command]
pub async fn export_large(state: tauri::State<'_, AppState>, app: tauri::AppHandle, req: ExportReq) -> AppResult<i64> {
    // 启动后台任务
    let task_id = chrono::Utc::now().timestamp_millis();
    let app2 = app.clone();
    tokio::spawn(async move {
        let mut conn = state.db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, content FROM logs WHERE ...").unwrap();
        let mut count = 0;
        // 每 1000 行 emit 一次进度
        // ... 用 app.emit("export-progress", ExportProgress { task_id, count }) ...
    });
    Ok(task_id)
}
```

前端订阅进度:
```ts
listen('export-progress', (e) => {
  const { task_id, count } = e.payload;
  // 更新 UI
});
```

详见 Part 4。

---

# Part 3 — 异步特性

## 3.1 sync vs async command

**大多数情况用 `sync`** 就够。Tauri 内部用 tokio 调度,但你的函数是阻塞的也没事——会占住一个 worker 线程。Tauri 默认有多个 worker(每个 CPU 核一个),普通 CPU 操作不会卡住 UI。

```rust
// ✅ 推荐:大多数工具
#[tauri::command]
pub fn json_format(req: JsonReq) -> AppResult<JsonResp> { ... }

// async 用于:真正需要异步的操作
#[tauri::command]
pub async fn fetch_url(url: String) -> AppResult<String> {
    reqwest::get(&url).await?.text().await.map_err(...)
}
```

## 3.2 并发执行:tokio::join!

假设一个工具需要同时调 3 个独立操作:

```rust
#[tauri::command]
pub async fn dashboard(state: tauri::State<'_, AppState>) -> AppResult<DashboardResp> {
    let (history_count, favorite_count, error_count) = tokio::join!(
        count_history(&state),
        count_favorites(&state),
        count_errors(&state),
    );
    Ok(DashboardResp {
        history_count: history_count?,
        favorite_count: favorite_count?,
        error_count: error_count?,
    })
}
```

**注意**:三个并发操作必须能同时访问 db。Mutex 会阻塞,async 友好做法用 `tokio::sync::Mutex`:

```rust
state: tauri::State<'_, AppState>,
// ...
let guard = state.db.lock().await;
```

## 3.3 进度报告:`emit`

```rust
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn long_task(app: AppHandle) -> AppResult<i64> {
    let task_id = uuid::Uuid::new_v4().to_string();
    let app_clone = app.clone();
    let task_id_clone = task_id.clone();

    tokio::spawn(async move {
        for i in 0..100 {
            // 干活 ...
            app_clone.emit("long-task-progress", Progress {
                task_id: task_id_clone.clone(),
                progress: i,
                message: format!("Step {i}/100"),
            }).ok();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        app_clone.emit("long-task-done", task_id_clone).ok();
    });

    Ok(task_id)
}
```

前端订阅:
```ts
import { listen, Event } from '@tauri-apps/api/event';

let unlisten: () => void;
const startTask = async () => {
  const taskId = await invoke<string>('long_task');

  unlisten = await listen<{ task_id: string; progress: number; message: string }>('long-task-progress', (e) => {
    if (e.payload.task_id === taskId) {
      console.log(`${e.payload.progress}% - ${e.payload.message}`);
    }
  });
};
onUnmounted(() => unlisten?.());
```

## 3.4 取消任务

```rust
use tokio::sync::watch;
use std::sync::Arc;

pub struct TaskState {
    pub cancel: watch::Sender<bool>,
}

#[tauri::command]
pub async fn cancellable_task(state: tauri::State<'_, TaskState>) -> AppResult<()> {
    let mut cancel_rx = state.cancel.subscribe();
    tokio::spawn(async move {
        for i in 0..1_000_000 {
            if cancel_rx.changed().await.unwrap_or(false) && *cancel_rx.borrow() {
                break;  // 用户取消
            }
            // 干活 ...
        }
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_task(state: tauri::State<'_, TaskState>) -> AppResult<()> {
    state.cancel.send(true).map_err(|_| AppError::Internal("send failed".into()))?;
    Ok(())
}
```

## 3.5 后台常驻任务

```rust
// main.rs
.setup(|app| {
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        // 每 5 分钟刷新一次缓存
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            if let Err(e) = refresh_cache(&handle).await {
                log::error!("refresh cache failed: {e}");
            }
        }
    });
    Ok(())
})
```

## 3.6 并行 + 超时

```rust
use tokio::time::{timeout, Duration};

#[tauri::command]
pub async fn with_timeout(input: String) -> AppResult<String> {
    match timeout(Duration::from_secs(5), heavy_computation(input)).await {
        Ok(Ok(s)) => Ok(s),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(AppError::Internal("超时(>5s)".into())),
    }
}
```

---

# Part 4 — 事件与流

## 4.1 基础 emit / listen

```rust
use tauri::{AppHandle, Emitter, Manager};

#[tauri::command]
pub fn do_something(app: AppHandle) -> AppResult<()> {
    app.emit("data-changed", Payload { id: 1 })?;
    Ok(())
}
```

前端:
```ts
import { listen } from '@tauri-apps/api/event';
import { emit } from '@tauri-apps/api/event';

const unlisten = await listen<{ id: number }>('data-changed', (e) => {
  console.log('changed:', e.payload);
});
// 之后 unlisten();

// 触发
await emit('custom-event', { foo: 'bar' });
```

## 4.2 目标窗口过滤

```rust
// 只发给特定窗口
use tauri::WebviewWindow;
let win = app.get_webview_window("main").unwrap();
win.emit("to-main", payload)?;

// 全应用广播
app.emit("global", payload)?;

// 后台窗口接收
app.emit_to("background", "job-done", payload)?;
```

## 4.3 大文件流式读写

Tauri 2 用 `tauri-plugin-fs` 在 Rust 端读大文件流:

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};
use tauri::ipc::Channel;

#[tauri::command]
pub async fn stream_file_lines(path: String, on_line: Channel<String>) -> AppResult<u64> {
    let file = File::open(&path).map_err(|e| AppError::Io(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut count = 0u64;
    for line in reader.lines() {
        let line = line.map_err(|e| AppError::Io(e.to_string()))?;
        on_line.send(line).map_err(|e| AppError::Internal(format!("send: {e}")))?;
        count += 1;
    }
    Ok(count)
}
```

前端:
```ts
import { Channel } from '@tauri-apps/api/core';

const channel = new Channel<string>();
channel.onmessage = (line) => console.log(line);

const total = await invoke<number>('stream_file_lines', { path: '/tmp/log.txt', onLine: channel });
console.log(`received ${total} lines`);
```

## 4.4 WebSocket-like 长连接

```rust
use tauri::ipc::Channel;

#[tauri::command]
pub async fn subscribe(state: tauri::State<'_, AppState>, events: Channel<MyEvent>) {
    let rx = state.events_tx.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let _ = events.send(event);
        }
    });
}
```

## 4.5 进度 + 取消 + 完成 的三事件协议

```rust
// 协议:每个长任务发 3 类事件
// task-started    { task_id, name }
// task-progress   { task_id, percent, message }
// task-done       { task_id, success, error? }
```

前端订阅模板:
```ts
const subscribeTask = (taskId: string, onProgress: (p: number) => void, onDone: () => void) => {
  const unlisteners: Array<() => void> = [];
  (async () => {
    unlisteners.push(await listen('task-progress', (e) => {
      if (e.payload.task_id === taskId) onProgress(e.payload.percent);
    }));
    unlisteners.push(await listen('task-done', (e) => {
      if (e.payload.task_id === taskId) onDone();
    }));
  })();
  return () => unlisteners.forEach(u => u());
};
```

---

# Part 5 — 状态管理

## 5.1 `State<T>` 注入

```rust
// lib.rs: app.manage(MyState { ... })
struct AppState { pub db: Mutex<Connection> }
.manage(AppState { db: Mutex::new(conn) })

// command: tauri::State<'_, AppState>
#[tauri::command]
pub fn query(state: tauri::State<'_, AppState>) -> AppResult<Vec<Row>> { ... }
```

**State 注意事项**:
- State 类型必须 `Send + Sync + 'static`
- 内部可变用 `Mutex` / `RwLock` / `tokio::sync::Mutex`(async)
- 不要在 State 中存 `RefCell`(单线程)

## 5.2 多状态

```rust
struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub cache: Arc<RwLock<Cache>>,
    pub http_client: reqwest::Client,  // 已经是 Arc 内部
    pub config: Arc<Config>,
}

// 各自注入
#[tauri::command]
pub fn cache_get(state: tauri::State<'_, AppState>) -> ...
```

## 5.3 前端 Pinia store 模式

每个 store 一个文件,export 一个 setup function:

```ts
// stores/settings.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<'system' | 'light' | 'dark'>('system');
  const language = ref<'zh-CN' | 'en'>('zh-CN');
  const autoUpdate = ref(true);

  function setTheme(m: typeof theme.value) {
    theme.value = m;
    localStorage.setItem('theme', m);
  }

  return { theme, language, autoUpdate, setTheme };
});
```

**约定**:
- ✅ 用 setup 风格(组合式)
- ✅ 只暴露 ref / computed / function
- ❌ 不要塞大量数据到 store,组件内 ref 即可
- ❌ 不要让 store 间循环引用

## 5.4 跨窗口状态同步

如果应用有多个窗口,用 localStorage + `storage` 事件同步:

```ts
window.addEventListener('storage', (e) => {
  if (e.key === 'theme') {
    applyTheme(JSON.parse(e.newValue));
  }
});
```

或者用 Tauri 的事件:
```rust
#[tauri::command]
pub fn update_theme_global(app: AppHandle, theme: String) -> AppResult<()> {
    app.emit("theme-changed", &theme)?;
    Ok(())
}
```

---

# Part 6 — 性能与最佳实践

## 6.1 不要在热路径里 lock DB

```rust
// ❌ 多次 lock,阻塞其他 command
let a = state.db.lock().unwrap();
let b = state.db.lock().unwrap();

// ✅ 一次 lock,闭包内完成所有操作
state.with_db(|conn| {
    // ... 多个 SQL
})
```

## 6.2 大数据流式而非一次性

```rust
// ❌ 一次性读 1GB 文本到内存
let big = std::fs::read_to_string("big.txt")?;

// ✅ 流式
let file = File::open(...)?;
let reader = BufReader::new(file);
// 一行一行处理
```

## 6.3 SQLite 优化

```sql
-- 常用查询建索引
CREATE INDEX idx_history_tool_time ON history(tool_id, created_at DESC);

-- 批量插入用事务
BEGIN;
INSERT INTO ... ;
INSERT INTO ... ;
COMMIT;
```

```rust
// 批量
let tx = conn.transaction()?;
for item in items {
    tx.execute(...)?;
}
tx.commit()?;
```

## 6.4 前端性能

- 大列表用 `el-table-v2` 或虚拟滚动
- JSON 大文档折叠后再格式化
- 防抖处理:输入框搜索用 200ms debounce

```ts
import { useDebounceFn } from '@vueuse/core';
// 或手写:
function debounce<T extends (...args: any[]) => void>(fn: T, ms = 200): T {
  let timer: any;
  return ((...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  }) as T;
}
```

## 6.5 避免重复触发

```vue
<!-- ❌ 每次输入都触发 -->
<el-input @input="search" />

<!-- ✅ debounce 后 -->
<el-input @input="debouncedSearch" />
```

---

# Part 7 — 测试进阶

## 7.1 单元测试模式

### Rust 单元测试

```rust
// 在每个 commands 文件底部
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path() {
        assert_eq!(encode("hi").unwrap(), "aGk=");
    }

    #[test]
    #[should_panic(expected = "empty")]
    fn panic_on_empty() {
        // ...
    }

    #[test]
    fn property_based() {
        // 用 proptest 做属性测试
        proptest::proptest!(|(s: String)| {
            let enc = encode(&s).unwrap();
            let dec = decode(&enc).unwrap();
            assert_eq!(s, dec);
        });
    }
}
```

### Mock 数据库

```rust
// tests/db.rs
use rusqlite::Connection;
use crate::db::open_and_migrate;

pub fn test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    // 直接执行迁移
    crate::db::migrate::run(&conn).unwrap();
    conn
}

#[test]
fn test_history_save() {
    let mut conn = test_db();
    save_history(&conn, "tool1", "in", "out", "success", None).unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 1);
}
```

## 7.2 集成测试(command 端到端)

```rust
// tests/integration.rs
use tauri::test::mock_builder;

#[test]
fn json_format_command() {
    let app = mock_builder().build(tauri::generate_context!()).unwrap();
    let result: JsonResp = tauri::test::get_ipc_response(
        &app,
        "json_format",
        JsonReq { input: "{}".into(), indent: 2, sort_keys: false, escape_unicode: false },
    );
    assert_eq!(result.output, "{}");
}
```

(需要 `tauri = { features = ["test"] }`)

## 7.3 前端测试(Vitest)

```ts
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  plugins: [vue()],
  test: { environment: 'jsdom' },
});
```

```ts
// src/utils/format.test.ts
import { describe, expect, it } from 'vitest';
import { relativeTime } from './format';

describe('relativeTime', () => {
  it('returns seconds ago', () => {
    const now = Date.now();
    expect(relativeTime(now - 5000)).toBe('5 秒前');
  });
});
```

## 7.4 Mock Tauri API

```ts
// vitest.setup.ts
import { vi } from 'vitest';

(globalThis as any).window = (globalThis as any).window ?? {};
(window as any).__TAURI_INTERNALS__ = {
  invoke: vi.fn(async (cmd, args) => {
    if (cmd === 'list_tools') return [{ tool_id: 'test', name: 'Test' }];
    throw new Error(`unmocked: ${cmd}`);
  }),
};
```

---

# Part 8 — 实战案例

## 8.1 案例 1:文件拖拽支持

让任意工具页支持拖入文件:

```ts
// src/composables/useDroppedFile.ts
import { ref } from 'vue';

export function useDroppedFile(onFile: (text: string, file: File) => void) {
  const dragging = ref(false);

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    dragging.value = false;
    const file = e.dataTransfer?.files?.[0];
    if (!file) return;
    file.text().then((text) => onFile(text, file));
  };

  return {
    dragging,
    onDragOver: (e: DragEvent) => { e.preventDefault(); dragging.value = true; },
    onDragLeave: () => { dragging.value = false; },
    onDrop: handleDrop,
  };
}
```

```vue
<template>
  <div :class="{ 'is-dragging': dragging }"
       @dragover="onDragOver" @dragleave="onDragLeave" @drop="onDrop">
    <!-- 工具 UI -->
  </div>
</template>
```

## 8.2 案例 2:大 JSON 文件分析

```rust
// src-tauri/src/commands/json_inspector.rs
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize)]
pub struct JsonInspectorReq {
    pub path: String,
    #[serde(default = "default_max")]
    pub max_depth: usize,
    #[serde(default = "default_max")]
    pub max_samples: usize,
}
fn default_max() -> usize { 100 }

#[derive(Serialize)]
pub struct JsonInspectorResp {
    pub valid: bool,
    pub size_bytes: u64,
    pub depth: usize,
    pub node_count: usize,
    pub keys_top: Vec<(String, usize)>,
    pub types: TypeCounts,
}

#[derive(Serialize, Default)]
pub struct TypeCounts {
    pub object: usize, pub array: usize,
    pub string: usize, pub number: usize,
    pub boolean: usize, pub null: usize,
}

#[tauri::command]
pub fn json_inspect(req: JsonInspectorReq) -> AppResult<JsonInspectorResp> {
    let file = File::open(&req.path).map_err(|e| AppError::Io(e.to_string()))?;
    let reader = BufReader::new(file);
    let value: serde_json::Value = serde_json::from_reader(reader)
        .map_err(|e| AppError::Invalid(format!("解析失败: {e}")))?;
    let mut counter = NodeCounter { max_depth: 0, count: 0, types: Default::default(), keys: HashMap::new() };
    walk(&value, 0, req.max_depth, &mut counter, req.max_samples);
    Ok(JsonInspectorResp { valid: true, size_bytes: std::fs::metadata(&req.path).map(|m| m.len()).unwrap_or(0),
        depth: counter.max_depth, node_count: counter.count, keys_top: counter.keys.into_iter().take(20).collect(),
        types: counter.types })
}
```

## 8.3 案例 3:工具组合(管道)

允许把一个工具的输出直接接到另一个:

```rust
#[tauri::command]
pub fn pipeline(steps: Vec<PipelineStep>) -> AppResult<String> {
    let mut current = steps[0].input.clone();
    for step in &steps {
        current = match step.tool.as_str() {
            "base64_encode" => base64::Engine::encode(&base64::engine::general_purpose::STANDARD, current.as_bytes()),
            "hex_encode" => hex::encode(current.as_bytes()),
            "json_format" => serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&current)?)?,
            // ...
            other => return Err(AppError::Invalid(format!("不支持的步骤: {other}"))),
        };
    }
    Ok(current)
}
```

前端可视化拖拽连线(用 vue-flow 库)。

## 8.4 案例 4:收藏快捷键面板

按 `Cmd+K` 弹出收藏面板,数字键 1-9 直达对应工具:

```vue
<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useAppStore } from '@/stores/app';

const router = useRouter();
const app = useAppStore();

const handler = (e: KeyboardEvent) => {
  if (!e.metaKey && !e.ctrlKey) return;
  const favs = [...app.favorites];
  const num = parseInt(e.key);
  if (num >= 1 && num <= favs.length) {
    const tool = app.tools.find((t) => t.tool_id === favs[num - 1]);
    if (tool) router.push(tool.route);
  }
};
onMounted(() => window.addEventListener('keydown', handler));
onUnmounted(() => window.removeEventListener('keydown', handler));
</script>
```

## 8.5 案例 5:自动更新

集成 `tauri-plugin-updater`:

```toml
# Cargo.toml
[dependencies]
tauri-plugin-updater = "2"
```

```rust
// lib.rs
.plugin(tauri_plugin_updater::Builder::new().build())
```

前端:
```ts
import { check } from '@tauri-apps/plugin-updater';

const update = await check();
if (update?.available) {
  await update.downloadAndInstall();
}
```

GitHub Releases 自动匹配 tag。

---

# 附录 A — Cheat Sheet

```rust
// 自定义错误
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum AppError {
    #[error("...")] Invalid(ValidationError),
    #[error("...")] Db(String),
}

// 自定义响应
#[derive(Serialize)]
pub struct Resp {
    pub data: T,
    pub meta: Meta,
}

// 异步 + tokio
#[tauri::command]
pub async fn cmd() -> AppResult<()> { ... }

// 事件
app.emit("event-name", payload)?;
listen<Payload>("event-name", (e) => ...);

// 进度报告
#[tauri::command]
pub async fn long_task(app: AppHandle) -> AppResult<String> {
    let id = uuid::Uuid::new_v4().to_string();
    let app2 = app.clone();
    let id2 = id.clone();
    tokio::spawn(async move {
        for i in 0..100 {
            app2.emit("progress", Progress { id: id2.clone(), percent: i }).ok();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
    Ok(id)
}

// 状态
.manage(AppState { ... })
state: tauri::State<'_, AppState>

// 并发
let (a, b) = tokio::join!(do_a(), do_b());

// 超时
match tokio::time::timeout(Duration::from_secs(5), work()).await { ... }
```

---

# 附录 B — 常见陷阱清单

| 陷阱 | 解决方案 |
|---|---|
| `<script setup>` 里 `export const` | 抽出到独立 `.ts` 文件 |
| `pub use` 重导出后 `generate_handler` 找不到 `__cmd__` | 用完整路径 `commands::xxx::fn` |
| 结构体内部字段 snake_case 不一致 | Rust 与 TS 都用 snake_case,或加 `#[serde(rename_all)]` |
| `Box<dyn Rng>` 编译错 | 用 enum holder |
| `is_documentation` 等 IPv6 方法 unstable | 手动 RFC 检查 |
| `String::as_str()` 临时借用跨 match arm | 用 `let s = local.as_str();` |
| Vue 组件 `import` 放在 `</script>` 之外 | 只能放 `<script setup>` 内 |
| `as_any` 在多线程下 panic | 用 `Send + Sync` 显式约束 |
| SQLite lock 跨 await | 用 `tokio::sync::Mutex` |

---

# 附录 C — 推荐学习路径

1. **基础**(1 天):Tauri 2 官方教程 + Vue 3 + Rust Book 前 10 章
2. **进阶**(3 天):本项目走读 + DEVELOPMENT.md 走查
3. **高级**(1 周):tokio 异步编程 + SQLite 优化 + WebView 调优
4. **架构**(长期):阅读 ant-design / VueUse / Axum 等成熟项目的源码

推荐阅读:
- [Tauri 2 官方文档](https://tauri.app/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Vue 3 文档](https://vuejs.org/)
- [Element Plus 组件](https://element-plus.org/)

---

**Happy hacking!** 如有具体场景想深入(比如「加一个带图表统计的工具」/「接入 WebSocket 接收外部数据」/「写 Tauri 插件」),直接说,我可以再细化。
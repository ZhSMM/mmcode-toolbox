# 日志系统指南

> 本文讲 **Rust 后端 + 前端**两端的日志/调试体系。覆盖:
> - 启动时能看到什么
> - 每次工具调用发生了什么
> - 失败时日志长什么样
> - 如何调整日志级别
> - 常见调试技巧

---

## 1. 总览

| 端 | 机制 | 输出位置 | 调级别 |
|---|---|---|---|
| **Rust** | `log` crate + `env_logger` | stderr(开发终端) | `RUST_LOG=...` |
| **前端** | `console.debug` / `console.error` | 浏览器 devtools | dev:verbose, prod:warning |

---

## 2. 启动日志

启动 `pnpm tauri:dev`,会在你的终端看到:

```
2026-09-18 22:04:51.234  INFO [mmcode::setup] === Mavis Code Toolbox starting ===
2026-09-18 22:04:51.236  INFO [tauri] Tauri 2.0
2026-09-18 22:04:51.237  INFO [mmcode::setup] SQLite 数据目录: C:\Users\...\AppData\Roaming\com.minimax.mmcode-toolbox\app.db
2026-09-18 22:04:51.245  INFO [mmcode::setup] 数据库就绪
```

字段含义:
- **时间戳** 到毫秒
- **级别** 带颜色(绿/黄/红/青)
- **target** 模块路径(`mmcode::cmd` / `mmcode::setup` 等)

---

## 3. 每次工具调用

我们给所有 25 个工具 + 系统命令(`list_tools` / `save_history` 等)统一接入 `log_and_run`,所以**每个操作都有完整生命周期日志**。

### 3.1 成功调用

```
2026-09-18 22:04:55.123  INFO [mmcode::cmd] [json_format] start tool=json-formatter
2026-09-18 22:04:55.126  INFO [mmcode::cmd] [json_format] ok in 3ms tool=json-formatter
```

字段:
- `[json_format]` — Rust 函数名(`#[tauri::command]` 标记的函数)
- `tool=json-formatter` — 工具 ID(对应 `TOOLS` 常量里的 `tool_id`)
- `3ms` — 耗时

### 3.2 失败调用

```
2026-09-18 22:04:55.123  INFO [mmcode::cmd] [base64_encode] start tool=base64
2026-09-18 22:04:55.124 ERROR [mmcode::cmd] [base64_encode] FAILED in 1ms tool=base64 kind=invalid msg="输入为空"
```

字段:
- `FAILED in 1ms` — 失败
- `kind=invalid` — `AppError` 的 kind(invalid / io / db / internal / unauthorized / rate_limited / upstream)
- `msg="..."` — 完整消息

### 3.3 系统级命令

`list_tools` / `list_history` / `save_history` 等没有 tool_id,用 `log_and_run_sys`:

```
2026-09-18 22:04:55.123  INFO [mmcode::cmd] [list_tools] start tool=-
2026-09-18 22:04:55.124  INFO [mmcode::cmd] [list_tools] ok in 2ms tool=-
```

---

## 4. 调整日志级别

默认级别由 `RUST_LOG` 环境变量控制:

```bash
# 默认 (INFO):只看摘要
pnpm tauri:dev

# DEBUG:看每个命令的输入/输出详情(可能很大!)
RUST_LOG=debug pnpm tauri:dev

# 只看项目日志,其他库走 INFO
RUST_LOG=mmcode_toolbox_lib=debug,tauri=info pnpm tauri:dev

# 只看错误 + 警告
RUST_LOG=warn pnpm tauri:dev

# 看某个模块
RUST_LOG=mmcode::cmd=trace pnpm tauri:dev
```

环境变量设置(`Windows PowerShell`):
```powershell
$env:RUST_LOG = "debug"; pnpm tauri:dev
```

---

## 5. 前端日志

`src/api/index.ts` 给每次 invoke 加了详细日志。

### 5.1 成功调用

DevTools Console:

```
[invoke] ▶ json_format {"req":{"input":"...","indent":2,...}}
[invoke] ✓ json_format (3.2ms)
```

### 5.2 失败调用

```
[invoke] ✗ json_format (1.5ms)
  kind:        invalid
  message:     输入为空
  args:        {"req":{"input":"","indent":2,"sort_keys":true,"escape_unicode":false}}
  stack:       Error: ...
    at ...
```

---

## 6. 调试技巧

### 6.1 找某个工具失败的根因

```bash
RUST_LOG=mmcode_toolbox_lib=debug,mmcode::cmd=debug pnpm tauri:dev
```

触发工具 → 在终端能看到完整输入/输出/堆栈。

### 6.2 调试「界面卡死但没日志」

通常是前端无限循环或未捕获 Promise rejection:

1. 打开 DevTools (F12) → Console
2. 看红色错误(API 失败时 console.error 完整堆栈会在这里)
3. 看 Sources 标签页打断点

### 6.3 找性能瓶颈

```bash
RUST_LOG=mmcode::cmd=info pnpm tauri:dev
```

每个调用都打印耗时。慢的工具一目了然。

### 6.4 写入到文件(生产调试)

```bash
RUST_LOG=info pnpm tauri:dev 2>&1 | tee -i app.log
```

---

## 8. 添加新命令时如何接入日志

新写一个 command 时,用 `log_and_run` 包裹业务体:

```rust
use crate::logging::{log_and_run, log_and_run_sys};

#[tauri::command]
pub fn my_tool(req: MyReq) -> AppResult<MyResp> {
    log_and_run("my_tool", "my-tool-id", || {
        // 业务逻辑,返回 AppResult<MyResp>
        Ok(MyResp { .. })
    })
}
```

无 tool_id 的系统命令:

```rust
#[tauri::command]
pub fn my_admin_cmd(req: MyReq) -> AppResult<MyResp> {
    log_and_run_sys("my_admin_cmd", || {
        Ok(MyResp { .. })
    })
}
```

如果你需要更细粒度(比如手动加输入摘要),用 `CmdTrace`:

```rust
use crate::logging::CmdTrace;

#[tauri::command]
pub fn my_tool(req: MyReq) -> AppResult<MyResp> {
    let trace = CmdTrace::new("my_tool", Some("my-tool-id"))
        .with_input_summary(&req.input);   // 自动截断到 200 字符

    let result = expensive_call(&req);
    match &result {
        Ok(_) => trace.ok(),
        Err(e) => trace.err(e),
    }
    result
}
```

---

## 9. 错误分类速查

| kind | 含义 | 何时 |
|---|---|---|
| `db` | SQLite 错误 | SQL 语法错 / 磁盘满 / 锁 |
| `io` | 文件 I/O 错误 | 文件不存在 / 权限 |
| `invalid` | 用户输入校验失败 | 缺字段 / 格式错 / 长度超限 |
| `internal` | 代码 bug | panic 恢复 / 不变量违反 |
| `unauthorized` | 鉴权失败(预留) | 后续扩展 |
| `rate_limited` | 调用次数超限(预留) | 后续扩展 |
| `upstream` | 外部服务错误(预留) | 后续扩展 |

新增 kind:`src-tauri/src/error.rs` 加一个枚举变体 + 在 `kind()` 函数里加字符串。

---

## 10. 持久化错误日志(可选)

如果想把错误也写到数据库(便于排查用户报告),参考 `ADVANCED.md` §1.5。

当前实现:
- **Rust** 写 stderr(开发终端可见,生产可在 `pnpm tauri:build` 时重定向)
- **前端** 只 console.error,不上报(隐私考虑,工具涉及 JWT secret / 密码等敏感数据)

如果你想加 Sentry / PostHog 之类的上报,在 `src/api/index.ts` 的 catch 块加 `Sentry.captureException(err)` 即可。**注意过滤敏感字段**(`secret`、`password` 等)。

---

## 11. 已知小问题

- **Windows 终端不显示颜色**:`\x1b[31m` 等 ANSI 转义只在支持 ANSI 的终端有颜色。Windows PowerShell 启用:在 `$PROFILE` 加 `$PSStyle.OutputRendering = 'Ansi'`。或者用 Windows Terminal。
- **第三方 crate 的日志**:默认级别可能太啰嗦。用 `RUST_LOG=mmcode_toolbox_lib=debug` 只开本项目。

---

## 12. 速查

```bash
# 调整级别
RUST_LOG=debug pnpm tauri:dev          # 全开
RUST_LOG=warn pnpm tauri:dev           # 只看 warn/error
RUST_LOG=mmcode::cmd=trace pnpm tauri:dev  # 仅命令追踪

# 写入文件
RUST_LOG=info pnpm tauri:dev 2>&1 | tee -i app.log

# Windows ANSI 颜色
$PSStyle.OutputRendering = 'Ansi'  # PowerShell 7+
```
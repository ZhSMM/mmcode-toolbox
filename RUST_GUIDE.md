# Rust 入门与项目实战 — 跟着 Mavis Code Toolbox 学 Rust

> 本教程**专为熟悉 JS/TS 但没写过 Rust 的开发者**设计:每章 1 个核心概念 + 多个可运行示例 + 与本项目代码的对应关系。学完能直接读懂 `src-tauri/src/commands/` 下的任何文件。

> 假设你已读过 [`DEVELOPMENT.md`](./DEVELOPMENT.md)(项目结构)与 [`ADVANCED.md`](./ADVANCED.md)(错误/响应/异步体系)。

---

## 目录

- **Part 1 基础**(1-2 天)
  1. Hello, World!
  2. 变量与可变性
  3. 基本类型
  4. 函数与控制流
  5. **所有权**(Rust 的灵魂)
  6. **借用与引用**
  7. **切片**
  8. **生命周期**

- **Part 2 中级**(2-3 天)
  9. 结构体
  10. **枚举与 Option/Result**(Rust 的另一个灵魂)
  11. **模式匹配**
  12. **错误处理**:`Result<T, E>` + `?`
  13. **Trait**:定义共享行为
  14. 泛型
  15. 集合:`Vec` / `HashMap` / `String`
  16. 模块与可见性

- **Part 3 进阶**(3-5 天)
  17. 闭包
  18. 迭代器
  19. **智能指针**:`Box` / `Rc` / `Arc` / `Mutex`
  20. Cargo 与依赖

- **Part 4 异步**(1 周)
  21. **async/await** 基础
  22. **tokio** runtime
  23. **Send + Sync**:线程安全基础
  24. 并发模式:`join!` / `select!` / `spawn`
  25. **Channel**:流式数据
  26. **取消**:长任务优雅退出

- **Part 5 项目实战**(穿插在各章)
  27. Tauri command 模式
  28. AppError 体系
  29. serde 序列化
  30. **rusqlite** 集成
  31. **State** 注入与共享
  32. **emit** 事件与前后端通信
  33. 性能与零拷贝

- **附录**
  - A. 速查卡
  - B. 常见编译错误
  - C. 推荐阅读

---

# Part 1 — 基础

## 1. Hello, World!

```rust
// src/main.rs
fn main() {
    println!("Hello, world!");
}
```

**对比 JS**:不像 JS 那样可以直接在浏览器/Node REPL 跑,Rust 必须先 `cargo run`。但执行速度比 JS 快 10-100 倍。

**对比本项目**:`src-tauri/src/main.rs` 也只有这几行——它把所有工作 delegate 给 `mmcode_toolbox_lib::run()`。

```rust
// src-tauri/src/main.rs
fn main() {
    mmcode_toolbox_lib::run();
}
```

为什么?这样可以**库 + 二进制**双重目标,以后写集成测试或移动端能复用同一份代码。

---

## 2. 变量与可变性

```rust
let x = 5;        // 不可变(immutable)
let mut y = 10;   // 可变(mutable)
y = 20;           // OK
// x = 6;          // ❌ 编译错:Cannot assign twice to immutable variable
```

**JS 思维迁移**:
- JS 的 `let`/`const` 区分「变量绑定可变」vs「值不可变」
- Rust 的 `let`/`mut` 区分「变量绑定可变」vs「绑定不可变」
- 含义不一样!

**为什么默认不可变?** 安全:大多数 bug 都来自「我以为这里不会变,但实际变了」。Rust 强制你声明意图。

**shadowing**:
```rust
let x = 5;
let x = x + 1;     // 重新 shadow,不是赋值
let x = "hello";   // 类型都可以换
```

**本项目示例** `src-tauri/src/commands/csv_viewer.rs`:

```rust
let mut parts: Vec<&str> = items.split(...).map(str::trim).filter(|s| !s.is_empty()).collect();
//          ^^^^^^^^
//          必须 mut,因为后面 .shuffle(&mut rng) 要修改它
```

---

## 3. 基本类型

### 整数

| 长度 | 有符号 | 无符号 |
|---|---|---|
| 8 bit | `i8` | `u8` |
| 16 bit | `i16` | `u16` |
| 32 bit | `i32`(默认) | `u32` |
| 64 bit | `i64` | `u64` |
| arch | `isize` | `usize`(索引/长度) |

```rust
let a: i32 = -42;
let b: u8 = 255;          // 注意:u8 最大 255
let size: usize = 100;    // Vec 长度常用 usize
let hex = 0xff;           // 默认 i32,值 = 255
let bin = 0b1010;
let byte = b'A';          // u8,ASCII 字符
```

### 浮点

```rust
let f: f64 = 3.14;        // 默认 f64(比 f32 精度高)
let precise: f32 = 1.5;   // 用于图形/嵌入式
```

### 布尔 / 字符 / 字符串

```rust
let b: bool = true;
let c: char = '中';        // 4 字节 Unicode
let s1: &str = "hello";    // 不可变字符串切片(借用)
let s2: String = String::from("hello");  // 拥有所有权的字符串
let s3 = "hello".to_string();
```

**关键区分**:
- `&str`:固定长度的字符串切片(借用,UTF-8 字节序列)
- `String`:堆上分配的、可增长、拥有所有权的字符串

```rust
let s: String = String::from("hello");
let slice: &str = &s;        // 借用,无拷贝
let owned: String = s;        // move,s 之后不可用
```

### 数组与元组

```rust
let arr: [i32; 3] = [1, 2, 3];
let tup: (i32, &str, bool) = (42, "answer", true);
let (x, y, z) = tup;          // 解构
```

**本项目示例** `src-tauri/src/commands/uuid.rs`:

```rust
let mut items = Vec::with_capacity(count);
for _ in 0..count {
    let s = match req.version.as_str() {
        "v4" => Uuid::new_v4().to_string(),
        "v7" => Uuid::now_v7().to_string(),
        // ...
    };
    items.push(if req.uppercase { s.to_uppercase() } else { s });
}
```

---

## 4. 函数与控制流

### 函数

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b    // 不带分号 = 返回值
}

fn print_sum(a: i32, b: i32) {
    println!("sum = {}", a + b);  // () 返回 unit
}
```

**对比 TS**:
```ts
function add(a: number, b: number): number { return a + b; }
```

### if 是表达式(有值)

```rust
let x = if condition { 1 } else { 2 };
```

### 循环

```rust
// for 遍历迭代器
for i in 0..5 { println!("{}", i); }      // 0..5 是 0,1,2,3,4
for i in 0..=5 { println!("{}", i); }     // 0..=5 是 0..5
for ch in "hello".chars() { println!("{}", ch); }

// while
while x > 0 { x -= 1; }

// loop(无限循环,带返回值)
let result = loop {
    x += 1;
    if x > 10 { break x * 2; }
};
```

### match(模式匹配)

```rust
match x {
    1 => println!("one"),
    2 | 3 => println!("two or three"),
    4..=10 => println!("4-10"),
    _ => println!("other"),
}
```

`match` 是穷尽的:编译器会强制你处理所有可能。**比 JS 的 switch 严格**。

**本项目示例** `src-tauri/src/commands/cidr.rs`:

```rust
match net {
    IpNetwork::V4(n) => { /* IPv4 处理 */ }
    IpNetwork::V6(n) => { /* IPv6 处理 */ }
}
// ❌ 如果漏写 V6 分支,编译报错
```

---

## 5. **所有权**——Rust 的灵魂

**规则**:
1. 每个值有且只有一个所有者(owner)
2. 所有者离开作用域时,值被 drop
3. 赋值/传参默认 move(转移所有权)

```rust
let s1 = String::from("hello");
let s2 = s1;     // s1 的所有权 move 到 s2
// println!("{}", s1);  // ❌ s1 已无效

let s3 = s2.clone();  // 深拷贝,s2 仍有效
println!("{} {}", s2, s3);  // OK
```

**为什么需要?**:JS/Python 的 GC 是不确定的暂停 + 内存占用高;C/C++ 的 malloc/free 容易漏。Rust 在**编译期**保证内存安全,无 GC 无需手动释放。

**与 JS 关键差异**:
```js
// JS:对象按引用共享
const a = { x: 1 };
const b = a;
b.x = 2;
console.log(a.x);  // 2(共享)

```
```rust
// Rust:赋值是 move,不是引用共享
let a = String::from("x");
let b = a;          // move
// b.push_str("y");
// println!("{}", a);  // ❌ a 已无效
```

**本项目示例** `src-tauri/src/commands/history.rs`:

```rust
fn truncate(s: &Option<String>, max: usize) -> Option<String> {
    s.as_ref().map(|v| {
        if v.len() <= max {
            v.clone()         // 需要 clone,因为要返回 owned
        } else {
            let mut out = v[..max].to_string();  // 切片转 owned
            out.push_str("\n...[已截断]...");
            out
        }
    })
}
```

这里 `v` 是 `&String`(借用),`v[..max]` 是 `&str` 切片,`.to_string()` 申请新堆内存返回 `String`。

---

## 6. **借用与引用**

`&T` 不可变借用(可同时有多个),`&mut T` 可变借用(同一时刻只能有一个)。

```rust
fn len(s: &String) -> usize {  // 借用,不获取所有权
    s.len()
}                            // s 离开作用域,但因为只是借用,原值不动

let s = String::from("hello");
let n = len(&s);
println!("{} has length {}", s, n);  // s 仍可用
```

**借用规则**(编译器强制):
1. 同一时刻,要么一个 `&mut`,要么任意多个 `&`
2. 引用必须总是有效的(不能悬垂)

```rust
let mut s = String::from("hello");
let r1 = &s;       // OK
let r2 = &s;       // OK
// let r3 = &mut s; // ❌ r1, r2 还在用
println!("{} {}", r1, r2);
let r3 = &mut s;   // OK,r1/r2 不再用了
```

**对比 TS/JS**:
```js
// JS 函数可以修改参数
function mutate(obj) { obj.x = 1; }
const a = {};
mutate(a);
console.log(a.x); // 1(被改)

// Rust 函数签名声明是否能改
fn mutate(s: &mut String) { s.push_str("!"); }  // 改
fn read(s: &String) -> usize { s.len() }         // 不改
```

**本项目示例** `src-tauri/src/db/mod.rs`:

```rust
pub fn seed_tools(conn: &Connection) -> AppResult<()> {  // 借用
    let tx = conn.unchecked_transaction()?;             // 借用 conn 拿 tx(可变借用)
    {
        let mut stmt = tx.prepare(...)?;                // stmt 是 tx 的可变借用
        for t in TOOLS.iter() {                          // 不可变借用 TOOLS
            stmt.execute(...)?;                          // stmt 仍持有
        }
    }
    tx.commit()?;
    Ok(())
}
```

---

## 7. 切片

切片是**引用**到集合的一部分:

```rust
let s = String::from("hello world");
let hello: &str = &s[0..5];     // "hello"
let world: &str = &s[6..11];    // "world"

let arr = [1, 2, 3, 4, 5];
let slice: &[i32] = &arr[1..3];  // &[2, 3]
```

**本项目示例** `src-tauri/src/commands/hex.rs`:

```rust
fn parse_hex(s: &str) -> Result<(u8, u8, u8, u8), AppError> {
    let h = s.trim().trim_start_matches('#');  // &str 切片借用
    let bytes = hex::decode(h)?;               // &[u8] -> Vec<u8>(owned)
    // ...
}
```

---

## 8. **生命周期**

**问题**:引用是借用,编译器怎么知道它是否还有效?

```rust
fn longest(x: &str, y: &str) -> &str {  // ❌ 不知道返回谁的引用
    if x.len() > y.len() { x } else { y }
}
```

**解决**:生命周期标注 `'a`(撇号 + 名字)

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
//     ↑                ↑                ↑
//   x, y, 返回值 都用同一个生命周期 'a
//   含义:返回的引用与 x、y 一样长寿
```

**Rust 2021+ 生命周期省略**:多数情况不需要写。函数签名只有 `&str` 一个借用时,编译器自动推断。

**本项目示例** `src-tauri/src/state.rs`:

```rust
pub fn with_db<F, R>(&self, f: F) -> Result<R, crate::error::AppError>
where
    F: FnOnce(&mut Connection) -> Result<R, rusqlite::Error>,
{
    let mut guard = self.db.lock().map_err(...)?;   // guard: MutexGuard
    f(&mut guard).map_err(crate::error::AppError::from)  // 把 rusqlite::Error 转为 AppError
}
```

`&mut guard` 是可变借用,生命周期与 `guard` 一致(局部)。`FnOnce` 闭包接受 `&mut Connection`(生命周期任意)。

---

# Part 2 — 中级

## 9. 结构体

```rust
struct ToolMeta {
    tool_id: String,
    name: String,
    category: String,
    enabled: bool,
}

let t = ToolMeta {
    tool_id: "json-formatter".into(),
    name: "JSON 格式化".into(),
    category: "encode".into(),
    enabled: true,
};

// 字段简写:变量名与字段名同名
fn make(name: String) -> ToolMeta {
    ToolMeta {
        tool_id: name.clone(),
        name,           // 等同 name: name
        category: "encode".into(),
        enabled: true,
    }
}

// 方法
impl ToolMeta {
    fn is_text_tool(&self) -> bool {
        self.category == "text"
    }
}
```

**字段更新语法**(非常 JS-friendly):

```rust
let t2 = ToolMeta { name: "新名字".into(), ..t };
//            ^^^^ 其它字段保持 t 的值(只更新 name)
```

**tuple struct**:

```rust
struct Color(u8, u8, u8);
let red = Color(255, 0, 0);
let Color(r, g, b) = red;   // 解构
```

**本项目示例** `src-tauri/src/commands/tools.rs`:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ToolMeta {
    pub tool_id: String,
    pub name: String,
    pub category: String,
    pub route: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub enabled: bool,
}
```

`#[derive(Debug, Clone, Serialize)]` 自动派生:
- `Debug`:`{:?}` 格式化
- `Clone`:深拷贝
- `Serialize`:JSON 序列化

---

## 10. **枚举与 Option/Result**

### 自定义枚举

```rust
enum Network {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

enum Either<L, R> {
    Left(L),
    Right(R),
}
```

### Option:表示「可能为空」

```rust
enum Option<T> {
    None,        // 没有值
    Some(T),     // 有值
}

let x: Option<i32> = Some(42);
let y: Option<i32> = None;

x.unwrap();       // 42(若无值会 panic)
x.is_some();      // true
x.map(|v| v * 2); // Some(84)
```

**对比 JS**:
```js
const x = 42;
const y = null;
// JS 没有强制空检查,容易出 bug
```

### Result:表示「可能失败」

```rust
enum Result<T, E> {
    Ok(T),       // 成功
    Err(E),      // 失败
}

let r: Result<i32, String> = Ok(42);
let r: Result<i32, String> = Err("oops".into());

r.is_ok();                 // true
r.unwrap();                 // 42
r.unwrap_or(0);             // 42(若是 Err 返回 0)
r.map(|v| v.to_string());   // Ok("42")
```

**本项目示例** `src-tauri/src/error.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Db(String),

    #[error("输入非法: {0}")]
    Invalid(String),
}

pub type AppResult<T> = std::result::Result<T, AppError>;
//  ↑ 别名,统一错误类型,这样所有 command 都能返回 AppResult<T>
```

---

## 11. **模式匹配**

```rust
// match 必须穷尽
match opt {
    Some(x) if x > 0 => println!("positive: {x}"),
    Some(x) => println!("other: {x}"),
    None => println!("none"),
}

// if let 简化单分支
if let Some(name) = tool.icon {
    println!("icon: {name}");
}

// let ... else 提前返回
let x = parse(input) else {
    return Err(AppError::Invalid("parse failed".into()));
};

// while let
while let Some(line) = lines.next() {
    println!("{line}");
}

// match 嵌套
match result {
    Ok(ToolMeta { category, .. }) if category == "text" => /* ... */,
    Ok(_) => /* ... */,
    Err(e) => log::warn!("{e}"),
}
```

**本项目示例** `src-tauri/src/commands/base64.rs`:

```rust
fn pick_engine(url_safe: bool) -> base64::engine::general_purpose::GeneralPurpose {
    if url_safe {
        base64::engine::general_purpose::URL_SAFE_NO_PAD
    } else {
        base64::engine::general_purpose::STANDARD
    }
}
```

`GeneralPurpose` 是 enum,两个变体来自不同常量,但类型相同。

---

## 12. **错误处理**:`Result<T, E>` + `?`

**`?` 操作符**:遇到 `Err` 立即返回。

```rust
fn read_file(path: &str) -> Result<String, std::io::Error> {
    let s = std::fs::read_to_string(path)?;   // ? 自动 return Err(...)
    Ok(s)
}
```

**等效于**:
```rust
fn read_file(path: &str) -> Result<String, std::io::Error> {
    let s = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(e),
    };
    Ok(s)
}
```

**转换 `?`**:用 `From` trait

```rust
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

fn read() -> AppResult<String> {
    let s = std::fs::read_to_string("path")?;   // io::Error → AppError::Io (自动)
    Ok(s)
}
```

**对比 TS**:
```ts
// TS:用 try/catch
function readFile(path: string): string | Error {
    try {
        return fs.readFileSync(path, 'utf-8');
    } catch (e) {
        return e as Error;
    }
}

// Rust:? 让 happy path 一气呵成,只有异常时才跳
```

**本项目示例** `src-tauri/src/commands/history.rs`:

```rust
#[tauri::command]
pub fn save_history(state: tauri::State<'_, AppState>, req: SaveHistoryReq) -> AppResult<i64> {
    if req.tool_id.is_empty() {
        return Err(AppError::Invalid("tool_id 不能为空".into()));
    }
    // ...
    state.with_db(|conn| {
        conn.execute(
            "INSERT INTO history ...",
            rusqlite::params![req.tool_id, input, output, req.status, req.error_msg, now],
        )?;
        Ok(conn.last_insert_rowid())   // 闭包返回 Result
    })
}
```

闭包返回 `Result<i64, rusqlite::Error>`,`with_db` 把它转为 `Result<i64, AppError>`。

---

## 13. **Trait**:定义共享行为

```rust
trait Summary {
    fn summarize(&self) -> String;

    // 默认实现
    fn default_summary(&self) -> String {
        String::from("(no summary)")
    }
}

struct Article { pub title: String, pub content: String }

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{}...", &self.content[..50])
    }
}
```

**Trait bound**(泛型约束):

```rust
fn print_summary<T: Summary>(item: &T) {
    println!("{}", item.summarize());
}

// where 子句更清晰
fn notify<T>(item: &T)
where
    T: Summary + Clone,  // 多个 trait
{
    println!("breaking: {}!", item.summarize());
}
```

**常见 trait**:
- `Debug`:`{:?}` 格式化
- `Clone`:深拷贝
- `Copy`:隐式拷贝(只对简单类型)
- `Default`:默认值
- `From` / `Into`:类型转换
- `Iterator`:迭代
- `Send` / `Sync`:线程安全(异步关键)
- `Display` / `Serialize`:输出/序列化

**本项目示例** `src-tauri/src/state.rs`:

```rust
pub fn with_db<F, R>(&self, f: F) -> Result<R, AppError>
where
    F: FnOnce(&mut Connection) -> Result<R, rusqlite::Error>,
{
    // FnOnce:只能调用一次的闭包
    // &mut Connection:接受可变借用
    // Result<R, rusqlite::Error>:返回 Result
}
```

---

## 14. 泛型

```rust
fn largest<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut max = list[0];
    for &item in list {
        if item > max { max = item; }
    }
    max
}

struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self { Self { x, y } }
}

let p = Point::new(1, 2);          // Point<i32>
let p = Point::new("hi", "there");  // Point<&str>
```

**对比 TS**:
```ts
function largest<T extends Comparable & Copyable>(list: T[]): T { ... }
```

Rust 的泛型是**单态化**:每个具体类型生成一份代码,运行时零开销。

---

## 15. 集合

### Vec<T>

```rust
let mut v: Vec<i32> = Vec::new();
v.push(1);
v.push(2);
v.pop();
v.len();
v.is_empty();
v[0];          // 越界 panic
v.get(0);      // Option<&T>

// vec! 宏
let v = vec![1, 2, 3];

// 遍历
for (i, x) in v.iter().enumerate() {
    println!("v[{i}] = {x}");
}

v.iter().map(|x| x * 2).collect::<Vec<_>>();
v.iter().filter(|x| **x > 0).count();
```

### HashMap<K, V>

```rust
use std::collections::HashMap;
let mut m: HashMap<String, i32> = HashMap::new();
m.insert("a".into(), 1);
m.get("a");              // Option<&i32>
m.contains_key("a");     // true
m.remove("a");           // Option<i32>

for (k, v) in &m {
    println!("{k}: {v}");
}

// entry API(插入或更新)
m.entry("a".into()).or_insert(0);
m.entry("a".into()).and_modify(|v| *v += 1).or_insert(1);
```

### String 常用方法

```rust
let mut s = String::from("hello");
s.push_str(" world");
s.push('!');
s.len();            // 字节数(UTF-8)
s.chars().count();   // 字符数
s.contains("world");
s.replace("hello", "hi");
s.split_whitespace();
s.trim();
s.to_uppercase();
format!("{s}, {}", 42);  // 格式化
```

**本项目示例** `src-tauri/src/commands/string_stats.rs`:

```rust
let mut freq: HashMap<String, usize> = HashMap::new();
for c in s.chars() {
    if c.is_whitespace() {
        // ...
        if in_word && !word_buf.is_empty() {
            *freq.entry(word_buf.clone()).or_insert(0) += 1;
        }
    } else if c.is_ascii_alphabetic() {
        word_buf.push(c);
    }
}
```

---

## 16. 模块与可见性

```rust
// src/lib.rs
mod commands;   // 声明子模块(对应 src/commands.rs 或 src/commands/mod.rs)
mod db;

pub use commands::tools::list_tools;  // 重新导出

// 子模块用 super 访问父模块
// src/commands/mod.rs
use crate::error::AppError;
```

**可见性**:
- 默认私有(只在当前模块可见)
- `pub`:公开
- `pub(crate)`:整个 crate 可见
- `pub(super)`:父模块可见

**本项目结构** `src-tauri/src/`:
```
main.rs            # 二进制入口
lib.rs             # 库入口,声明 mod
├── mod commands;       # 注册所有工具
│   ├── mod tools;
│   ├── mod history;
│   └── ... 每个工具一个文件
├── mod db;
└── mod error / state;
```

---

# Part 3 — 进阶

## 17. 闭包

```rust
let add = |a, b| a + b;
println!("{}", add(2, 3));  // 5

// 捕获环境
let x = 10;
let add_x = |a| a + x;

// 三种 Fn trait:
// Fn     - 只读借用捕获(&T)
// FnMut  - 可变借用(&mut T)
// FnOnce - 消耗所有权(T)
```

**本项目示例** `src-tauri/src/state.rs`:

```rust
pub fn with_db<F, R>(&self, f: F) -> Result<R, AppError>
where
    F: FnOnce(&mut Connection) -> Result<R, rusqlite::Error>,
{
    let mut guard = self.db.lock().map_err(...)?;
    f(&mut guard).map_err(AppError::from)
}
//   ^^^^^^^^^^^^ 传入闭包,接受 &mut Connection,返回 rusqlite::Error
```

**move 闭包**:用 `move` 强制获取所有权(用于跨线程):

```rust
let s = String::from("hello");
let closure = move || println!("{s}");  // s move 进闭包
std::thread::spawn(closure);  // 闭包转移到新线程
```

---

## 18. 迭代器

```rust
let v = vec![1, 2, 3, 4, 5];

// 三种迭代器
v.iter()       // &T
v.iter_mut()   // &mut T
v.into_iter()  // T(消耗)

// 适配器(惰性)
v.iter().map(|x| x * 2);          // 链式
v.iter().filter(|x| **x > 2);
v.iter().take(3);
v.iter().skip(2);
v.iter().enumerate();

// 消费器
v.iter().sum::<::<::i32>();
v.iter().collect::<Vec<_>>();
v.iter().count();
v.iter().any(|x| > 5);
v.iter().all(|x| > 0);
v.iter().fold(0, |acc, x| acc + x);

// 组合
let sum_of_doubled_positives: i32 = v.iter()
    .filter(|x| **x > 0)
    .map(|x| x * 2)
    .sum();
```

**对比 TS**:
```ts
v.filter(x => x > 0).map(x => x * 2).reduce((a, b) => a + b, 0);
```

**本项目示例** `src-tauri/src/commands/string_stats.rs`:

```rust
let mut top: Vec<(String, usize)> = freq.into_iter().collect();
top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));  // 按次数降序,同名按字符序
top.truncate(req.top_n);
```

---

## 19. **智能指针**

| 类型 | 用途 | 是否线程安全 |
|---|---|---|
| `Box<T>` | 堆分配(固定大小) | 否 |
| `Rc<T>` | 引用计数共享所有权(单线程) | 否 |
| `Arc<T>` | 原子引用计数(多线程) | 是 |
| `Mutex<T>` | 互斥锁(可借用内部可变) | 是 |
| `RwLock<T>` | 读写锁 | 是 |
| `RefCell<T>` | 内部可变性(单线程) | 否 |

**`Arc<Mutex<T>>`:共享可变状态的标准模式**

```rust
use std::sync::{Arc, Mutex};

let shared: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));

// 克隆 Arc 跨线程共享
let shared2 = Arc::clone(&shared);
std::thread::spawn(move || {
    let mut v = shared2.lock().unwrap();
    v.push(1);
});
```

**`Box<T>`**:把 T 放到堆上,适合递归类型:

```rust
enum List {
    Cons(i32, Box<List>),
    Nil,
}
```

**本项目示例** `src-tauri/src/state.rs`:

```rust
pub struct AppState {
    pub db: Mutex<Connection>,   // 单线程内,不需要 Arc(Mutex 本身是 Sync)
}
```

Tauri command 在 worker 线程上跑,但 `tauri::State<'_, T>` 通过 `&T` 访问,`T` 本身需要 `Sync`。`Mutex<T>` 是 `Sync`(只要 T: Send)。

**对比 JS**:
```js
// JS 单线程,不需要锁;Rust 多线程,需要 Sync/Send/锁
```

---

## 20. Cargo 与依赖

```toml
# Cargo.toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"     # Rust 版本

[dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
proptest = "1"       # 仅 cargo test 用
```

```bash
cargo new my-app          # 新建项目
cargo build              # 编译
cargo run                # 编译并运行
cargo test               # 跑测试
cargo check              # 快速类型检查(不生成二进制)
cargo clippy             # lint
cargo fmt                # 格式化
cargo update             # 更新 lockfile
cargo doc --open         # 生成并打开文档
```

---

# Part 4 — 异步

## 21. **async/await** 基础

```rust
async fn fetch() -> String {
    // 内部用 .await 等待异步操作
    let resp = reqwest::get("https://api.example.com").await.unwrap();
    resp.text().await.unwrap()
}
```

**关键概念**:
- `async fn` 返回一个实现了 `Future` 的类型
- 调用 `async fn` 不立即执行,需要 `.await` 才执行
- `.await` 挂起当前任务,让出线程给其他任务

**普通函数 vs async**:

```rust
fn sync_fn() -> i32 { 42 }

async fn async_fn() -> i32 { 42 }

// main 函数不能直接 await
fn main() {
    // ❌ async_fn().await 不允许(async 必须在 async 上下文)
}
```

**Tokio runtime**:

```rust
#[tokio::main]
async fn main() {
    let s = fetch().await;
    println!("{}", s);
}
```

`#[tokio::main]` 自动创建 runtime 并运行 `main`。

---

## 22. **tokio** runtime

```rust
use tokio::time::{sleep, Duration};

async fn slow() {
    sleep(Duration::from_secs(1)).await;
}

// spawn 后台任务(类似 setTimeout,但 async)
tokio::spawn(async move {
    slow().await;
    println!("done");
});

// 等所有任务完成
tokio::join!(task1, task2, task3);

// 超时
match tokio::time::timeout(Duration::from_secs(5), slow()).await {
    Ok(_) => println!("done"),
    Err(_) => println!("timeout"),
}
```

**Tauri command 默认有 tokio runtime**——你不需要 `#[tokio::main]`,Tauri 帮你管理。

**本项目示例** `src-tauri/src/commands/timestamp.rs`:

```rust
#[tauri::command]
pub async fn timestamp_to(req: TimestampToReq) -> AppResult<TimestampToResp> {
    // ...
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        // sync 处理
        return Ok(...);
    }
    // ...
}
```

这里虽然是 `async fn`,但函数体是同步的——Tauri 仍能调度。

---

## 23. **Send + Sync**:线程安全基础

- `Send`:可以**转移所有权**到另一个线程
- `Sync`:可以**共享引用**(`&T`)到另一个线程

大多数类型都自动 `Send + Sync`(基本类型、String、Vec<T: Send>)。
不 `Send`: `Rc<T>`, `MutexGuard`(未实现 Send)
不 `Sync`: `RefCell<T>`, `Cell<T>`

**Tauri command 要求**:
- `Send` 是 async command 必需的
- `Sync` 是 `tauri::State<'_, T>` 必需的

**判断类型是否线程安全**:编译器会自动检查,看到 `'static + Send + Sync + Clone` 之类的约束时不要慌。

---

## 24. 并发模式

### `tokio::join!`:并发执行,等所有完成

```rust
let (a, b, c) = tokio::join!(
    fetch_a(),
    fetch_b(),
    fetch_c(),
);
```

### `tokio::select!`:第一个完成就返回

```rust
tokio::select! {
    val = fetch_a() => println!("a done: {val}"),
    val = fetch_b() => println!("b done: {val}"),
    _ = tokio::time::sleep(Duration::from_secs(5)) => println!("timeout"),
}
```

### `tokio::spawn`:后台任务

```rust
let handle = tokio::spawn(async move {
    // 后台运行
    expensive_work().await
});

// 等结果
let result = handle.await.unwrap();
```

**本项目示例** `src-tauri/src/commands/timestamp.rs`:

```rust
// rust 的 sync 处理
#[tauri::command]
pub fn timestamp_now() -> NowResp {
    let now = Utc::now();
    // ...
}
// ↑ 实际上不需要 async
```

---

## 25. **Channel**:流式数据

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel::<String>(100);

// 生产者
tokio::spawn(async move {
    for i in 0..10 {
        tx.send(format!("msg-{i}")).await.unwrap();
    }
});

// 消费者
while let Some(msg) = rx.recv().await {
    println!("received: {msg}");
}
```

**Tauri Channel**(与前端通信):

```rust
use tauri::ipc::Channel;

#[tauri::command]
pub async fn stream_logs(path: String, on_line: Channel<String>) -> AppResult<()> {
    let file = tokio::fs::File::open(path).await?;
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut lines = BufReader::new(file).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        on_line.send(line).map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(())
}
```

前端:
```ts
import { Channel } from '@tauri-apps/api/core';
const channel = new Channel<string>();
channel.onmessage = (line) => appendToUI(line);
await invoke('stream_logs', { path, onLine: channel });
```

---

## 26. **取消**:长任务优雅退出

```rust
use tokio::sync::watch;

let (cancel_tx, cancel_rx) = watch::channel(false);

tokio::spawn(async move {
    for i in 0..1_000_000 {
        // 检查取消
        if *cancel_rx.borrow() { break; }
        // 或用 select! 异步等待
        tokio::select! {
            _ = cancel_rx.changed() => {
                if *cancel_rx.borrow() { break; }
            }
            _ = do_work(i) => {}
        }
    }
});

// 用户点取消
cancel_tx.send(true).unwrap();
```

**前端触发**:
```ts
await invoke('cancel_long_task');
```

---

# Part 5 — 项目实战

## 27. Tauri command 模式

**模板**:

```rust
use tauri::State;
use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct MyReq {
    pub input: String,
    #[serde(default)]
    pub option_a: bool,
}

#[derive(Debug, Serialize)]
pub struct MyResp {
    pub output: String,
}

#[tauri::command]
pub fn my_command(
    state: State<'_, AppState>,    // 可选:需要访问共享状态
    req: MyReq,                    // 单一参数 struct
) -> AppResult<MyResp> {
    if req.input.is_empty() {
        return Err(AppError::Invalid("输入为空".into()));
    }
    // ... 业务逻辑 ...
    Ok(MyResp { output: format!("got: {}", req.input) })
}
```

**规则**:
1. 函数名 = snake_case
2. 参数要么 1 个 struct,要么全是原始类型
3. 返回 `AppResult<T>`
4. 在 `commands/mod.rs` `pub mod`
5. 在 `lib.rs` 的 `generate_handler!` 注册

---

## 28. AppError 体系

详见 `ADVANCED.md` §1。核心:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Db(String),
    #[error("输入非法: {0}")]
    Invalid(String),
    // ...
}

// 手写 Serialize:序列化为 { kind, message }
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        // 输出 { kind: "db", message: "..." }
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

---

## 29. serde 序列化

**基础**:

```rust
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub count: u32,
}
```

**字段重命名**:
```rust
#[derive(Serialize)]
pub struct Resp {
    #[serde(rename = "userId")]
    pub user_id: u64,
}
```

**可选字段**:
```rust
#[derive(Serialize)]
pub struct Resp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<String>,  // None 时不输出字段
}
```

**默认值**(反序列化时):
```rust
#[derive(Deserialize)]
pub struct Req {
    #[serde(default)]
    pub limit: u32,           // 默认 0

    #[serde(default = "fn_name")]
    pub mode: String,
}

fn fn_name() -> String { "default".into() }
```

**枚举**:
```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity { Info, Warn, Error }
// 序列化为 "info" / "warn" / "error"
```

**tagged enum**(前端的 `{ kind, data }` 模式):
```rust
#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum Event {
    ToolRan { tool_id: String, duration_ms: u128 },
    ErrorOccurred(AppError),
}
```

**本项目示例** `src-tauri/src/commands/jwt.rs`:

```rust
#[derive(Serialize)]
pub struct JwtDecodeResp {
    pub header: JwtPart,
    pub payload: JwtPart,
    pub signature_hex: String,
    pub verified: Option<bool>,  // None 表示用户没启用 verify
    pub error: Option<String>,
}
```

---

## 30. **rusqlite** 集成

**打开**:
```rust
let conn = Connection::open("path.db")?;
conn.pragma_update(None, "journal_mode", "WAL")?;
```

**迁移**(本项目用 SQL 文件 + `include_str!`):

```rust
const MIGRATIONS: &[(&str, &str)] = &[
    ("001_init", include_str!("../../migrations/001_init.sql")),
];

for (name, sql) in MIGRATIONS {
    conn.execute_batch(sql)?;
}
```

**Prepared Statement**:
```rust
let mut stmt = conn.prepare("INSERT INTO tools (tool_id, name) VALUES (?1, ?2)")?;
stmt.execute(params!["json-formatter", "JSON 格式化"])?;
```

**Query**:
```rust
let mut stmt = conn.prepare("SELECT id, name FROM tools WHERE category = ?1")?;
let rows = stmt.query_map(["encode"], |row| {
    Ok(ToolMeta {
        tool_id: row.get(0)?,
        name: row.get(1)?,
        // ...
    })
})?;
let tools: Vec<ToolMeta> = rows.collect::<Result<_, _>>()?;
```

**事务**:
```rust
let tx = conn.unchecked_transaction()?;
tx.execute(...)?;
tx.execute(...)?;
tx.commit()?;
```

**本项目示例** `src-tauri/src/db/mod.rs`:

```rust
pub fn seed_tools(conn: &Connection) -> AppResult<()> {
    use crate::commands::tools::{CATEGORIES, TOOLS};
    let tx = conn.unchecked_transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR IGNORE INTO tools (tool_id, name, ...) VALUES (?1, ?2, ...)",
        )?;
        for t in TOOLS.iter() {
            stmt.execute(rusqlite::params![
                t.tool_id,        // &String -> &str 自动 Deref
                t.name,
                t.category,
                t.route,
                t.icon,
                t.sort_order,
                t.enabled as i32,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
```

---

## 31. **State** 注入与共享

**定义**:
```rust
struct AppState {
    pub db: Mutex<Connection>,
}
```

**注入**(setup 时):
```rust
.manage(AppState { db: Mutex::new(conn) })
```

**使用**:
```rust
#[tauri::command]
pub fn save(state: tauri::State<'_, AppState>, req: SaveReq) -> AppResult<i64> {
    let mut conn = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    conn.execute("INSERT ...", ...)?;
    Ok(conn.last_insert_rowid())
}
```

**约束**:
- `AppState: Send + Sync + 'static`
- `Mutex<T>` 中 T 必须 Send
- 不要存 `RefCell`(单线程)

---

## 32. **emit** 事件与前后端通信

**后端 emit**:
```rust
use tauri::{AppHandle, Emitter, Manager};

#[tauri::command]
pub async fn long_task(app: AppHandle) -> AppResult<i64> {
    let id = uuid::Uuid::new_v4().to_string();
    let app2 = app.clone();
    let id2 = id.clone();
    tokio::spawn(async move {
        for i in 0..100 {
            app2.emit("long-task-progress", Progress {
                id: id2.clone(),
                percent: i,
            }).map_err(|e| log::warn!("{e}")).ok();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        app2.emit("long-task-done", id2).ok();
    });
    Ok(id)
}
```

**前端监听**:
```ts
import { listen, UnlistenFn } from '@tauri-apps/api/event';

let unlistens: UnlistenFn[] = [];
(async () => {
    unlistens.push(await listen<{ id: string; percent: number }>('long-task-progress', (e) => {
        console.log(`${e.payload.percent}%`);
    }));
    unlistens.push(await listen<string>('long-task-done', (e) => {
        console.log(`task ${e.payload} done`);
        unlistens.forEach(u => u());   // 清理
    }));
})();
```

---

## 33. 性能与零拷贝

### 优先借用

```rust
// ❌ 不必要的 clone
fn process(s: String) -> String {
    println!("{}", s);   // 只读
    s
}

// ✅ 借用即可
fn process(s: &str) -> String {
    println!("{s}");
    s.to_string()
}
```

### 避免大结构 move

```rust
// ❌ Clone 一个大结构
let result = expensive_clone(&big_struct);

// ✅ 借用 + 返回引用
fn analyze(b: &BigStruct) -> &str {
    &b.summary
}
```

### Cow(克隆或借用)

```rust
use std::borrow::Cow;

fn maybe_modify(s: Cow<'_, str>) -> Cow<'_, str> {
    if s.contains("bad") {
        Cow::Owned(s.replace("bad", "good"))
    } else {
        s   // 不需要 clone
    }
}
```

**本项目示例** `src-tauri/src/commands/hex.rs`:

```rust
fn parse_hex(s: &str) -> Result<(u8, u8, u8, u8), AppError> {
    let h = s.trim().trim_start_matches('#');   // &str slice
    let bytes = hex::decode(h)?;                  // &[u8] 借用 -> Vec<u8>(owned)
    let (r, g, b, a) = match bytes.len() {       // 解构 owned Vec
        3 => (bytes[0] * 17, ...),
        // ...
    };
    Ok((r, g, b, a))
}
```

---

# 附录 A — 速查卡

### 类型
```rust
let i: i32 = 42;
let f: f64 = 3.14;
let b: bool = true;
let c: char = '中';
let s: String = String::from("hi");
let r: &str = "slice";
let v: Vec<i32> = vec![1, 2, 3];
let m: HashMap<String, i32> = HashMap::new();
let o: Option<i32> = Some(42);
let e: Result<i32, &str> = Ok(42);
```

### 控制流
```rust
if cond { /* */ } else { /* */ }
match x {
    1 => /* */,
    2 | 3 => /* */,
    4..=10 => /* */,
    _ => /* */,
}
for i in 0..n { /* */ }
while cond { /* */ }
loop { break; }
```

### 函数
```rust
fn name(a: i32, b: &str) -> Result<T, E> { ... }
async fn name() -> T { ... }
fn with_default(x: i32 = 42) { ... }     // 不允许,默认值用 trait 或重载
```

### 错误
```rust
Result<T, E>
Ok(val) / Err(e)
? 自动传播
.unwrap() / .expect("msg")
.unwrap_or(default)
.map_err(|e| ...)  // 转换错误类型
```

### 闭包
```rust
|x| x + 1
|x: i32| x + 1    // 类型标注
move || ...       // 强制获取所有权
Fn / FnMut / FnOnce
```

### 字符串
```rust
String::from("hi")
"hi".to_string()
format!("x={}", 42)
s.len()         // 字节数
s.chars()       // 字符迭代器
&s[..5]         // 切片
```

### 借用
```rust
&T         // 不可变借用
&mut T     // 可变借用(独占)
```

---

# 附录 B — 常见编译错误

| 错误 | 含义 | 修复 |
|---|---|---|
| `cannot move out of ...` | 借用数据的所有权被转移 | 加 `.clone()` 或重新设计 |
| `borrow of moved value` | 用了一个已 move 的值 | 不要 move,借用即可 |
| `expected ..., found ...` | 类型不匹配 | 显式标注或转换 |
| `cannot find value x` | 名字拼写/作用域错 | 检查 import |
| `cannot borrow as mutable` | 没有可变绑定 | 加 `let mut` |
| `lifetime ... required` | 生命周期不匹配 | 加生命周期标注 |
| `the trait bound ... is not satisfied` | 缺 trait 实现 | 加 `#[derive]` 或 impl |
| `expected struct X, found enum Y` | 类型/枚举不匹配 | match 穷尽分支 |
| `cannot return value referencing ...` | 返回了局部数据 | 调整返回类型(owned) |
| `dataflow ... has lifetime ...` | 借用跨作用域 | 把借用提到外层 |

**调试黄金法则**:
1. 看错误第一行(描述)
2. 看错误位置(`src/xx.rs:42:5`)
3. 看标注(`note:` 行)
4. 看修复建议(`help:` 行)

---

# 附录 C — 推荐阅读

按顺序:

1. **The Rust Book**(官方教程,免费): https://doc.rust-lang.org/book/
   - 前 10 章覆盖本教程 Part 1-2
   - 第 16 章讲智能指针

2. **Rust by Example**(代码示例): https://doc.rust-lang.org/rust-by-example/

3. **Rust Async Book**(异步): https://rust-lang.github.io/async-book/

4. **Tauri 官方教程**: https://tauri.app/start/

5. **Tokio Tutorial**: https://tokio.rs/tokio/tutorial

**推荐 crates** (遇到对应需求时查):
- `serde`:序列化
- `tauri`:桌面应用
- `tokio`:异步运行时
- `rusqlite`:SQLite
- `thiserror` / `anyhow`:错误处理
- `tracing`:结构化日志
- `proptest`:属性测试

---

**学完这些,你应该能**:
- ✅ 读懂本项目 `src-tauri/src/` 任何文件
- ✅ 用 DEVELOPMENT.md §5 的流程新增一个工具
- ✅ 用 ADVANCED.md 的模式做错误/响应/异步
- ✅ 调试常见编译错误
- ✅ 自己写小型 Rust 程序

**下一步实战**:打开 `src-tauri/src/commands/json_fmt.rs`,配合本教程读一遍——这是项目里最简单也最典型的 command。读完再看 `jwt.rs`(稍复杂)。然后尝试自己写一个新工具。
//! 工具元数据常量 + 列表命令。
//!
//! 所有工具都在这里集中注册。前端通过 `list_tools()` / `list_categories()` 拉取菜单。
//! `ToolMeta` / `Category` 字段使用 `String` 是因为 rusqlite 的 `row.get()` 需要 owned
//! 类型(`&str` 不实现 `FromSql`),常量数组在首次加载时克隆字符串。

use once_cell::sync::Lazy;
use serde::Serialize;

use crate::error::AppResult;
use crate::state::AppState;

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

#[derive(Debug, Clone, Serialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub sort: i32,
}

/// 大类(左侧菜单一级)。顺序由 `sort` 决定。
pub static CATEGORIES: Lazy<Vec<Category>> = Lazy::new(|| {
    vec![
        Category { id: "encode".into(),    name: "编解码".into(),  icon: Some("Coin".into()),       sort: 1 },
        Category { id: "text".into(),      name: "文本".into(),    icon: Some("EditPen".into()),    sort: 2 },
        Category { id: "crypto".into(),    name: "加解密".into(),  icon: Some("Lock".into()),       sort: 3 },
        Category { id: "generator".into(), name: "生成器".into(),  icon: Some("MagicStick".into()), sort: 4 },
    ]
});

/// 全部工具。占位工具也保留 enabled,占位由前端 PlaceholderTool 组件渲染。
pub static TOOLS: Lazy<Vec<ToolMeta>> = Lazy::new(|| {
    vec![
        // ===== 编编码 =====
        ToolMeta { tool_id: "json-formatter".into(), name: "JSON 格式化".into(), category: "encode".into(), route: "/tools/json-formatter".into(), icon: Some("Document".into()),    sort_order: 11, enabled: true },
        ToolMeta { tool_id: "base64".into(),         name: "Base64 编解码".into(), category: "encode".into(), route: "/tools/base64".into(),         icon: Some("Connection".into()), sort_order: 12, enabled: true },
        ToolMeta { tool_id: "url-codec".into(),      name: "URL 编解码".into(),   category: "encode".into(), route: "/tools/url-codec".into(),      icon: Some("Link".into()),        sort_order: 13, enabled: true },
        ToolMeta { tool_id: "hex".into(),            name: "Hex 转换".into(),     category: "encode".into(), route: "/tools/hex".into(),            icon: Some("Histogram".into()),   sort_order: 14, enabled: true },

        // ===== 文本 =====
        ToolMeta { tool_id: "regex-tester".into(),   name: "正则测试".into(),     category: "text".into(),   route: "/tools/regex-tester".into(),   icon: Some("Search".into()),       sort_order: 21, enabled: true },
        ToolMeta { tool_id: "diff".into(),           name: "Diff 对比".into(),    category: "text".into(),   route: "/tools/diff".into(),           icon: Some("Files".into()),        sort_order: 22, enabled: true },
        ToolMeta { tool_id: "string-stats".into(),   name: "字符串统计".into(),   category: "text".into(),   route: "/tools/string-stats".into(),   icon: Some("DataAnalysis".into()), sort_order: 23, enabled: true },

        // ===== 加解密 =====
        ToolMeta { tool_id: "md5".into(),            name: "MD5".into(),          category: "crypto".into(), route: "/tools/md5".into(),            icon: Some("Key".into()),          sort_order: 31, enabled: true },
        ToolMeta { tool_id: "sha".into(),            name: "SHA-1/256".into(),    category: "crypto".into(), route: "/tools/sha".into(),            icon: Some("Lock".into()),         sort_order: 32, enabled: true },
        ToolMeta { tool_id: "hmac".into(),           name: "HMAC".into(),         category: "crypto".into(), route: "/tools/hmac".into(),           icon: Some("Stamp".into()),        sort_order: 33, enabled: true },

        // ===== 生成器 =====
        ToolMeta { tool_id: "uuid".into(),           name: "UUID 生成".into(),    category: "generator".into(), route: "/tools/uuid".into(),         icon: Some("Promotion".into()),    sort_order: 41, enabled: true },
        ToolMeta { tool_id: "password".into(),       name: "随机密码".into(),     category: "generator".into(), route: "/tools/password".into(),     icon: Some("Key".into()),          sort_order: 42, enabled: true },
        ToolMeta { tool_id: "timestamp".into(),      name: "时间戳转换".into(),   category: "generator".into(), route: "/tools/timestamp".into(),    icon: Some("Timer".into()),        sort_order: 43, enabled: true },
        ToolMeta { tool_id: "qrcode".into(),         name: "二维码生成".into(),   category: "generator".into(), route: "/tools/qrcode".into(),       icon: Some("Picture".into()),      sort_order: 44, enabled: true },
    ]
});

#[tauri::command]
pub fn list_categories() -> AppResult<Vec<Category>> {
    let mut out = CATEGORIES.clone();
    out.sort_by_key(|c| c.sort);
    Ok(out)
}

#[tauri::command]
pub fn list_tools(state: tauri::State<'_, AppState>) -> AppResult<Vec<ToolMeta>> {
    // 优先从 DB 读(运营态下可被动态调整),失败则回落到常量。
    let db_result = state.with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT tool_id, name, category, route, icon, sort_order, enabled \
             FROM tools WHERE enabled = 1 ORDER BY category, sort_order",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ToolMeta {
                tool_id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                route: row.get(3)?,
                icon: row.get(4)?,
                sort_order: row.get(5)?,
                enabled: row.get::<_, i32>(6)? != 0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
    });

    match db_result {
        Ok(mut v) => {
            v.sort_by(|a, b| a.category.cmp(&b.category).then(a.sort_order.cmp(&b.sort_order)));
            Ok(v)
        }
        Err(e) => {
            log::warn!("从 DB 读 tools 失败, 回落常量: {e}");
            Ok(TOOLS.clone())
        }
    }
}
#!/usr/bin/env python3
"""
Codemod: 把所有 commands/*.rs 的 #[tauri::command] 函数体用 log_and_run 包起来。

策略:用文本处理,而不是完整 Rust 解析,简单可靠。
每个 command 函数结构:
    #[tauri::command]
    pub fn NAME(req: Req) -> AppResult<Resp> {
        BODY
    }

    #[tauri::command]
    pub async fn NAME(state, req) -> AppResult<Resp> {
        BODY
    }

转换后:
    use crate::log::{log_and_run, log_and_run_sys};

    #[tauri::command]
    pub fn NAME(req: Req) -> AppResult<Resp> {
        log_and_run("NAME", "tool-id", || {
            BODY
        })
    }
"""

import re
import sys
from pathlib import Path

COMMANDS_DIR = Path(__file__).parent.parent / "src-tauri" / "src" / "commands"

# 每个文件的 tool_id 与函数名映射
TOOL_ID_MAP = {
    "tools.rs": "system",          # 跳过(里面是宏 + 元数据)
    "mod.rs": None,                # 跳过
    "json_fmt.rs": "json_fmt",
    "base64.rs": "base64",
    "url_codec.rs": "url-codec",
    "hex.rs": "hex",
    "regex.rs": "regex-tester",
    "diff.rs": "diff",
    "string_stats.rs": "string-stats",
    "md5.rs": "md5",
    "sha.rs": "sha",
    "hmac.rs": "hmac",
    "uuid.rs": "uuid",
    "password.rs": "password",
    "timestamp.rs": "timestamp",
    "qrcode.rs": "qrcode",
    "aes.rs": "aes",
    "jwt.rs": "jwt",
    "markdown.rs": "markdown",
    "csv_viewer.rs": "csv-viewer",
    "color.rs": "color",
    "cron.rs": "cron",
    "sql_format.rs": "sql-format",
    "base_n.rs": "base-n",
    "cidr.rs": "cidr",
    "random.rs": "random",
    # 系统级(无 tool_id)
    "history.rs": None,            # 系统命令,用 log_and_run_sys
    "favorites.rs": None,
}

# 系统级文件:用 log_and_run_sys
SYS_FILES = {"history.rs", "favorites.rs"}


def find_command_bodies(text: str):
    """返回 [(fn_name, start_idx, body_start, body_end), ...]"""
    # 匹配 #[tauri::command] 后跟 pub fn 或 pub async fn NAME(...)...{BODY}
    # 用正则逐个找

    results = []
    pattern = re.compile(
        r'#\[tauri::command\]\s*\n\s*pub(?:\s+async)?\s+fn\s+(\w+)\s*\(([^)]*)\)\s*->\s*AppResult<([^>]+)>\s*\{',
        re.MULTILINE,
    )
    for m in pattern.finditer(text):
        fn_name = m.group(1)
        args_str = m.group(2)
        ret_type = m.group(3)
        body_start = m.end()  # 在 { 之后
        # 找到匹配的 },需要处理嵌套
        depth = 1
        i = body_start
        while i < len(text) and depth > 0:
            c = text[i]
            if c == '{':
                depth += 1
            elif c == '}':
                depth -= 1
            i += 1
        body_end = i - 1  # 在 } 之前
        results.append((fn_name, args_str, ret_type, m.start(), body_start, body_end))
    return results


def wrap_function(text: str, fn_name: str, args_str: str, ret_type: str,
                 body_start: int, body_end: int, tool_id: str | None) -> tuple[str, bool]:
    """把函数体包到 log_and_run 里。返回 (new_text, changed)。"""
    body = text[body_start:body_end]
    body_stripped = body.strip()

    # 检查是否已经被包装(避免重复)
    if "log_and_run(" in body[:200] or "log_and_run_sys(" in body[:200]:
        return text, False

    if tool_id is None:
        wrapper = "log_and_run_sys"
        first_arg = f'"{fn_name}"'
    else:
        wrapper = "log_and_run"
        first_arg = f'"{fn_name}", "{tool_id}"'

    new_body = f'\n        {wrapper}({first_arg}, || {{\n{body}\n        }})\n    '
    new_text = text[:body_start] + new_body + text[body_end:]
    return new_text, True


def add_use_import(text: str) -> tuple[str, bool]:
    """在 use crate::error::*; 后加 use crate::log::*"""
    if "use crate::log::" in text:
        return text, False
    # 在 use crate::error::{AppError, AppResult}; 后插入
    pat = re.compile(r'use crate::error::\{([^}]+)\};')
    m = pat.search(text)
    if m:
        insert_pos = m.end()
        # 加逗号
        existing = m.group(1)
        if "log_and_run_sys" not in existing and "log_and_run" not in existing:
            new_imports = f'\nuse crate::log::{{log_and_run, log_and_run_sys}};'
            text = text[:insert_pos] + new_imports + text[insert_pos:]
            return text, True
    return text, False


def process_file(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    original = text
    n_changed = 0

    # 1) 加 use import
    text, _ = add_use_import(text)

    # 2) 找出所有 .body 并包装
    bodies = find_command_bodies(text)
    # 从后往前改,避免索引错位
    for fn_name, args_str, ret_type, m_start, body_start, body_end in reversed(bodies):
        tool_id = TOOL_ID_MAP.get(path.name)
        if path.name in SYS_FILES:
            tool_id = None
        text, changed = wrap_function(text, fn_name, args_str, ret_type, body_start, body_end, tool_id)
        if changed:
            n_changed += 1

    if text != original:
        path.write_text(text, encoding="utf-8")
    return n_changed


def main():
    total = 0
    for path in sorted(COMMANDS_DIR.glob("*.rs")):
        n = process_file(path)
        if n > 0:
            print(f"{path.name}: wrapped {n} command(s)")
            total += n
    print(f"Done. Total {total} commands wrapped.")


if __name__ == "__main__":
    main()
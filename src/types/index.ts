// 与 Rust 端对齐的类型定义。
// Rust 端使用 snake_case 序列化，前端通过 invoke<T>() 拿到原始字段。

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
  created_at: number;
}

export interface SaveHistoryReq {
  tool_id: string;
  input?: string | null;
  output?: string | null;
  status?: 'success' | 'error';
  error_msg?: string | null;
}

export interface JsonReq {
  input: string;
  indent?: number;
  sort_keys?: boolean;
  escape_unicode?: boolean;
}

export interface JsonResp {
  output: string;
  duration_ms: number;
}

export interface Base64Req {
  input: string;
  url_safe?: boolean;
}

export interface Base64Resp {
  output: string;
}

// 菜单分组：按 category 分组的工具
export interface ToolGroup {
  category: Category;
  tools: ToolMeta[];
}
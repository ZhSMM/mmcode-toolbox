import { invokeCmd } from './index';
export interface MarkdownReq { input: string; enable_tables?: boolean; enable_footnotes?: boolean; enable_strikethrough?: boolean; enable_tasklists?: boolean }
export interface MarkdownResp { html: string; byte_size: number; line_count: number }
export function markdownRender(req: MarkdownReq) { return invokeCmd<MarkdownResp>('markdown_render', { req }); }
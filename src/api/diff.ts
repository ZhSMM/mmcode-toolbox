import { invokeCmd } from './index';

export interface DiffReq { left: string; right: string; ignore_whitespace?: boolean }
export interface DiffHunk { tag: string; left_line: number | null; right_line: number | null; value: string }
export interface DiffStats { left_lines: number; right_lines: number; added: number; deleted: number; unchanged: number }
export interface DiffResp { hunks: DiffHunk[]; stats: DiffStats }

export function diffText(req: DiffReq) { return invokeCmd<DiffResp>('diff_text', { req }); }
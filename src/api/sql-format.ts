import { invokeCmd } from './index';
export interface SqlFormatReq { input: string; indent?: number; uppercase?: boolean; minify?: boolean }
export interface SqlFormatResp { output: string; line_count: number }
export function sqlFormat(req: SqlFormatReq) { return invokeCmd<SqlFormatResp>('sql_format', { req }); }
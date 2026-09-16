import { invokeCmd } from './index';
export interface CsvParseReq { input: string; delimiter?: string; has_headers?: boolean; row_limit?: number }
export interface ColumnStat { name: string; non_empty: number; empty: number; unique: number; sample_values: string[] }
export interface CsvParseResp {
  headers: string[]; rows: string[][]; total_rows: number;
  column_count: number; delimiter: string; stats: ColumnStat[];
}
export function csvParse(req: CsvParseReq) { return invokeCmd<CsvParseResp>('csv_parse', { req }); }

export interface CsvExtractReq { input: string; delimiter?: string; column_index: number; has_headers: boolean }
export interface CsvExtractResp { column_name: string | null; values: string[] }
export function csvExtractColumn(req: CsvExtractReq) { return invokeCmd<CsvExtractResp>('csv_extract_column', { req }); }
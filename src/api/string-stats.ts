import { invokeCmd } from './index';

export interface StringStatsReq { input: string; top_n?: number }
export interface StringStatsResp {
  chars: number; chars_no_whitespace: number; bytes: number; lines: number;
  words: number; sentences: number; paragraphs: number;
  chinese_chars: number; ascii_chars: number; digits: number; spaces: number;
  top_words: [string, number][];
  byte_distribution: [number, number][];
}

export function stringStats(req: StringStatsReq) { return invokeCmd<StringStatsResp>('string_stats', { req }); }
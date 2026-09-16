import { invokeCmd } from './index';

export interface RegexReq { pattern: string; input: string; flags?: string[] }
export interface RegexMatch { start: number; end: number; text: string; groups: (string | null)[]; named_groups: Record<string, string | null> }
export interface RegexResp { valid: boolean; error: string | null; matches: RegexMatch[] }

export function regexTest(req: RegexReq) { return invokeCmd<RegexResp>('regex_test', { req }); }
export function regexReplace(pattern: string, input: string, replacement: string, flags: string[] = []) {
  return invokeCmd<string>('regex_replace', { pattern, input, replacement, flags });
}
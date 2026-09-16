import { invokeCmd } from './index';

export interface TimestampParseReq { input: string; unit?: 's' | 'ms'; timezone?: 'local' | 'utc' }
export interface TimestampParseResp {
  ms: number; seconds: number;
  iso_utc: string; iso_local: string; rfc2822: string;
  timezone_offset: string; weekday: string;
}
export function timestampParse(req: TimestampParseReq) { return invokeCmd<TimestampParseResp>('timestamp_parse', { req }); }

export interface TimestampToReq { input: string; timezone?: 'local' | 'utc' }
export interface TimestampToResp { ms: number; seconds: number; iso_utc: string }
export function timestampTo(req: TimestampToReq) { return invokeCmd<TimestampToResp>('timestamp_to', { req }); }

export interface NowResp { ms: number; seconds: number; iso_utc: string; iso_local: string }
export function timestampNow() { return invokeCmd<NowResp>('timestamp_now'); }
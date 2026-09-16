import { invokeCmd } from './index';

export interface UrlCodecReq { input: string; mode: 'query' | 'component' }
export interface UrlCodecResp { output: string }

export function urlEncode(req: UrlCodecReq) { return invokeCmd<UrlCodecResp>('url_encode', { req }); }
export function urlDecode(req: UrlCodecReq) { return invokeCmd<UrlCodecResp>('url_decode', { req }); }
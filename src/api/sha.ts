import { invokeCmd } from './index';
export interface ShaReq { input: string; algorithm?: 'sha1' | 'sha256' | 'sha512'; uppercase?: boolean }
export interface ShaResp { hex: string; base64: string }
export function shaHash(req: ShaReq) { return invokeCmd<ShaResp>('sha_hash', { req }); }
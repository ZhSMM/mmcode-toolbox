import { invokeCmd } from './index';
export interface HmacReq { input: string; secret: string; algorithm?: 'hmac-sha256' | 'hmac-sha512'; uppercase?: boolean }
export interface HmacResp { hex: string; base64: string }
export function hmacHash(req: HmacReq) { return invokeCmd<HmacResp>('hmac_hash', { req }); }
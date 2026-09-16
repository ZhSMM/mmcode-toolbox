import { invokeCmd } from './index';
export interface HashReq { input: string }
export interface HashResp { hex: string; base64: string; bytes: number[] }
export function md5Hash(req: HashReq) { return invokeCmd<HashResp>('md5_hash', { req }); }
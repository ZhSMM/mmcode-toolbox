import { invokeCmd } from './index';
export interface UuidReq { count?: number; version?: 'v4' | 'v7' | 'nil'; uppercase?: boolean; with_hyphens?: boolean }
export interface UuidResp { items: string[] }
export function uuidGenerate(req: UuidReq) { return invokeCmd<UuidResp>('uuid_generate', { req }); }
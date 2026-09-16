import { invokeCmd } from './index';
export interface QrCodeReq { input: string; min_version?: number; dark?: string; light?: string; quiet_zone?: number }
export interface QrCodeResp { svg: string; modules: number; version: number }
export function qrcodeGenerate(req: QrCodeReq) { return invokeCmd<QrCodeResp>('qrcode_generate', { req }); }
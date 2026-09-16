import { invokeCmd } from './index';
export interface AesReq { input: string; key_hex: string; nonce_hex?: string; aad_hex?: string }
export interface AesResp { output: string }
export function aesEncrypt(req: AesReq) { return invokeCmd<AesResp>('aes_encrypt', { req }); }
export function aesDecrypt(req: AesReq) { return invokeCmd<AesResp>('aes_decrypt', { req }); }
export function aesRandomKey() { return invokeCmd<AesResp>('aes_random_key'); }
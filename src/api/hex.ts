import { invokeCmd } from './index';

export interface HexReq { input: string; mode: 'encode' | 'decode' | 'hexdump'; uppercase?: boolean }
export interface HexResp { output: string }

export function hexEncode(req: HexReq) { return invokeCmd<HexResp>('hex_encode', { req }); }
export function hexDecode(req: HexReq) { return invokeCmd<HexResp>('hex_decode', { req }); }
export function hexDump(req: HexReq) { return invokeCmd<HexResp>('hex_dump', { req }); }
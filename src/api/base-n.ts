import { invokeCmd } from './index';
export interface Base32Req { input: string; variant?: 'base32' | 'base32hex'; no_pad?: boolean }
export interface Base32Resp { encoded: string }
export function base32Encode(req: Base32Req) { return invokeCmd<Base32Resp>('base32_encode', { req }); }
export function base32Decode(req: Base32Req) { return invokeCmd<Base32Resp>('base32_decode', { req }); }

export interface Base58Req { input: string }
export interface Base58Resp { encoded: string }
export function base58Encode(req: Base58Req) { return invokeCmd<Base58Resp>('base58_encode', { req }); }
export function base58Decode(req: Base58Req) { return invokeCmd<Base58Resp>('base58_decode', { req }); }
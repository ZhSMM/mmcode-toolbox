import { invokeCmd } from './index';
export interface JwtDecodeReq { token: string; verify?: boolean; algorithm?: string; secret?: string }
export interface JwtPart { raw: string; decoded: Record<string, unknown> }
export interface JwtDecodeResp {
  header: JwtPart; payload: JwtPart;
  signature_hex: string; signature_bytes: number;
  verified: boolean | null; error: string | null;
}
export function jwtDecode(req: JwtDecodeReq) { return invokeCmd<JwtDecodeResp>('jwt_decode', { req }); }
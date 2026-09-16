import { invokeCmd } from './index';
export interface PasswordReq {
  length?: number; count?: number;
  uppercase?: boolean; lowercase?: boolean; digits?: boolean; symbols?: boolean;
  no_ambiguous?: boolean; must_include_each?: boolean;
}
export interface PasswordResp { passwords: string[]; strength: string; entropy_bits: number }
export function passwordGenerate(req: PasswordReq) { return invokeCmd<PasswordResp>('password_generate', { req }); }
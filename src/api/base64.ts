import { invokeCmd } from './index';
import type { Base64Req, Base64Resp } from '@/types';

export function base64Encode(req: Base64Req): Promise<Base64Resp> {
  return invokeCmd<Base64Resp>('base64_encode', { req });
}

export function base64Decode(req: Base64Req): Promise<Base64Resp> {
  return invokeCmd<Base64Resp>('base64_decode', { req });
}
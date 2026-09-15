import { invokeCmd } from './index';
import type { JsonReq, JsonResp } from '@/types';

export function jsonFormat(req: JsonReq): Promise<JsonResp> {
  return invokeCmd<JsonResp>('json_format', { req });
}

export function jsonMinify(req: JsonReq): Promise<JsonResp> {
  return invokeCmd<JsonResp>('json_minify', { req });
}
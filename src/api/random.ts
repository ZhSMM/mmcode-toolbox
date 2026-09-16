import { invokeCmd } from './index';
export interface RandomReq {
  mode?: 'int' | 'float' | 'pick' | 'dice' | 'bytes' | 'shuffle';
  min?: number; max?: number; count?: number; items?: string; seed?: number;
}
export interface RandomResp { results: string[]; description: string }
export function randomGenerate(req: RandomReq) { return invokeCmd<RandomResp>('random_generate', { req }); }
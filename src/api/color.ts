import { invokeCmd } from './index';
export interface ColorReq { input: string; from?: 'hex' | 'rgb' | 'hsl' }
export interface ColorResp {
  hex: string; hex_long: string;
  rgb: [number, number, number]; rgba: string;
  hsl: [number, number, number]; hsla: string;
  luminance: number; contrast_white: number; contrast_black: number;
}
export function colorConvert(req: ColorReq) { return invokeCmd<ColorResp>('color_convert', { req }); }
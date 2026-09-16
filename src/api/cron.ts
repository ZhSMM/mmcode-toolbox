import { invokeCmd } from './index';
export interface CronReq { expression: string; start_iso?: string; count?: number }
export interface CronResp { valid: boolean; error: string | null; next_runs: string[] }
export function cronNext(req: CronReq) { return invokeCmd<CronResp>('cron_next', { req }); }
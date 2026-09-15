import { invokeCmd } from './index';
import type { HistoryItem, SaveHistoryReq } from '@/types';

export function listHistory(toolId: string, limit = 50): Promise<HistoryItem[]> {
  return invokeCmd<HistoryItem[]>('list_history', { toolId, limit });
}

export function saveHistory(req: SaveHistoryReq): Promise<number> {
  return invokeCmd<number>('save_history', { req });
}

export function deleteHistory(id: number): Promise<void> {
  return invokeCmd<void>('delete_history', { id });
}

export function clearHistory(toolId?: string): Promise<void> {
  return invokeCmd<void>('clear_history', { req: { tool_id: toolId } });
}
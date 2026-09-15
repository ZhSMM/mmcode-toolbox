import { invokeCmd } from './index';
import type { Category, ToolMeta } from '@/types';

export function listCategories(): Promise<Category[]> {
  return invokeCmd<Category[]>('list_categories');
}

export function listTools(): Promise<ToolMeta[]> {
  return invokeCmd<ToolMeta[]>('list_tools');
}
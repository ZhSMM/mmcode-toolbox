import { invokeCmd } from './index';

export function listFavorites(): Promise<string[]> {
  return invokeCmd<string[]>('list_favorites');
}

export function toggleFavorite(toolId: string): Promise<boolean> {
  return invokeCmd<boolean>('toggle_favorite', { toolId });
}
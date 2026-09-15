import { defineStore } from 'pinia';
import { ref } from 'vue';

import * as toolsApi from '@/api/tools';
import * as favApi from '@/api/favorites';
import type { Category, ToolMeta, ToolGroup } from '@/types';

/**
 * 全局应用状态:
 *   - 菜单(分类 + 工具列表)
 *   - 收藏 tool_id 集合
 *   - 当前路由工具 ID (由 layout 写入)
 */
export const useAppStore = defineStore('app', () => {
  const categories = ref<Category[]>([]);
  const tools = ref<ToolMeta[]>([]);
  const favorites = ref<Set<string>>(new Set());
  const currentToolId = ref<string | null>(null);

  /** 按 category 分组的工具集合,用于侧边栏渲染 */
  const groupedTools = (): ToolGroup[] => {
    const catMap = new Map<string, Category>();
    for (const c of categories.value) catMap.set(c.id, c);

    const groupMap = new Map<string, ToolMeta[]>();
    for (const t of tools.value) {
      if (!groupMap.has(t.category)) groupMap.set(t.category, []);
      groupMap.get(t.category)!.push(t);
    }

    const out: ToolGroup[] = [];
    for (const c of [...categories.value].sort((a, b) => a.sort - b.sort)) {
      out.push({ category: c, tools: groupMap.get(c.id) ?? [] });
    }
    return out;
  };

  const loadMenu = async () => {
    const [cats, ts] = await Promise.all([
      toolsApi.listCategories(),
      toolsApi.listTools(),
    ]);
    categories.value = cats;
    tools.value = ts.sort((a, b) =>
      a.category.localeCompare(b.category) || a.sort_order - b.sort_order
    );
  };

  const loadFavorites = async () => {
    const ids = await favApi.listFavorites();
    favorites.value = new Set(ids);
  };

  const isFavorite = (toolId: string) => favorites.value.has(toolId);

  const toggleFavorite = async (toolId: string) => {
    const next = await favApi.toggleFavorite(toolId);
    if (next) favorites.value.add(toolId);
    else favorites.value.delete(toolId);
    // 触发响应式刷新
    favorites.value = new Set(favorites.value);
    return next;
  };

  return {
    categories,
    tools,
    favorites,
    currentToolId,
    groupedTools,
    loadMenu,
    loadFavorites,
    isFavorite,
    toggleFavorite,
  };
});
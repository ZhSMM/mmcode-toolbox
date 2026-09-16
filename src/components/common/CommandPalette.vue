<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useAppStore } from '@/stores/app';
import { copyText } from '@/utils/format';
import { applyTheme } from '@/utils/theme';
import { ElMessage } from 'element-plus';

/**
 * 全局命令面板 (Cmd/Ctrl + K)
 *
 * 设计:
 * - 顶部模糊搜索框,支持按工具名 / 类别 / tool_id 模糊匹配
 * - 列表里每项显示:图标 + 名称 + 分类 + tool_id
 * - Enter / 点击 → 跳到对应工具页
 * - Cmd/Ctrl + K  打开 / 关闭
 * - Esc 关闭
 * - 顶部下拉:可以选择「工具」「命令」,命令包括收藏、设置、刷新菜单、清除历史等
 */
const router = useRouter();
const appStore = useAppStore();

const visible = ref(false);
const query = ref('');
const activeIndex = ref(0);
const inputRef = ref<HTMLInputElement | null>(null);

type Command =
  | { kind: 'tool'; tool_id: string; name: string; category: string; icon: string | null }
  | { kind: 'cmd'; id: string; name: string; icon: string };

const commands = computed<Command[]>(() => {
  const tools: Command[] = appStore.tools.map((t) => ({
    kind: 'tool', tool_id: t.tool_id, name: t.name, category: t.category, icon: t.icon,
  }));
  const sys: Command[] = [
    { kind: 'cmd', id: 'go:home', name: '🏠 回到首页', icon: 'HomeFilled' },
    { kind: 'cmd', id: 'go:settings', name: '⚙️ 打开设置', icon: 'Setting' },
    { kind: 'cmd', id: 'app:reload-menu', name: '🔄 刷新菜单 / 收藏', icon: 'Refresh' },
    { kind: 'cmd', id: 'app:export-fav', name: '⭐ 导出收藏', icon: 'Download' },
    { kind: 'cmd', id: 'app:toggle-theme', name: '🌓 切换主题(跟随 / 亮 / 暗)', icon: 'Brush' },
  ];
  return [...tools, ...sys];
});

const filtered = computed(() => {
  const q = query.value.toLowerCase().trim();
  if (!q) return commands.value.slice(0, 30);
  const matched: { c: Command; score: number }[] = [];
  for (const c of commands.value) {
    const name = c.name.toLowerCase();
    const id = c.kind === 'tool' ? c.tool_id : c.id;
    let score = 0;
    if (name.includes(q)) score += 5;
    if (name.startsWith(q)) score += 10;
    if (id.toLowerCase().includes(q)) score += 8;
    if (score > 0) matched.push({ c, score });
  }
  matched.sort((a, b) => b.score - a.score);
  return matched.slice(0, 30).map((m) => m.c);
});

watch(filtered, () => { activeIndex.value = 0; });

const open = async () => {
  visible.value = true;
  query.value = '';
  activeIndex.value = 0;
  await nextTick();
  inputRef.value?.focus();
};
const close = () => { visible.value = false; };

const onKeydown = (e: KeyboardEvent) => {
  const isCmdK = (e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k';
  if (isCmdK) { e.preventDefault(); visible.value ? close() : open(); return; }
  if (!visible.value) return;
  if (e.key === 'Escape') { close(); }
  else if (e.key === 'ArrowDown') { e.preventDefault(); activeIndex.value = Math.min(activeIndex.value + 1, filtered.value.length - 1); }
  else if (e.key === 'ArrowUp') { e.preventDefault(); activeIndex.value = Math.max(activeIndex.value - 1, 0); }
  else if (e.key === 'Enter') {
    e.preventDefault();
    const c = filtered.value[activeIndex.value];
    if (c) execute(c);
  }
};

const execute = async (c: Command) => {
  close();
  if (c.kind === 'tool') {
    router.push(`/tools/${c.tool_id.replace(/-/g, '-')}`);
    // 上面是按 tool_id 直接构建路径;更稳的方式是按 tools 元数据里的 route 字段
    const t = appStore.tools.find((x) => x.tool_id === c.tool_id);
    if (t) router.push(t.route);
  } else {
    switch (c.id) {
      case 'go:home': router.push({ name: 'home' }); break;
      case 'go:settings': router.push({ name: 'settings' }); break;
      case 'app:reload-menu':
        await appStore.loadMenu();
        await appStore.loadFavorites();
        ElMessage.success('菜单已刷新');
        break;
      case 'app:export-fav': exportFavorites(); break;
      case 'app:toggle-theme': toggleTheme(); break;
    }
  }
};

const exportFavorites = () => {
  const data = {
      type: 'mmcode-favorites',
      version: 1,
      exported_at: new Date().toISOString(),
      tool_ids: [...appStore.favorites],
    };
  copyText(JSON.stringify(data, null, 2));
  ElMessage.success('收藏列表(JSON)已复制到剪贴板,可粘贴保存');
};

const toggleTheme = () => {
  const cur = localStorage.getItem('mmcode-theme') || 'system';
  const next = cur === 'system' ? 'light' : cur === 'light' ? 'dark' : 'system';
  applyTheme(next);
  ElMessage.success('主题: ' + (next === 'system' ? '跟随系统' : next === 'light' ? '浅色' : '深色'));
};

onMounted(() => {
  // 启动时应用已保存的主题
  const saved = (localStorage.getItem('mmcode-theme') as 'system' | 'light' | 'dark' | null) || 'system';
  applyTheme(saved);
  // 全局快捷键监听
  window.addEventListener('keydown', onKeydown);
});

defineExpose({ open, close });
</script>

<template>
  <el-dialog v-model="visible" :show-close="false" width="560px" align-center :modal-class="'command-palette-mask'" :style="{ borderRadius: '8px' }">
    <div style="padding:4px 8px 12px;">
      <el-input ref="inputRef" v-model="query" placeholder="搜索工具 / 命令..." size="large" clearable :prefix-icon="'Search'" />
    </div>
    <div style="max-height:360px;overflow:auto;">
      <div v-if="!filtered.length" style="padding:24px;text-align:center;color:#909399;">无匹配项</div>
      <div v-for="(c, i) in filtered" :key="(c as any).kind === 'tool' ? (c as any).tool_id : (c as any).id"
           :class="['cp-item', i === activeIndex ? 'active' : '']"
           @click="execute(c)"
           @mouseenter="activeIndex = i">
        <el-icon v-if="(c as any).icon" :size="18">
          <component :is="(c as any).icon" />
        </el-icon>
        <span style="flex:1;">{{ (c as any).name }}</span>
        <span v-if="c.kind === 'tool'" style="font-size:11px;color:#909399;font-family:monospace;">{{ (c as any).tool_id }}</span>
        <span v-if="c.kind === 'tool'" style="font-size:11px;padding:1px 6px;background:#f0f0f0;border-radius:3px;color:#606266;">{{ (c as any).category }}</span>
        <span v-else style="font-size:11px;padding:1px 6px;background:#ecf5ff;color:#409eff;border-radius:3px;">命令</span>
      </div>
    </div>
    <div style="border-top:1px solid #f0f0f0;padding:8px 12px;font-size:11px;color:#909399;display:flex;gap:16px;">
      <span>↑↓ 选择</span><span>Enter 执行</span><span>Esc 关闭</span><span>Cmd+K 切换</span>
    </div>
  </el-dialog>
</template>

<style scoped>
.cp-item {
  display: flex; align-items: center; gap: 10px;
  padding: 8px 12px; border-radius: 4px;
  cursor: pointer; user-select: none;
}
.cp-item:hover, .cp-item.active {
  background: #f5f7fa;
}
</style>
<script setup lang="ts">
/**
 * 工具页通用外壳。
 *
 * 用法:
 *   <ToolPanel tool-id="json-formatter" title="JSON 格式化" description="...">
 *     <template #options>...</template>
 *     <template #default>...输入/输出区...</template>
 *   </ToolPanel>
 *
 * 负责:
 *   - 标题 + 描述
 *   - 收藏按钮 (右上角)
 *   - 历史按钮 (右上角,带 Badge)
 *   - 历史抽屉 (右侧滑出)
 */
import { computed, ref, watch } from 'vue';
import { ElMessage } from 'element-plus';

import HistoryDrawer from './HistoryDrawer.vue';
import { useAppStore } from '@/stores/app';

const props = defineProps<{
  toolId: string;
  title: string;
  description?: string;
}>();

const appStore = useAppStore();

const favorited = computed(() => appStore.isFavorite(props.toolId));
const toggleFav = async () => {
  const next = await appStore.toggleFavorite(props.toolId);
  ElMessage.success(next ? '已加入收藏' : '已取消收藏');
};

// 历史抽屉
const historyVisible = ref(false);
const historyKey = ref(0); // 触发抽屉内重新拉取

const openHistory = () => {
  historyKey.value++;
  historyVisible.value = true;
};

const onPicked = (input: string) => {
  historyVisible.value = false;
  // 子组件监听 `historyPicked` 事件,从输入区中写入
  emit('pickHistory', input);
};

const emit = defineEmits<{
  pickHistory: [input: string];
}>();
</script>

<template>
  <div class="tool-panel">
    <header class="tool-panel__head">
      <div>
        <h2>{{ title }}</h2>
        <p v-if="description" class="desc">{{ description }}</p>
      </div>
      <div class="actions" style="display:flex;gap:8px;">
        <el-button :icon="favorited ? 'StarFilled' : 'Star'" @click="toggleFav">
          {{ favorited ? '已收藏' : '收藏' }}
        </el-button>
        <el-button icon="Clock" @click="openHistory">历史</el-button>
      </div>
    </header>

    <slot name="options" />

    <div class="tool-panel__body">
      <slot />
    </div>

    <HistoryDrawer
      :key="historyKey"
      v-model:visible="historyVisible"
      :tool-id="toolId"
      @pick="onPicked"
    />
  </div>
</template>
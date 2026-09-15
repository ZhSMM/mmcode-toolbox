<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';

import * as historyApi from '@/api/history';
import type { HistoryItem } from '@/types';
import { relativeTime, truncate } from '@/utils/format';

const props = defineProps<{
  visible: boolean;
  toolId: string;
}>();

const emit = defineEmits<{
  'update:visible': [v: boolean];
  pick: [input: string];
}>();

const items = ref<HistoryItem[]>([]);
const loading = ref(false);

const refresh = async () => {
  loading.value = true;
  try {
    items.value = await historyApi.listHistory(props.toolId, 50);
  } catch (e: any) {
    ElMessage.error('加载历史失败: ' + (e?.message ?? e));
  } finally {
    loading.value = false;
  }
};

watch(
  () => props.visible,
  (v) => {
    if (v) refresh();
  }
);

onMounted(() => {
  if (props.visible) refresh();
});

const close = () => emit('update:visible', false);

const handlePick = (item: HistoryItem) => {
  if (item.input == null) {
    ElMessage.warning('该历史记录没有输入可回填');
    return;
  }
  emit('pick', item.input);
};

const handleDelete = async (item: HistoryItem) => {
  try {
    await ElMessageBox.confirm('确定删除该条历史？', '删除确认', {
      type: 'warning',
    });
    await historyApi.deleteHistory(item.id);
    items.value = items.value.filter((x) => x.id !== item.id);
    ElMessage.success('已删除');
  } catch (e: any) {
    if (e === 'cancel') return;
    ElMessage.error('删除失败: ' + (e?.message ?? e));
  }
};

const handleClearAll = async () => {
  try {
    await ElMessageBox.confirm(`清空 ${props.toolId} 的全部历史？`, '清空确认', {
      type: 'warning',
    });
    await historyApi.clearHistory(props.toolId);
    items.value = [];
    ElMessage.success('已清空');
  } catch (e: any) {
    if (e === 'cancel') return;
    ElMessage.error('清空失败: ' + (e?.message ?? e));
  }
};
</script>

<template>
  <el-drawer
    :model-value="visible"
    title="历史记录"
    direction="rtl"
    size="420px"
    @update:model-value="emit('update:visible', $event)"
  >
    <div style="display:flex;justify-content:space-between;margin-bottom:12px;">
      <span style="color:var(--text-secondary);font-size:13px;">
        共 {{ items.length }} 条
      </span>
      <el-button
        size="small"
        type="danger"
        text
        :disabled="!items.length"
        @click="handleClearAll"
      >
        清空全部
      </el-button>
    </div>

    <el-skeleton v-if="loading" :rows="3" animated />

    <el-empty v-else-if="!items.length" description="暂无历史" :image-size="60" />

    <el-timeline v-else>
      <el-timeline-item
        v-for="item in items"
        :key="item.id"
        :type="item.status === 'error' ? 'danger' : 'primary'"
        :timestamp="relativeTime(item.created_at)"
        placement="top"
      >
        <el-card shadow="never" style="border-radius:6px;">
          <div style="display:flex;justify-content:space-between;gap:8px;">
            <span style="font-size:12px;color:var(--text-secondary);">
              {{ item.status === 'error' ? '失败' : '成功' }}
            </span>
            <span style="display:flex;gap:4px;">
              <el-button size="small" text type="primary" @click="handlePick(item)">
                回填
              </el-button>
              <el-button size="small" text type="danger" @click="handleDelete(item)">
                删除
              </el-button>
            </span>
          </div>
          <pre
            v-if="item.input"
            style="margin:6px 0 0;font-size:12px;color:var(--text-secondary);white-space:pre-wrap;word-break:break-all;"
          >输入: {{ truncate(item.input, 200) }}</pre>
          <pre
            v-if="item.output"
            style="margin:4px 0 0;font-size:12px;color:var(--text-primary);white-space:pre-wrap;word-break:break-all;"
          >输出: {{ truncate(item.output, 200) }}</pre>
          <pre
            v-if="item.error_msg"
            style="margin:4px 0 0;font-size:12px;color:#f56c6c;"
          >错误: {{ item.error_msg }}</pre>
        </el-card>
      </el-timeline-item>
    </el-timeline>
  </el-drawer>
</template>
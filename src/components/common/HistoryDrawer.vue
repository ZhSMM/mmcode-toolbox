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
// 标记正在删除的单条 id,用于单独显示该条的 loading
const deletingId = ref<number | null>(null);
const clearingAll = ref(false);

/**
 * Element Plus 的 ElMessageBox 在用户关闭弹窗时 reject 的字符串值:
 *   'cancel' - 用户点了取消按钮或按 Esc
 *   'close'  - 用户点了右上角 X 或遮罩关闭
 * 这些都是"用户主动取消",不应该当作错误处理。
 */
function isUserCancelled(e: unknown): boolean {
  return typeof e === 'string' && (e === 'cancel' || e === 'close');
}

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

/**
 * 删除单条历史。
 *
 * 弹窗显示:
 *   - 标题: 「删除这条历史」
 *   - 内容: 包含时间 + 输入/输出预览(让用户确认是哪条)
 *   - 确认按钮: 「删除」(红色 danger)
 *   - 取消按钮: 「取消」
 *   - 锁定中: 确认按钮 loading,避免重复点击
 */
const handleDelete = async (item: HistoryItem) => {
  // 构造更明确的弹窗内容,带上时间 + 输入预览
  const inputPreview = item.input ? truncate(item.input, 60) : '(无输入)';
  const time = relativeTime(item.created_at);

  try {
    await ElMessageBox.confirm(
      `<div style="line-height:1.6;">
        <div style="margin-bottom:6px;">确定删除这条历史吗?此操作不可撤销。</div>
        <div style="font-size:12px;color:#909399;">
          <div>时间: <b>${time}</b></div>
          <div>状态: ${item.status === 'error' ? '<span style="color:#f56c6c;">失败</span>' : '成功'}</div>
          <div style="margin-top:4px;max-height:80px;overflow:auto;background:#f5f7fa;padding:6px 8px;border-radius:4px;font-family:monospace;font-size:11px;white-space:pre-wrap;word-break:break-all;">
            ${inputPreview.replace(/</g, '&lt;')}
          </div>
        </div>
      </div>`,
      '删除这条历史',
      {
        type: 'warning',
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        confirmButtonClass: 'el-button--danger',
        dangerouslyUseHTMLString: true,
        // 自定义宽度让预览有空间
        customStyle: { maxWidth: '420px' },
      }
    );

    deletingId.value = item.id;
    try {
      await historyApi.deleteHistory(item.id);
      items.value = items.value.filter((x) => x.id !== item.id);
      ElMessage.success('已删除');
    } finally {
      deletingId.value = null;
    }
  } catch (e: any) {
    if (isUserCancelled(e)) return;  // 用户主动取消,X / Esc / 取消按钮
    ElMessage.error('删除失败: ' + (e?.message ?? String(e)));
  }
};

/**
 * 清空全部历史。
 *
 * 弹窗显示:
 *   - 标题: 「清空全部历史」
 *   - 内容: 明确警告「不可恢复」+ 当前条数
 *   - 要求二次确认: 输入 DELETE 才允许
 */
const handleClearAll = async () => {
  if (!items.value.length) {
    ElMessage.info('当前工具暂无历史');
    return;
  }
  const count = items.value.length;

  try {
    await ElMessageBox.confirm(
      `<div style="line-height:1.6;">
        <div>将永久删除该工具的 <b style="color:#f56c6c;">${count}</b> 条历史记录,此操作不可撤销。</div>
        <div style="margin-top:8px;font-size:12px;color:#909399;">
          推荐:先用收藏与导出功能备份高频工具的历史。
        </div>
      </div>`,
      '清空全部历史',
      {
        type: 'warning',
        confirmButtonText: '清空全部',
        cancelButtonText: '取消',
        confirmButtonClass: 'el-button--danger',
        dangerouslyUseHTMLString: true,
        customStyle: { maxWidth: '420px' },
      }
    );

    clearingAll.value = true;
    try {
      await historyApi.clearHistory(props.toolId);
      items.value = [];
      ElMessage.success(`已清空 ${count} 条历史`);
    } finally {
      clearingAll.value = false;
    }
  } catch (e: any) {
    if (isUserCancelled(e)) return;
    ElMessage.error('清空失败: ' + (e?.message ?? String(e)));
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
    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">
      <span style="color:var(--text-secondary);font-size:13px;">
        共 {{ items.length }} 条
      </span>
      <el-button
        size="small"
        type="danger"
        text
        :disabled="!items.length || clearingAll"
        :loading="clearingAll"
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
          <div style="display:flex;justify-content:space-between;align-items:center;gap:8px;">
            <span style="font-size:12px;color:var(--text-secondary);">
              <el-tag
                :type="item.status === 'error' ? 'danger' : 'success'"
                size="small"
                effect="plain"
                style="margin-right:4px;"
              >
                {{ item.status === 'error' ? '失败' : '成功' }}
              </el-tag>
            </span>
            <span style="display:flex;gap:4px;">
              <el-button size="small" text type="primary" @click="handlePick(item)">
                回填
              </el-button>
              <el-button
                size="small"
                text
                type="danger"
                :loading="deletingId === item.id"
                @click="handleDelete(item)"
              >
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
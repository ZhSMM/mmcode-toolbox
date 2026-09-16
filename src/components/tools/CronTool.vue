<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/cron';

const TOOL_ID = 'cron';
const expr = ref('*/15 * * * *');
const count = ref(5);
const result = ref<any>(null);
const busy = ref(false);

const parse = async () => {
  busy.value = true;
  try {
    result.value = await api.cronNext({ expression: expr.value, count: count.value });
  } catch (e: any) {
    ElMessage.error('解析失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};
parse();

const fmt = (iso: string) => {
  const d = new Date(iso);
  return `${d.toLocaleString()}  (${iso})`;
};
</script>

<template>
  <ToolPanel tool-id="cron" title="Cron 解析" description="5 字段标准 cron 表达式,预测接下来 N 次执行。">
    <template #options>
      <div class="tool-panel__options">
        <el-input v-model="expr" placeholder="*/5 * * * *" style="width:240px;" />
        <span style="color:#606266;font-size:13px;">次数</span>
        <el-input-number v-model="count" :min="1" :max="20" size="small" style="width:120px;" />
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="parse">解析</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane" style="grid-column:span 2;">
        <div class="tool-panel__pane-header">
          <span>
            结果
            <span v-if="result?.valid === false" style="color:#f56c6c;margin-left:8px;">{{ result.error }}</span>
          </span>
        </div>
        <div class="tool-panel__pane-body" style="padding:12px;">
          <div v-if="!result" style="color:#909399;">点击「解析」</div>
          <div v-else-if="result.valid === false" style="color:#f56c6c;padding:8px;">非法: {{ result.error }}</div>
          <ol v-else style="line-height:1.8;padding-left:24px;">
            <li v-for="(r, i) in result.next_runs" :key="i" style="font-family:monospace;font-size:13px;">
              <span style="display:inline-block;width:24px;color:#999;">#{{ i + 1 }}</span> {{ fmt(r) }}
            </li>
          </ol>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/uuid';
import * as historyApi from '@/api/history';
import { copyText } from '@/utils/format';

const TOOL_ID = 'uuid';
const count = ref(5);
const version = ref<'v4' | 'v7' | 'nil'>('v4');
const uppercase = ref(false);
const withHyphens = ref(true);
const items = ref<string[]>([]);
const busy = ref(false);

const generate = async () => {
  busy.value = true;
  try {
    const r = await api.uuidGenerate({ count: count.value, version: version.value, uppercase: uppercase.value, with_hyphens: withHyphens.value });
    items.value = r.items;
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: `${version.value} × ${count.value}`, output: r.items.join('\n'), status: 'success' });
  } catch (e: any) {
    ElMessage.error('生成失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};
const copyAll = async () => {
  if (!items.value.length) { ElMessage.warning('暂无 UUID'); return; }
  await copyText(items.value.join('\n'));
  ElMessage.success(`已复制 ${items.value.length} 个`);
};
const copyOne = async (s: string) => { await copyText(s); ElMessage.success('已复制'); };
</script>

<template>
  <ToolPanel tool-id="uuid" title="UUID 生成" description="v4(随机) / v7(时间排序) / nil(全零)。">
    <template #options>
      <div class="tool-panel__options">
        <span style="color:#606266;font-size:13px;">数量</span>
        <el-input-number v-model="count" :min="1" :max="500" size="small" style="width:120px;" />
        <el-radio-group v-model="version" size="small">
          <el-radio-button value="v4">v4 随机</el-radio-button>
          <el-radio-button value="v7">v7 时序</el-radio-button>
          <el-radio-button value="nil">nil</el-radio-button>
        </el-radio-group>
        <el-checkbox v-model="withHyphens">带连字符</el-checkbox>
        <el-checkbox v-model="uppercase">大写</el-checkbox>
        <div style="flex:1" />
        <el-button :icon="'CopyDocument'" :disabled="!items.length" @click="copyAll">复制全部</el-button>
        <el-button type="primary" :loading="busy" @click="generate">生成</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane" style="grid-column:span 2;">
        <div class="tool-panel__pane-header">
          <span>结果 ({{ items.length }} 个)</span>
        </div>
        <div class="tool-panel__pane-body mono-input" style="padding:8px 12px;overflow:auto;">
          <div v-if="!items.length" style="color:#909399;padding:8px;">点击「生成」</div>
          <div v-for="(s, i) in items" :key="i" style="display:flex;align-items:center;gap:8px;padding:4px 0;border-bottom:1px solid #f0f0f0;">
            <code style="flex:1;font-family:monospace;">{{ s }}</code>
            <el-button size="small" text @click="copyOne(s)">复制</el-button>
          </div>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
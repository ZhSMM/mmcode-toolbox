<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/sql-format';
import { copyText } from '@/utils/format';

const TOOL_ID = 'sql-format';
const input = ref(`select id, name, email from users where active = true and created_at > '2024-01-01' order by created_at desc limit 100;`);
const output = ref('');
const indent = ref(2);
const uppercase = ref(true);
const minify = ref(false);
const busy = ref(false);

const run = async () => {
  if (!input.value) return;
  busy.value = true;
  try {
    const r = await api.sqlFormat({ input: input.value, indent: indent.value, uppercase: uppercase.value, minify: minify.value });
    output.value = r.output;
  } catch (e: any) {
    ElMessage.error('格式化失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};
const copy = async () => {
  if (!output.value) return;
  await copyText(output.value);
  ElMessage.success('已复制');
};
</script>

<template>
  <ToolPanel tool-id="sql-format" title="SQL 格式化" description="基于 sqlformat,支持美化/压缩、大写关键字。">
    <template #options>
      <div class="tool-panel__options">
        <span style="color:#606266;font-size:13px;">缩进</span>
        <el-input-number v-model="indent" :min="1" :max="8" size="small" style="width:120px;" />
        <el-checkbox v-model="uppercase">关键字大写</el-checkbox>
        <el-checkbox v-model="minify">压缩为单行</el-checkbox>
        <div style="flex:1" />
        <el-button :icon="'CopyDocument'" :disabled="!output" @click="copy">复制</el-button>
        <el-button type="primary" :loading="busy" @click="run">格式化</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>输入 SQL</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="input" type="textarea" :rows="22" resize="none" spellcheck="false" />
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>输出</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="output" type="textarea" :rows="22" readonly spellcheck="false" />
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
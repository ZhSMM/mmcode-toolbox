<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/regex';
import * as historyApi from '@/api/history';
import { copyText } from '@/utils/format';

const TOOL_ID = 'regex-tester';
const pattern = ref('\\b\\w+@\\w+\\.\\w+\\b');
const input = ref('联系我们:support@example.com 或 sales@mavis.io,不要给 admin@internal 写信。');
const flags = ref<string[]>(['g']);
const matches = ref<any[]>([]);
const error = ref<string | null>(null);
const valid = ref(true);
const replacement = ref('');
const busy = ref(false);

const run = async () => {
  busy.value = true;
  try {
    const r = await api.regexTest({ pattern: pattern.value, input: input.value, flags: flags.value });
    valid.value = r.valid;
    error.value = r.error;
    matches.value = r.matches;
    if (!r.valid) ElMessage.warning('正则非法: ' + r.error);
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: `${pattern.value} on ${input.value.substring(0, 200)}`, output: `${r.matches.length} 匹配`, status: r.valid ? 'success' : 'error', error_msg: r.error ?? undefined });
  } catch (e: any) {
    ElMessage.error('执行失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};

const runReplace = async () => {
  if (!valid.value) { ElMessage.warning('先修正正则'); return; }
  busy.value = true;
  try {
    const out = await api.regexReplace(pattern.value, input.value, replacement.value, flags.value);
    input.value = out;
    ElMessage.success('替换完成');
  } catch (e: any) {
    ElMessage.error('替换失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};
</script>

<template>
  <ToolPanel tool-id="regex-tester" title="正则测试" description="Rust regex 语法,支持 i/m/s/x/u 标志。">
    <template #options>
      <div class="tool-panel__options">
        <el-input v-model="pattern" placeholder="正则表达式" style="width:300px" />
        <el-checkbox-group v-model="flags" size="small">
          <el-checkbox-button value="i">i</el-checkbox-button>
          <el-checkbox-button value="m">m</el-checkbox-button>
          <el-checkbox-button value="s">s</el-checkbox-button>
          <el-checkbox-button value="x">x</el-checkbox-button>
          <el-checkbox-button value="u">u</el-checkbox-button>
          <el-checkbox-button value="g">g</el-checkbox-button>
        </el-checkbox-group>
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="run">测试</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>输入文本 ({{ input.length }} 字符)</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="input" type="textarea" :rows="20" resize="none" spellcheck="false" />
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span>匹配结果 ({{ matches.length }})</span>
        </div>
        <div class="tool-panel__pane-body" style="padding:8px 12px;overflow:auto;">
          <div v-if="error" style="color:#f56c6c;padding:8px;">{{ error }}</div>
          <div v-else-if="!matches.length" style="color:#909399;padding:8px;">无匹配</div>
          <div v-for="(m, i) in matches" :key="i" style="border-bottom:1px solid #eee;padding:6px 0;">
            <div><span style="background:#67c23a;color:#fff;padding:1px 4px;border-radius:2px;font-size:11px;">[{{ m.start }}, {{ m.end }})</span> <strong>{{ m.text }}</strong></div>
            <div v-if="m.groups.length > 1" style="margin-top:4px;color:#666;font-size:12px;">
              <span v-for="(g, j) in m.groups.slice(1)" :key="j" style="margin-right:8px;">${{ j + 1 }}=<code>{{ g ?? '∅' }}</code></span>
            </div>
          </div>
        </div>
        <div class="tool-panel__pane-header">
          <el-input v-model="replacement" placeholder="替换为..." size="small" style="flex:1;margin-right:8px;" />
          <el-button size="small" @click="runReplace">替换</el-button>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
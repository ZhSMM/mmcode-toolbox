<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/hex';
import * as historyApi from '@/api/history';
import { copyText, truncate } from '@/utils/format';

const TOOL_ID = 'hex';
const input = ref('Hello, Mavis!');
const output = ref('');
const mode = ref<'encode' | 'decode' | 'hexdump'>('encode');
const uppercase = ref(false);
const busy = ref(false);

const run = async () => {
  if (!input.value && mode.value !== 'encode') { ElMessage.warning('请输入内容'); return; }
  busy.value = true;
  try {
    const fn = mode.value === 'encode' ? api.hexEncode : mode.value === 'decode' ? api.hexDecode : api.hexDump;
    const r = await fn({ input: input.value, mode: mode.value, uppercase: uppercase.value });
    output.value = r.output;
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: input.value, output: r.output, status: 'success' });
  } catch (e: any) {
    const msg = e?.message ?? String(e);
    ElMessage.error(`执行失败: ${msg}`);
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: truncate(input.value, 8000), output: null, status: 'error', error_msg: msg }).catch(() => undefined);
  } finally { busy.value = false; }
};

const copyOutput = async () => {
  if (!output.value) { ElMessage.warning('暂无输出'); return; }
  const ok = await copyText(output.value);
  ElMessage[ok ? 'success' : 'error'](ok ? '已复制' : '复制失败');
};
const loadFromHistory = (v: string) => { input.value = v; run(); };
</script>

<template>
  <ToolPanel tool-id="hex" title="Hex 转换" description="UTF-8 ↔ hex,以及十六进制 dump。" @pick-history="loadFromHistory">
    <template #options>
      <div class="tool-panel__options">
        <el-radio-group v-model="mode" size="small">
          <el-radio-button value="encode">encode</el-radio-button>
          <el-radio-button value="decode">decode</el-radio-button>
          <el-radio-button value="hexdump">hexdump</el-radio-button>
        </el-radio-group>
        <el-checkbox v-model="uppercase">大写</el-checkbox>
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="run">执行</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>输入</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="input" type="textarea" :rows="22" resize="none" spellcheck="false" />
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span>输出</span>
          <el-button size="small" text @click="copyOutput">复制</el-button>
        </div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="output" type="textarea" :rows="22" readonly spellcheck="false" />
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
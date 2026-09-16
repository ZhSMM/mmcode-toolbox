<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/url-codec';
import * as historyApi from '@/api/history';
import { copyText, truncate } from '@/utils/format';

const TOOL_ID = 'url-codec';
const input = ref('https://example.com/search?q=你好 世界&lang=zh-CN');
const output = ref('');
const mode = ref<'encode' | 'decode'>('encode');
const codecMode = ref<'query' | 'component'>('query');
const busy = ref(false);

const run = async () => {
  if (!input.value) { ElMessage.warning('请输入内容'); return; }
  busy.value = true;
  try {
    const fn = mode.value === 'encode' ? api.urlEncode : api.urlDecode;
    const r = await fn({ input: input.value, mode: codecMode.value });
    output.value = r.output;
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: input.value, output: r.output, status: 'success' });
  } catch (e: any) {
    const msg = e?.message ?? String(e);
    ElMessage.error(`${mode.value === 'encode' ? '编码' : '解码'}失败: ${msg}`);
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: truncate(input.value, 8000), output: null, status: 'error', error_msg: msg }).catch(() => undefined);
  } finally { busy.value = false; }
};

const copyOutput = async () => {
  if (!output.value) { ElMessage.warning('暂无输出'); return; }
  ElMessage[await copyText(output.value) ? 'success' : 'error'](await copyText(output.value) ? '已复制' : '复制失败');
};
const swap = () => { mode.value = mode.value === 'encode' ? 'decode' : 'encode'; };
const loadFromHistory = (v: string) => { input.value = v; run(); };
</script>

<template>
  <ToolPanel tool-id="url-codec" title="URL 编解码" description="Percent-encoding。支持 query 与 component 两种字符集。"
    @pick-history="loadFromHistory">
    <template #options>
      <div class="tool-panel__options">
        <el-radio-group v-model="mode" size="small">
          <el-radio-button value="encode">编码</el-radio-button>
          <el-radio-button value="decode">解码</el-radio-button>
        </el-radio-group>
        <el-radio-group v-model="codecMode" size="small">
          <el-radio-button value="query">query</el-radio-button>
          <el-radio-button value="component">component</el-radio-button>
        </el-radio-group>
        <div style="flex:1" />
        <el-button :icon="'Refresh'" @click="swap">互换</el-button>
        <el-button type="primary" :loading="busy" @click="run">{{ mode === 'encode' ? '编码' : '解码' }}</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>原文</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="input" type="textarea" :rows="20" resize="none" spellcheck="false" />
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span>输出</span>
          <el-button size="small" text @click="copyOutput">复制</el-button>
        </div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="output" type="textarea" :rows="20" readonly spellcheck="false" />
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
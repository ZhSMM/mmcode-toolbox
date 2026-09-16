<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/qrcode';
import * as historyApi from '@/api/history';

const TOOL_ID = 'qrcode';
const input = ref('https://github.com/ZhSMM/mmcode-toolbox');
const dark = ref('#000000');
const light = ref('#ffffff');
const result = ref<{ svg: string; modules: number; version: string } | null>(null);
const busy = ref(false);

const generate = async () => {
  if (!input.value) { ElMessage.warning('请输入内容'); return; }
  busy.value = true;
  try {
    result.value = await api.qrcodeGenerate({ input: input.value, dark: dark.value, light: light.value });
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: input.value.substring(0, 200), output: `${result.value.modules}x${result.value.modules} v${result.value.version}`, status: 'success' });
  } catch (e: any) {
    ElMessage.error('生成失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};

const download = () => {
  if (!result.value) return;
  const blob = new Blob([result.value.svg], { type: 'image/svg+xml' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'qrcode.svg';
  a.click();
  URL.revokeObjectURL(url);
};
</script>

<template>
  <ToolPanel tool-id="qrcode" title="二维码生成" description="输出 SVG,可直接保存或嵌入。">
    <template #options>
      <div class="tool-panel__options">
        <el-color-picker v-model="dark" size="small" />
        <span style="color:#606266;font-size:13px;">前景 / 背景</span>
        <el-color-picker v-model="light" size="small" />
        <div style="flex:1" />
        <el-button :icon="'Download'" :disabled="!result" @click="download">下载 SVG</el-button>
        <el-button type="primary" :loading="busy" @click="generate">生成</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>输入内容</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="input" type="textarea" :rows="22" resize="none" spellcheck="false" />
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span>预览</span>
          <span v-if="result" style="font-size:12px;color:#909399;">{{ result.modules }}×{{ result.modules }} · 版本 {{ result.version }}</span>
        </div>
        <div class="tool-panel__pane-body" style="display:flex;align-items:center;justify-content:center;padding:20px;">
          <div v-if="!result" style="color:#909399;">点击「生成」</div>
          <div v-else v-html="result.svg" style="max-width:90%;max-height:90%;" />
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
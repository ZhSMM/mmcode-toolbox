<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';

import ToolPanel from '@/components/common/ToolPanel.vue';
import * as base64Api from '@/api/base64';
import * as historyApi from '@/api/history';
import { copyText, truncate } from '@/utils/format';

const TOOL_ID = 'base64';

const mode = ref<'encode' | 'decode'>('encode');
const input = ref<string>('Hello, Mavis Code Toolbox!');
const output = ref<string>('');
const urlSafe = ref<boolean>(false);
const busy = ref(false);

const run = async () => {
  if (!input.value) {
    ElMessage.warning('请输入内容');
    return;
  }
  busy.value = true;
  try {
    const fn = mode.value === 'encode' ? base64Api.base64Encode : base64Api.base64Decode;
    const r = await fn({ input: input.value, url_safe: urlSafe.value });
    output.value = r.output;
    await historyApi.saveHistory({
      tool_id: TOOL_ID,
      input: input.value,
      output: r.output,
      status: 'success',
    });
  } catch (e: any) {
    output.value = '';
    const msg = e?.message ?? String(e);
    ElMessage.error(`${mode.value === 'encode' ? '编码' : '解码'}失败: ${msg}`);
    await historyApi
      .saveHistory({
        tool_id: TOOL_ID,
        input: truncate(input.value, 8000),
        output: null,
        status: 'error',
        error_msg: msg,
      })
      .catch(() => undefined);
  } finally {
    busy.value = false;
  }
};

const swap = () => {
  // 输入框 <-> 输出框 互换
  const t = input.value;
  input.value = output.value;
  output.value = t;
  mode.value = mode.value === 'encode' ? 'decode' : 'encode';
};

const clearAll = () => {
  input.value = '';
  output.value = '';
};

const copyOutput = async () => {
  if (!output.value) {
    ElMessage.warning('暂无输出可复制');
    return;
  }
  const ok = await copyText(output.value);
  ElMessage[ok ? 'success' : 'error'](ok ? '已复制到剪贴板' : '复制失败');
};

const loadFromHistory = (v: string) => {
  input.value = v;
  run();
};
</script>

<template>
  <ToolPanel
    :tool-id="TOOL_ID"
    title="Base64 编解码"
    description="支持标准 Base64 与 URL-safe(无 padding)变体。Rust 端使用 base64 引擎 + 严格 UTF-8 校验。"
    @pick-history="loadFromHistory"
  >
    <template #options>
      <div class="tool-panel__options">
        <el-radio-group v-model="mode" size="small">
          <el-radio-button value="encode">编码</el-radio-button>
          <el-radio-button value="decode">解码</el-radio-button>
        </el-radio-group>

        <el-checkbox v-model="urlSafe">URL-safe (无 padding)</el-checkbox>

        <div style="flex:1" />

        <el-button :icon="mode === 'encode' ? 'Promotion' : 'Refresh'" @click="swap">
          互换输入/输出
        </el-button>
        <el-button type="primary" :loading="busy" @click="run">
          {{ mode === 'encode' ? '编码' : '解码' }}
        </el-button>
        <el-button icon="Delete" @click="clearAll">清空</el-button>
      </div>
    </template>

    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span>
            {{ mode === 'encode' ? '原文输入' : 'Base64 输入' }}
          </span>
          <span style="font-size:11px;color:var(--text-secondary);">
            {{ input.length }} 字符
          </span>
        </div>
        <div class="tool-panel__pane-body mono-input">
          <el-input
            v-model="input"
            type="textarea"
            resize="none"
            :rows="20"
            spellcheck="false"
          />
        </div>
      </div>

      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span>{{ mode === 'encode' ? 'Base64 输出' : '解码输出' }}</span>
          <el-button size="small" text icon="CopyDocument" @click="copyOutput">
            复制
          </el-button>
        </div>
        <div class="tool-panel__pane-body mono-input">
          <el-input
            v-model="output"
            type="textarea"
            resize="none"
            :rows="20"
            readonly
            spellcheck="false"
          />
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';

import ToolPanel from '@/components/common/ToolPanel.vue';
import * as jsonApi from '@/api/json';
import * as historyApi from '@/api/history';
import { copyText, truncate } from '@/utils/format';

const TOOL_ID = 'json-formatter';

const input = ref<string>(`{
  "name": "Mavis Code Toolbox",
  "version": "0.1.0",
  "features": ["JSON", "Base64", "Crypto", "Generator"],
  "active": true,
  "author": { "name": "Mavis", "role": "developer" }
}`);
const output = ref<string>('');
const indent = ref<number>(2);
const sortKeys = ref<boolean>(true);
const escapeUnicode = ref<boolean>(false);
const busy = ref(false);
const duration = ref<number | null>(null);

const runFormat = async () => {
  if (!input.value.trim()) {
    ElMessage.warning('请输入 JSON 内容');
    return;
  }
  busy.value = true;
  try {
    const r = await jsonApi.jsonFormat({
      input: input.value,
      indent: indent.value,
      sort_keys: sortKeys.value,
      escape_unicode: escapeUnicode.value,
    });
    output.value = r.output;
    duration.value = r.duration_ms;
    await historyApi.saveHistory({
      tool_id: TOOL_ID,
      input: input.value,
      output: r.output,
      status: 'success',
    });
  } catch (e: any) {
    output.value = '';
    duration.value = null;
    const msg = e?.message ?? String(e);
    ElMessage.error('格式化失败: ' + msg);
    await historyApi.saveHistory({
      tool_id: TOOL_ID,
      input: truncate(input.value, 8000),
      output: null,
      status: 'error',
      error_msg: msg,
    }).catch(() => undefined);
  } finally {
    busy.value = false;
  }
};

const runMinify = async () => {
  if (!input.value.trim()) {
    ElMessage.warning('请输入 JSON 内容');
    return;
  }
  busy.value = true;
  try {
    const r = await jsonApi.jsonMinify({
      input: input.value,
      indent: 0,
      sort_keys: sortKeys.value,
      escape_unicode: escapeUnicode.value,
    });
    output.value = r.output;
    duration.value = r.duration_ms;
    await historyApi.saveHistory({
      tool_id: TOOL_ID,
      input: input.value,
      output: r.output,
      status: 'success',
    });
  } catch (e: any) {
    output.value = '';
    duration.value = null;
    const msg = e?.message ?? String(e);
    ElMessage.error('压缩失败: ' + msg);
  } finally {
    busy.value = false;
  }
};

const clearAll = () => {
  input.value = '';
  output.value = '';
  duration.value = null;
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
  // 自动再跑一次
  runFormat();
};
</script>

<template>
  <ToolPanel
    :tool-id="TOOL_ID"
    title="JSON 格式化"
    description="格式化、压缩、键排序、Unicode 转义。Rust 端使用 serde_json 解析,失败会写入错误历史。"
    @pick-history="loadFromHistory"
  >
    <template #options>
      <div class="tool-panel__options">
        <span style="color:var(--text-secondary);font-size:13px;">缩进:</span>
        <el-radio-group v-model="indent" size="small">
          <el-radio-button :value="0">0</el-radio-button>
          <el-radio-button :value="2">2</el-radio-button>
          <el-radio-button :value="4">4</el-radio-button>
          <el-radio-button :value="8">8</el-radio-button>
        </el-radio-group>

        <el-checkbox v-model="sortKeys">按键名排序</el-checkbox>
        <el-checkbox v-model="escapeUnicode">Unicode 转义</el-checkbox>

        <div style="flex:1" />

        <el-button
          type="primary"
          :loading="busy"
          @click="runFormat"
          icon="MagicStick"
        >
          格式化
        </el-button>
        <el-button :loading="busy" @click="runMinify" icon="Compress">
          压缩
        </el-button>
        <el-button @click="clearAll" icon="Delete">清空</el-button>
      </div>
    </template>

    <template #default>
      <!-- 输入面板 -->
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span>输入</span>
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
            placeholder="粘贴或输入 JSON..."
            spellcheck="false"
          />
        </div>
      </div>

      <!-- 输出面板 -->
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span>
            输出
            <span
              v-if="duration != null"
              style="margin-left:8px;font-size:11px;color:var(--text-secondary);"
            >
              耗时 {{ duration }} ms
            </span>
          </span>
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
            placeholder="结果会在这里显示..."
            spellcheck="false"
          />
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
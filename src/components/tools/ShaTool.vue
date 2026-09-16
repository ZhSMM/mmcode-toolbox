<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/sha';
import * as historyApi from '@/api/history';
import { copyText } from '@/utils/format';

const TOOL_ID = 'sha';
const input = ref('Hello, Mavis!');
const algorithm = ref<'sha1' | 'sha256' | 'sha512'>('sha256');
const uppercase = ref(false);
const out = ref<{ hex: string; base64: string } | null>(null);
const busy = ref(false);

const run = async () => {
  busy.value = true;
  try {
    out.value = await api.shaHash({ input: input.value, algorithm: algorithm.value, uppercase: uppercase.value });
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: input.value, output: out.value.hex, status: 'success' });
  } catch (e: any) {
    ElMessage.error('执行失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};
const copy = async (s: string) => { await copyText(s); ElMessage.success('已复制'); };
const loadFromHistory = (v: string) => { input.value = v; run(); };
</script>

<template>
  <ToolPanel tool-id="sha" title="SHA-1 / 256 / 512" description="安全散列家族。" @pick-history="loadFromHistory">
    <template #options>
      <div class="tool-panel__options">
        <el-radio-group v-model="algorithm" size="small">
          <el-radio-button value="sha1">SHA-1</el-radio-button>
          <el-radio-button value="sha256">SHA-256</el-radio-button>
          <el-radio-button value="sha512">SHA-512</el-radio-button>
        </el-radio-group>
        <el-checkbox v-model="uppercase">大写</el-checkbox>
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="run">计算</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>输入</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="input" type="textarea" :rows="20" resize="none" spellcheck="false" />
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>{{ algorithm.toUpperCase() }} 输出</span></div>
        <div class="tool-panel__pane-body" style="padding:12px;">
          <div v-if="!out" style="color:#909399;">点击「计算」</div>
          <div v-else>
            <div style="margin-bottom:8px;color:#606266;font-size:12px;">hex</div>
            <div style="display:flex;align-items:center;gap:8px;margin-bottom:12px;">
              <code style="flex:1;padding:6px 10px;background:#f5f7fa;border-radius:4px;word-break:break-all;font-family:monospace;">{{ out.hex }}</code>
              <el-button size="small" @click="copy(out.hex)">复制</el-button>
            </div>
            <div style="margin-bottom:8px;color:#606266;font-size:12px;">base64</div>
            <div style="display:flex;align-items:center;gap:8px;">
              <code style="flex:1;padding:6px 10px;background:#f5f7fa;border-radius:4px;word-break:break-all;font-family:monospace;">{{ out.base64 }}</code>
              <el-button size="small" @click="copy(out.base64)">复制</el-button>
            </div>
          </div>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
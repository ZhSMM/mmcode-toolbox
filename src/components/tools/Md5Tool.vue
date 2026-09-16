<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/md5';
import * as historyApi from '@/api/history';
import { copyText } from '@/utils/format';

const TOOL_ID = 'md5';
const input = ref('Hello, Mavis!');
const out = ref<{ hex: string; base64: string } | null>(null);
const busy = ref(false);

const run = async () => {
  busy.value = true;
  try {
    out.value = await api.md5Hash({ input: input.value });
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: input.value, output: out.value.hex, status: 'success' });
  } catch (e: any) {
    ElMessage.error('执行失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};
const copy = async (s: string) => { await copyText(s); ElMessage.success('已复制'); };
const loadFromHistory = (v: string) => { input.value = v; run(); };
</script>

<template>
  <ToolPanel tool-id="md5" title="MD5" description="128-bit 散列。注意:MD5 已不安全,仅用于校验/去重。" @pick-history="loadFromHistory">
    <template #options>
      <div class="tool-panel__options">
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="run">计算</el-button>
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
        <div class="tool-panel__pane-header"><span>输出</span></div>
        <div class="tool-panel__pane-body" style="padding:12px;">
          <div v-if="!out" style="color:#909399;">点击「计算」</div>
          <div v-else>
            <div style="margin-bottom:12px;">
              <div style="color:#606266;font-size:12px;margin-bottom:4px;">hex</div>
              <div style="display:flex;align-items:center;gap:8px;">
                <code style="flex:1;padding:6px 10px;background:#f5f7fa;border-radius:4px;word-break:break-all;">{{ out.hex }}</code>
                <el-button size="small" @click="copy(out.hex)">复制</el-button>
              </div>
            </div>
            <div>
              <div style="color:#606266;font-size:12px;margin-bottom:4px;">base64</div>
              <div style="display:flex;align-items:center;gap:8px;">
                <code style="flex:1;padding:6px 10px;background:#f5f7fa;border-radius:4px;word-break:break-all;">{{ out.base64 }}</code>
                <el-button size="small" @click="copy(out.base64)">复制</el-button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
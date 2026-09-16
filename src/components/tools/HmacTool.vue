<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/hmac';
import * as historyApi from '@/api/history';
import { copyText } from '@/utils/format';

const TOOL_ID = 'hmac';
const input = ref('Hello, Mavis!');
const secret = ref('my-secret-key');
const algorithm = ref<'hmac-sha256' | 'hmac-sha512'>('hmac-sha256');
const uppercase = ref(false);
const out = ref<{ hex: string; base64: string } | null>(null);
const busy = ref(false);

const run = async () => {
  busy.value = true;
  try {
    out.value = await api.hmacHash({ input: input.value, secret: secret.value, algorithm: algorithm.value, uppercase: uppercase.value });
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: `${input.value.substring(0, 50)} | secret=${secret.value.length}b`, output: out.value.hex, status: 'success' });
  } catch (e: any) {
    ElMessage.error('执行失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};
const copy = async (s: string) => { await copyText(s); ElMessage.success('已复制'); };
const loadFromHistory = (v: string) => { input.value = v; run(); };
</script>

<template>
  <ToolPanel tool-id="hmac" title="HMAC" description="带密钥的散列(SHA-256 / SHA-512)。" @pick-history="loadFromHistory">
    <template #options>
      <div class="tool-panel__options">
        <el-radio-group v-model="algorithm" size="small">
          <el-radio-button value="hmac-sha256">HMAC-SHA256</el-radio-button>
          <el-radio-button value="hmac-sha512">HMAC-SHA512</el-radio-button>
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
          <el-input v-model="input" type="textarea" :rows="10" resize="none" spellcheck="false" />
        </div>
        <div class="tool-panel__pane-header" style="border-top:1px solid #e4e7ed;">
          <span>密钥</span>
          <el-button size="small" text @click="copy(secret.value)">复制</el-button>
        </div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="secret" type="textarea" :rows="6" resize="none" spellcheck="false" />
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>{{ algorithm }} 输出</span></div>
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
<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/aes';
import * as historyApi from '@/api/history';
import { copyText } from '@/utils/format';

const TOOL_ID = 'aes';
const mode = ref<'encrypt' | 'decrypt'>('encrypt');
const input = ref('Hello, Mavis!');
const keyHex = ref('');
const nonceHex = ref('');
const aadHex = ref('');
const output = ref('');
const busy = ref(false);

const randomKey = async () => {
  try {
    const r = await api.aesRandomKey();
    const parsed = JSON.parse(r.output);
    keyHex.value = parsed.key;
    nonceHex.value = parsed.nonce;
    ElMessage.success('已生成随机 key + nonce');
  } catch (e: any) {
    ElMessage.error('生成失败: ' + (e?.message ?? e));
  }
};

const run = async () => {
  if (!input.value || !keyHex.value) { ElMessage.warning('请输入内容和 key'); return; }
  busy.value = true;
  try {
    const r = mode.value === 'encrypt'
      ? await api.aesEncrypt({ input: input.value, key_hex: keyHex.value, nonce_hex: nonceHex.value || undefined, aad_hex: aadHex.value || undefined })
      : await api.aesDecrypt({ input: input.value, key_hex: keyHex.value, nonce_hex: nonceHex.value || undefined, aad_hex: aadHex.value || undefined });
    output.value = r.output;
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: input.value.substring(0, 100), output: mode.value === 'encrypt' ? output.value.substring(0, 100) : '(解密结果已隐藏)', status: 'success' });
  } catch (e: any) {
    const msg = e?.message ?? String(e);
    ElMessage.error(`${mode.value === 'encrypt' ? '加密' : '解密'}失败: ${msg}`);
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: input.value.substring(0, 100), output: null, status: 'error', error_msg: msg }).catch(() => undefined);
  } finally { busy.value = false; }
};

const copy = async (s: string) => {
  if (!s) { ElMessage.warning('暂无内容'); return; }
  await copyText(s);
  ElMessage.success('已复制');
};
</script>

<template>
  <ToolPanel tool-id="aes" title="AES-256-GCM" description="认证加密(带关联数据)。输出格式:nonce(12B)||ciphertext||tag。">
    <template #options>
      <div class="tool-panel__options">
        <el-radio-group v-model="mode" size="small">
          <el-radio-button value="encrypt">加密</el-radio-button>
          <el-radio-button value="decrypt">解密</el-radio-button>
        </el-radio-group>
        <el-button :icon="'MagicStick'" size="small" @click="randomKey">生成随机 key + nonce</el-button>
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="run">执行</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>{{ mode === 'encrypt' ? '明文' : '密文 (hex)' }}</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="input" type="textarea" :rows="10" resize="none" spellcheck="false" />
        </div>
        <div class="tool-panel__pane-header" style="border-top:1px solid #e4e7ed;">
          <span>key (32 字节 hex = 64 字符) <el-button size="small" text @click="copy(keyHex)">复制</el-button></span>
        </div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="keyHex" type="textarea" :rows="3" resize="none" spellcheck="false" />
        </div>
        <div class="tool-panel__pane-header" style="border-top:1px solid #e4e7ed;">
          <span>nonce (12 字节 hex = 24 字符,可空)</span>
        </div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="nonceHex" type="textarea" :rows="2" resize="none" spellcheck="false" />
        </div>
        <div class="tool-panel__pane-header" style="border-top:1px solid #e4e7ed;">
          <span>AAD (附加认证数据 hex,可空)</span>
        </div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="aadHex" type="textarea" :rows="2" resize="none" spellcheck="false" />
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span>输出</span>
          <el-button size="small" text @click="copy(output)">复制</el-button>
        </div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="output" type="textarea" :rows="22" readonly spellcheck="false" />
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
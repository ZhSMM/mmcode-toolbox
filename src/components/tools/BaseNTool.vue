<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/base-n';
import * as historyApi from '@/api/history';
import { copyText } from '@/utils/format';

const TOOL_ID = 'base-n';
const mode = ref<'b32-enc' | 'b32-dec' | 'b58-enc' | 'b58-dec'>('b32-enc');
const variant = ref<'base32' | 'base32hex'>('base32');
const noPad = ref(false);
const input = ref('Hello');
const output = ref('');
const busy = ref(false);

const run = async () => {
  if (!input.value) { ElMessage.warning('请输入内容'); return; }
  busy.value = true;
  try {
    let r;
    if (mode.value === 'b32-enc') r = await api.base32Encode({ input: input.value, variant: variant.value, no_pad: noPad.value });
    else if (mode.value === 'b32-dec') r = await api.base32Decode({ input: input.value, variant: variant.value, no_pad: noPad.value });
    else if (mode.value === 'b58-enc') r = await api.base58Encode({ input: input.value });
    else r = await api.base58Decode({ input: input.value });
    output.value = (r as any).encoded;
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: input.value, output: output.value, status: 'success' });
  } catch (e: any) {
    ElMessage.error('执行失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};

const copy = async () => {
  if (!output.value) return;
  await copyText(output.value);
  ElMessage.success('已复制');
};
</script>

<template>
  <ToolPanel tool-id="base-n" title="Base32 / Base58" description="RFC 4648 Base32 / Base32hex,Bitcoin Base58。">
    <template #options>
      <div class="tool-panel__options">
        <el-radio-group v-model="mode" size="small">
          <el-radio-button value="b32-enc">Base32 编</el-radio-button>
          <el-radio-button value="b32-dec">Base32 解</el-radio-button>
          <el-radio-button value="b58-enc">Base58 编</el-radio-button>
          <el-radio-button value="b58-dec">Base58 解</el-radio-button>
        </el-radio-group>
        <template v-if="mode.startsWith('b32')">
          <el-radio-group v-model="variant" size="small">
            <el-radio-button value="base32">base32</el-radio-button>
            <el-radio-button value="base32hex">base32hex</el-radio-button>
          </el-radio-group>
          <el-checkbox v-model="noPad">无 padding</el-checkbox>
        </template>
        <div style="flex:1" />
        <el-button :icon="'CopyDocument'" :disabled="!output" @click="copy">复制</el-button>
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
        <div class="tool-panel__pane-header"><span>输出</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="output" type="textarea" :rows="22" readonly spellcheck="false" />
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
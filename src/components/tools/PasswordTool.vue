<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/password';
import * as historyApi from '@/api/history';
import { copyText } from '@/utils/format';

const TOOL_ID = 'password';
const length = ref(16);
const count = ref(5);
const upper = ref(true);
const lower = ref(true);
const digits = ref(true);
const symbols = ref(false);
const noAmbig = ref(false);
const mustEach = ref(true);
const result = ref<{ passwords: string[]; strength: string; entropy_bits: number } | null>(null);
const busy = ref(false);

const gen = async () => {
  busy.value = true;
  try {
    result.value = await api.passwordGenerate({
      length: length.value, count: count.value,
      uppercase: upper.value, lowercase: lower.value, digits: digits.value, symbols: symbols.value,
      no_ambiguous: noAmbig.value, must_include_each: mustEach.value,
    });
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: `len=${length.value} count=${count.value}`, output: result.value.passwords.join('\n'), status: 'success' });
  } catch (e: any) {
    ElMessage.error('生成失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};

const copyAll = async () => {
  if (!result.value) return;
  await copyText(result.value.passwords.join('\n'));
  ElMessage.success('已复制全部');
};
const copyOne = async (s: string) => { await copyText(s); ElMessage.success('已复制'); };
</script>

<template>
  <ToolPanel tool-id="password" title="随机密码生成" description="可控字符集 / 长度 / 数量,显示熵与强度。">
    <template #options>
      <div class="tool-panel__options">
        <span>长度</span>
        <el-input-number v-model="length" :min="4" :max="128" size="small" style="width:120px;" />
        <span>数量</span>
        <el-input-number v-model="count" :min="1" :max="50" size="small" style="width:120px;" />
        <el-checkbox v-model="upper">大写</el-checkbox>
        <el-checkbox v-model="lower">小写</el-checkbox>
        <el-checkbox v-model="digits">数字</el-checkbox>
        <el-checkbox v-model="symbols">符号</el-checkbox>
        <el-checkbox v-model="noAmbig">去歧义 (0OolI1)</el-checkbox>
        <el-checkbox v-model="mustEach">每类至少 1</el-checkbox>
        <div style="flex:1" />
        <el-button :icon="'CopyDocument'" :disabled="!result?.passwords.length" @click="copyAll">复制全部</el-button>
        <el-button type="primary" :loading="busy" @click="gen">生成</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane" style="grid-column:span 2;">
        <div class="tool-panel__pane-header">
          <span>
            密码
            <span v-if="result" style="margin-left:8px;color:#909399;font-size:12px;">
              强度 {{ result.strength }} · 熵 {{ result.entropy_bits }} bits
            </span>
          </span>
        </div>
        <div class="tool-panel__pane-body" style="padding:8px 12px;overflow:auto;">
          <div v-if="!result" style="color:#909399;padding:8px;">点击「生成」</div>
          <div v-for="(s, i) in result?.passwords ?? []" :key="i" style="display:flex;align-items:center;gap:8px;padding:4px 0;border-bottom:1px solid #f0f0f0;">
            <code style="flex:1;font-family:monospace;font-size:14px;">{{ s }}</code>
            <el-button size="small" text @click="copyOne(s)">复制</el-button>
          </div>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
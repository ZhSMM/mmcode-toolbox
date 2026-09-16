<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/string-stats';

const TOOL_ID = 'string-stats';
const input = ref('Mavis Code Toolbox 是给开发者用的桌面工具箱,内置 25 个高频工具。统计包括字符数、词频、字节分布等。');
const topN = ref(10);
const result = ref<any>(null);
const busy = ref(false);

const run = async () => {
  busy.value = true;
  try {
    result.value = await api.stringStats({ input: input.value, top_n: topN.value });
  } catch (e: any) {
    ElMessage.error('执行失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};
</script>

<template>
  <ToolPanel tool-id="string-stats" title="字符串统计" description="字符 / 字节 / 行 / 词 / 词频 / 字节分布。">
    <template #options>
      <div class="tool-panel__options">
        <span style="color:#606266;font-size:13px;">词频 top</span>
        <el-input-number v-model="topN" :min="1" :max="100" size="small" style="width:120px;" />
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="run">统计</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>输入文本 ({{ input.length }} 字符)</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="input" type="textarea" :rows="20" resize="none" spellcheck="false" />
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span v-if="result">统计结果</span>
          <span v-else>结果</span>
        </div>
        <div class="tool-panel__pane-body" style="padding:12px;overflow:auto;">
          <div v-if="!result" style="color:#909399;">点击「统计」生成报告</div>
          <div v-else>
            <el-descriptions :column="3" border size="small">
              <el-descriptions-item label="字符数">{{ result.chars }}</el-descriptions-item>
              <el-descriptions-item label="非空白字符">{{ result.chars_no_whitespace }}</el-descriptions-item>
              <el-descriptions-item label="字节">{{ result.bytes }}</el-descriptions-item>
              <el-descriptions-item label="行数">{{ result.lines }}</el-descriptions-item>
              <el-descriptions-item label="词数">{{ result.words }}</el-descriptions-item>
              <el-descriptions-item label="句子">{{ result.sentences }}</el-descriptions-item>
              <el-descriptions-item label="中文字符">{{ result.chinese_chars }}</el-descriptions-item>
              <el-descriptions-item label="ASCII 字符">{{ result.ascii_chars }}</el-descriptions-item>
              <el-descriptions-item label="数字">{{ result.digits }}</el-descriptions-item>
            </el-descriptions>
            <h4 style="margin:16px 0 8px;font-size:13px;">词频 Top {{ result.top_words.length }}</h4>
            <el-table :data="result.top_words" size="small" style="width:100%;">
              <el-table-column label="词 / 字" prop="0" />
              <el-table-column label="次数" prop="1" width="100" />
            </el-table>
          </div>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
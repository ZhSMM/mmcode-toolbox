<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/diff';

const TOOL_ID = 'diff';
const left = ref('apple\nbanana\ncherry\ndate');
const right = ref('apple\nblueberry\ncherry\nelderberry');
const result = ref<any>(null);
const busy = ref(false);

const run = async () => {
  busy.value = true;
  try {
    result.value = await api.diffText({ left: left.value, right: right.value });
  } catch (e: any) {
    ElMessage.error('执行失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};
</script>

<template>
  <ToolPanel tool-id="diff" title="Diff 对比" description="基于 similar crate 的行级 diff。">
    <template #options>
      <div class="tool-panel__options">
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="run">对比</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>原文 A</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="left" type="textarea" :rows="20" resize="none" spellcheck="false" />
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span v-if="result">差异 (+{{ result.stats.added }} -{{ result.stats.deleted }} ={{ result.stats.unchanged }})</span>
          <span v-else>原文 B</span>
        </div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="right" type="textarea" :rows="6" resize="none" spellcheck="false" />
        </div>
        <div class="tool-panel__pane-body mono-input" style="border-top:1px solid #e4e7ed;">
          <div v-if="!result" style="color:#909399;padding:8px;">点击「对比」查看差异</div>
          <div v-else style="overflow:auto;height:100%;">
            <div v-for="(h, i) in result.hunks" :key="i"
                 :style="{
                   background: h.tag === 'insert' ? '#e7f7e7' : h.tag === 'delete' ? '#fde8e8' : '#fafafa',
                   padding: '2px 8px',
                   fontFamily: 'monospace',
                   fontSize: '12px'
                 }">
              <span style="display:inline-block;width:24px;color:#999;">
                {{ h.left_line ?? '' }}
              </span>
              <span style="display:inline-block;width:24px;color:#999;">
                {{ h.right_line ?? '' }}
              </span>
              <span style="color:#999;margin-right:6px;">{{ h.tag === 'insert' ? '+' : h.tag === 'delete' ? '-' : ' ' }}</span>
              {{ h.value }}
            </div>
          </div>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
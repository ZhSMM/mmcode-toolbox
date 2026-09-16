<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/color';
import { copyText } from '@/utils/format';

const TOOL_ID = 'color';
const input = ref('#409eff');
const from = ref<'hex' | 'rgb' | 'hsl'>('hex');
const result = ref<any>(null);
const busy = ref(false);

const convert = async () => {
  if (!input.value) return;
  busy.value = true;
  try {
    result.value = await api.colorConvert({ input: input.value, from: from.value });
  } catch (e: any) {
    ElMessage.error('解析失败: ' + (e?.message ?? e));
    result.value = null;
  } finally { busy.value = false; }
};

const copy = async (s: string) => { await copyText(s); ElMessage.success('已复制'); };

const previewStyle = computed(() => ({
  background: result.value?.hex ?? '#f0f0f0',
  width: '100%',
  height: '120px',
  borderRadius: '6px',
  border: '1px solid #e4e7ed',
}));
import { computed } from 'vue';
</script>

<template>
  <ToolPanel tool-id="color" title="颜色转换" description="HEX ↔ RGB ↔ HSL,显示 WCAG 对比度与亮度。">
    <template #options>
      <div class="tool-panel__options">
        <el-radio-group v-model="from" size="small">
          <el-radio-button value="hex">HEX</el-radio-button>
          <el-radio-button value="rgb">RGB</el-radio-button>
          <el-radio-button value="hsl">HSL</el-radio-button>
        </el-radio-group>
        <el-input v-model="input" placeholder="输入颜色" style="flex:1;" />
        <el-button type="primary" :loading="busy" @click="convert">转换</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>预览</span></div>
        <div class="tool-panel__pane-body" style="padding:20px;">
          <div :style="previewStyle" />
          <div v-if="result" style="margin-top:16px;">
            <el-descriptions :column="2" border size="small">
              <el-descriptions-item label="HEX">{{ result.hex }} <el-button size="small" text @click="copy(result.hex)">复制</el-button></el-descriptions-item>
              <el-descriptions-item label="HEX+alpha">{{ result.hex_long }} <el-button size="small" text @click="copy(result.hex_long)">复制</el-button></el-descriptions-item>
              <el-descriptions-item label="RGB">rgb({{ result.rgb[0] }}, {{ result.rgb[1] }}, {{ result.rgb[2] }}) <el-button size="small" text @click="copy(result.rgba)">复制</el-button></el-descriptions-item>
              <el-descriptions-item label="HSL">hsl({{ result.hsl[0] }}, {{ result.hsl[1] }}%, {{ result.hsl[2] }}%) <el-button size="small" text @click="copy(result.hsla)">复制</el-button></el-descriptions-item>
              <el-descriptions-item label="亮度 (WCAG)">{{ result.luminance }}</el-descriptions-item>
              <el-descriptions-item label="对比度 vs 白/黑">{{ result.contrast_white }} / {{ result.contrast_black }}</el-descriptions-item>
            </el-descriptions>
          </div>
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>示例输入</span></div>
        <div class="tool-panel__pane-body" style="padding:12px;">
          <el-button v-for="(c, i) in ['#409eff', '#67c23a', '#f56c6c', '#e6a23c', '#909399', '#000000', '#ffffff', '#ff5722', '#9c27b0']" :key="i"
            size="small" plain style="margin:4px;" @click="() => { from = 'hex'; input = c; convert(); }">
            {{ c }}
          </el-button>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
<script setup lang="ts">
/**
 * 占位工具页:尚未实现的工具都用这个。
 * 设计文档 §13 列出了当前所有占位项。
 */
import { computed } from 'vue';
import { useAppStore } from '@/stores/app';

const props = defineProps<{
  toolId: string;
  title: string;
}>();

const appStore = useAppStore();

const tool = computed(() =>
  appStore.tools.find((t) => t.tool_id === props.toolId)
);
</script>

<template>
  <div class="tool-panel">
    <header class="tool-panel__head">
      <div>
        <h2>{{ title }}</h2>
        <p class="desc">该工具尚未实现,作为脚手架占位。</p>
      </div>
    </header>

    <div
      style="
        flex:1;display:flex;flex-direction:column;align-items:center;
        justify-content:center;color:var(--text-secondary);gap:8px;
      "
    >
      <el-icon :size="56" color="#c0c4cc"><Tools /></el-icon>
      <h3 style="margin:8px 0 0;font-weight:500;">{{ title }}</h3>
      <p style="margin:0;font-size:13px;">tool_id: <code>{{ toolId }}</code></p>
      <p v-if="tool" style="margin:0;font-size:13px;">
        分类: {{ tool.category }} · 排序: {{ tool.sort_order }}
      </p>
      <p style="margin:16px 0 0;font-size:12px;max-width:480px;text-align:center;">
        详见 DESIGN.md §12,按照「写一个 Rust 纯函数 → 注册 command → 加进 TOOLS 常量 → 写 Vue 组件复用 ToolPanel」的流程即可补全。
      </p>
    </div>
  </div>
</template>
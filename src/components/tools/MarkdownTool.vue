<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/markdown';

const TOOL_ID = 'markdown';
const input = ref(`# Hello Mavis

这是一个 **Markdown** 预览工具。支持:

- 表格
- ~~删除线~~
- 任务列表
- 脚注[^1]

\`\`\`rust
fn main() {
    println!("Hello, world!");
}
\`\`\`

| 列1 | 列2 |
|---|---|
| a | b |

[^1]: 脚注示例。

- [x] 已完成
- [ ] 待办
`);
const result = ref<{ html: string; line_count: number; byte_size: number } | null>(null);
const busy = ref(false);

const render = async () => {
  busy.value = true;
  try {
    result.value = await api.markdownRender({ input: input.value });
  } catch (e: any) {
    ElMessage.error('渲染失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};
render();
</script>

<template>
  <ToolPanel tool-id="markdown" title="Markdown 预览" description="pulldown-cmark 渲染,支持 GFM(表格、删除线、任务列表、脚注)。">
    <template #options>
      <div class="tool-panel__options">
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="render">渲染</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span>Markdown 源</span>
          <span v-if="result" style="font-size:11px;color:#909399;">{{ result.line_count }} 行 / {{ result.byte_size }} 字节</span>
        </div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="input" type="textarea" :rows="22" resize="none" spellcheck="false" />
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>预览</span></div>
        <div class="tool-panel__pane-body" style="padding:24px;overflow:auto;">
          <div v-if="result" class="markdown-body" v-html="result.html" />
        </div>
      </div>
    </template>
  </ToolPanel>
</template>

<style>
.markdown-body { font-size: 14px; line-height: 1.6; color: #303133; }
.markdown-body h1, .markdown-body h2, .markdown-body h3 { margin: 16px 0 8px; font-weight: 600; }
.markdown-body h1 { font-size: 24px; border-bottom: 1px solid #e4e7ed; padding-bottom: 6px; }
.markdown-body h2 { font-size: 20px; }
.markdown-body h3 { font-size: 16px; }
.markdown-body p { margin: 8px 0; }
.markdown-body code { background: #f5f7fa; padding: 2px 6px; border-radius: 3px; font-family: monospace; font-size: 12px; }
.markdown-body pre { background: #f5f7fa; padding: 12px; border-radius: 4px; overflow-x: auto; }
.markdown-body pre code { background: transparent; padding: 0; }
.markdown-body table { border-collapse: collapse; margin: 12px 0; }
.markdown-body table th, .markdown-body table td { border: 1px solid #e4e7ed; padding: 6px 12px; }
.markdown-body table th { background: #f5f7fa; }
.markdown-body blockquote { border-left: 4px solid #dcdfe6; padding: 4px 12px; color: #606266; margin: 8px 0; }
.markdown-body ul, .markdown-body ol { padding-left: 24px; margin: 8px 0; }
.markdown-body li { margin: 4px 0; }
.markdown-body del { color: #909399; }
</style>
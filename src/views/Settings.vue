<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';

import * as historyApi from '@/api/history';
import { useAppStore } from '@/stores/app';

const appStore = useAppStore();

const version = ref('0.1.0');
const dataDir = ref('应用数据目录');

const clearAllHistory = async () => {
  try {
    await ElMessageBox.confirm('将清空全部工具的历史记录,且不可恢复,确定？', '清空全部历史', {
      type: 'warning',
      confirmButtonText: '确认清空',
      cancelButtonText: '取消',
    });
    await historyApi.clearHistory();
    ElMessage.success('已清空全部历史');
  } catch (e: any) {
    if (e === 'cancel') return;
    ElMessage.error('清空失败: ' + (e?.message ?? e));
  }
};

const reloadMenu = async () => {
  try {
    await appStore.loadMenu();
    await appStore.loadFavorites();
    ElMessage.success('菜单已刷新');
  } catch (e: any) {
    ElMessage.error('刷新失败: ' + (e?.message ?? e));
  }
};
</script>

<template>
  <div class="page" style="max-width:720px;">
    <h2 style="margin-top:0;">设置</h2>

    <el-card style="margin-bottom:16px;">
      <template #header><span>关于</span></template>
      <el-descriptions :column="1" border>
        <el-descriptions-item label="应用名称">Mavis Code Toolbox</el-descriptions-item>
        <el-descriptions-item label="版本">{{ version }}</el-descriptions-item>
        <el-descriptions-item label="技术栈">
          Tauri 2 · Vue 3 · TypeScript · SQLite · Element Plus
        </el-descriptions-item>
        <el-descriptions-item label="数据存储">{{ dataDir }} (由 Tauri Path API 解析)</el-descriptions-item>
        <el-descriptions-item label="已注册工具">
          {{ appStore.tools.length }} 项 · 收藏 {{ appStore.favorites.size }} 项
        </el-descriptions-item>
      </el-descriptions>
    </el-card>

    <el-card style="margin-bottom:16px;">
      <template #header><span>数据管理</span></template>
      <div style="display:flex;flex-direction:column;gap:12px;">
        <el-button type="danger" plain @click="clearAllHistory">
          清空全部历史记录
        </el-button>
        <el-button @click="reloadMenu">重新加载菜单</el-button>
      </div>
    </el-card>

    <el-card>
      <template #header><span>扩展说明</span></template>
      <p style="margin:0 0 8px 0;font-size:13px;color:var(--text-secondary);">
        新增一个工具的标准流程(详见 DESIGN.md §12)：
      </p>
      <ol style="margin:0;padding-left:20px;line-height:1.8;font-size:13px;">
        <li>Rust 端:在 <code>src-tauri/src/commands/</code> 新建 <code>&lt;tool&gt;.rs</code> 实现纯函数 + <code>#[tauri::command]</code>。</li>
        <li>在 <code>commands/mod.rs</code> 导出,在 <code>lib.rs</code> 的 <code>generate_handler!</code> 数组登记。</li>
        <li>在 <code>commands/tools.rs</code> 的 <code>TOOLS</code> 常量数组加一条。</li>
        <li>前端:在 <code>src/api/</code> 新建 <code>&lt;tool&gt;.ts</code> 封装 invoke。</li>
        <li>在 <code>src/router/index.ts</code> 加一条路由。</li>
        <li>在 <code>src/components/tools/&lt;Tool&gt;.vue</code> 复用 <code>ToolPanel.vue</code>。</li>
      </ol>
    </el-card>
  </div>
</template>
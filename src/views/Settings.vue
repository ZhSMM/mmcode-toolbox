<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';

import * as historyApi from '@/api/history';
import * as favApi from '@/api/favorites';
import { useAppStore } from '@/stores/app';
import { copyText } from '@/utils/format';

const appStore = useAppStore();

const version = ref('0.2.0');
const dataDir = ref('应用数据目录');

const exportFavorites = async () => {
  const data = {
    type: 'mmcode-favorites',
    version: 1,
    exported_at: new Date().toISOString(),
    tool_ids: [...appStore.favorites],
  };
  const json = JSON.stringify(data, null, 2);
  await copyText(json);
  ElMessage.success(`收藏列表已复制 (${appStore.favorites.size} 项)`);
};

const importFavorites = async () => {
  try {
    const raw = await ElMessageBox.prompt(
      '粘贴之前导出的收藏 JSON(格式: {"type":"mmcode-favorites","tool_ids":[...]})',
      '导入收藏',
      {
        inputType: 'textarea',
        inputPlaceholder: '{"type":"mmcode-favorites", "tool_ids": [...]}',
        inputValue: '',
        confirmButtonText: '导入',
        cancelButtonText: '取消',
        customStyle: { maxWidth: '520px' },
      }
    );
    const data = JSON.parse(raw.value);
    if (data.type !== 'mmcode-favorites' || !Array.isArray(data.tool_ids)) {
      ElMessage.error('JSON 格式不合法(缺少 type 或 tool_ids)');
      return;
    }
    const validIds = new Set(appStore.tools.map((t) => t.tool_id));
    const toImport = (data.tool_ids as string[]).filter((id) => validIds.has(id));
    const skipped = data.tool_ids.length - toImport.length;
    let added = 0;
    for (const id of toImport) {
      if (!appStore.isFavorite(id)) {
        await favApi.toggleFavorite(id);
        appStore.favorites.add(id);
        added++;
      }
    }
    appStore.favorites = new Set(appStore.favorites);
    ElMessage.success(`导入完成: 新增 ${added} 项${skipped ? `, 跳过 ${skipped} 项无效` : ''}`);
  } catch (e: any) {
    // Element Plus reject 字符串 'cancel' / 'close'
    if (e === 'cancel' || e === 'close') return;
    if (e instanceof SyntaxError) { ElMessage.error('JSON 解析失败'); return; }
    ElMessage.error('导入失败: ' + (e?.message ?? String(e)));
  }
};

const clearAllHistory = async () => {
  try {
    await ElMessageBox.confirm(
      '将清空全部工具的历史记录,且不可恢复。',
      '清空全部历史',
      {
        type: 'warning',
        confirmButtonText: '确认清空',
        cancelButtonText: '取消',
        confirmButtonClass: 'el-button--danger',
        customStyle: { maxWidth: '380px' },
      }
    );
    await historyApi.clearHistory();
    ElMessage.success('已清空全部历史');
  } catch (e: any) {
    // Element Plus 在点 X / Esc / 取消时 reject 字符串 'cancel' / 'close'
    if (e === 'cancel' || e === 'close') return;
    ElMessage.error('清空失败: ' + (e?.message ?? String(e)));
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
        <el-descriptions-item label="技术栈">Tauri 2 · Vue 3 · TypeScript · SQLite · Element Plus</el-descriptions-item>
        <el-descriptions-item label="数据存储">{{ dataDir }} (由 Tauri Path API 解析)</el-descriptions-item>
        <el-descriptions-item label="已注册工具">{{ appStore.tools.length }} 项 · 收藏 {{ appStore.favorites.size }} 项</el-descriptions-item>
        <el-descriptions-item label="主题切换">顶栏按钮 / Cmd+K 输入「切换主题」</el-descriptions-item>
        <el-descriptions-item label="快捷键">Cmd/Ctrl + K 打开命令面板</el-descriptions-item>
      </el-descriptions>
    </el-card>

    <el-card style="margin-bottom:16px;">
      <template #header><span>收藏管理</span></template>
      <div style="display:flex;flex-direction:column;gap:12px;">
        <div>
          <p style="margin:0 0 8px 0;color:#606266;font-size:13px;">
            把收藏列表导出为 JSON(已复制到剪贴板),粘贴到任意地方保存;在新设备上粘贴回此处导入。
          </p>
          <el-button @click="exportFavorites">导出收藏 (复制 JSON)</el-button>
        </div>
        <el-divider style="margin:4px 0;" />
        <div>
          <p style="margin:0 0 8px 0;color:#606266;font-size:13px;">
            粘贴之前导出的 JSON,只导入当前存在的项(按 tool_id 校验)。
          </p>
          <el-button type="primary" plain @click="importFavorites">导入收藏</el-button>
        </div>
      </div>
    </el-card>

    <el-card style="margin-bottom:16px;">
      <template #header><span>数据管理</span></template>
      <div style="display:flex;flex-direction:column;gap:12px;">
        <el-button type="danger" plain @click="clearAllHistory">清空全部历史记录</el-button>
        <el-button @click="reloadMenu">重新加载菜单 / 收藏</el-button>
      </div>
    </el-card>

    <el-card>
      <template #header><span>扩展指引</span></template>
      <p style="margin:0 0 8px 0;font-size:13px;color:var(--text-secondary);">
        新增一个工具的标准流程(详见 DESIGN.md §12):
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
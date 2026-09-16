<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/random';
import * as historyApi from '@/api/history';
import { copyText } from '@/utils/format';

const TOOL_ID = 'random';
const mode = ref<'int' | 'float' | 'pick' | 'dice' | 'bytes' | 'shuffle'>('int');
const count = ref(5);
const min = ref<number>(1);
const max = ref<number>(100);
const items = ref('apple, banana, cherry, date, elderberry, fig, grape');
const seed = ref<number | null>(null);
const result = ref<{ results: string[]; description: string } | null>(null);
const busy = ref(false);

const gen = async () => {
  busy.value = true;
  try {
    const req: any = { mode: mode.value, count: count.value };
    if (mode.value === 'int' || mode.value === 'float') { req.min = min.value; req.max = max.value; }
    if (mode.value === 'pick' || mode.value === 'shuffle') { req.items = items.value; }
    if (mode.value === 'dice') { req.max = max.value; }
    if (seed.value !== null) { req.seed = seed.value; }
    result.value = await api.randomGenerate(req);
    await historyApi.saveHistory({ tool_id: TOOL_ID, input: JSON.stringify(req), output: result.value.description + ': ' + result.value.results.join(','), status: 'success' });
  } catch (e: any) {
    ElMessage.error('生成失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};

const copy = async () => {
  if (!result.value) return;
  await copyText(result.value.results.join('\n'));
  ElMessage.success('已复制');
};
</script>

<template>
  <ToolPanel tool-id="random" title="随机数 / 抽样" description="整数、浮点、骰子、字节、抽样、洗牌。支持固定种子。">
    <template #options>
      <div class="tool-panel__options">
        <el-radio-group v-model="mode" size="small">
          <el-radio-button value="int">整数</el-radio-button>
          <el-radio-button value="float">浮点</el-radio-button>
          <el-radio-button value="dice">骰子</el-radio-button>
          <el-radio-button value="bytes">字节</el-radio-button>
          <el-radio-button value="pick">抽样</el-radio-button>
          <el-radio-button value="shuffle">洗牌</el-radio-button>
        </el-radio-group>
        <template v-if="mode === 'int' || mode === 'float' || mode === 'dice'">
          <span>min</span>
          <el-input-number v-model="min" :min="0" size="small" style="width:100px;" />
          <span>{{ mode === 'dice' ? '面数' : 'max' }}</span>
          <el-input-number v-model="max" :min="1" size="small" style="width:100px;" />
        </template>
        <template v-if="mode === 'pick' || mode === 'shuffle'">
          <el-input v-model="items" placeholder="用逗号或换行分隔" style="width:240px;" size="small" />
        </template>
        <span>数量</span>
        <el-input-number v-model="count" :min="1" :max="100" size="small" style="width:100px;" />
        <span>种子</span>
        <el-input-number v-model="seed" :min="0" :max="999999" size="small" style="width:120px;" placeholder="随机" />
        <div style="flex:1" />
        <el-button :icon="'CopyDocument'" :disabled="!result" @click="copy">复制</el-button>
        <el-button type="primary" :loading="busy" @click="gen">生成</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane" style="grid-column:span 2;">
        <div class="tool-panel__pane-header">
          <span v-if="result">{{ result.description }} ({{ result.results.length }} 个)</span>
          <span v-else>结果</span>
        </div>
        <div class="tool-panel__pane-body" style="padding:8px 12px;">
          <div v-if="!result" style="color:#909399;padding:8px;">点击「生成」</div>
          <div v-for="(r, i) in result?.results ?? []" :key="i" style="padding:4px 0;font-family:monospace;font-size:13px;border-bottom:1px solid #f0f0f0;">
            <span style="display:inline-block;width:40px;color:#999;">#{{ i + 1 }}</span> {{ r }}
          </div>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
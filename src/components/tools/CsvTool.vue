<script setup lang="ts">
import { ref, computed } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/csv-viewer';

const TOOL_ID = 'csv-viewer';
const input = ref(`name,age,city,active
Alice,30,Beijing,true
Bob,25,Shanghai,true
Carol,28,Guangzhou,false
Dave,35,Shenzhen,true
Alice,30,Beijing,true`);
const delimiter = ref<string>('');
const hasHeaders = ref(true);
const result = ref<any>(null);
const busy = ref(false);

const parse = async () => {
  if (!input.value) { ElMessage.warning('请输入 CSV'); return; }
  busy.value = true;
  try {
    result.value = await api.csvParse({
      input: input.value,
      delimiter: delimiter.value || undefined,
      has_headers: hasHeaders.value,
    });
  } catch (e: any) {
    ElMessage.error('解析失败: ' + (e?.message ?? e));
  } finally { busy.value = false; }
};

const displayRows = computed(() => result.value?.rows.slice(0, 100) ?? []);
parse();
</script>

<template>
  <ToolPanel tool-id="csv-viewer" title="CSV 工具" description="解析、统计、表格预览。自动检测逗号/TAB/分号。">
    <template #options>
      <div class="tool-panel__options">
        <span style="color:#606266;font-size:13px;">分隔符</span>
        <el-input v-model="delimiter" placeholder="自动" style="width:100px;" />
        <el-checkbox v-model="hasHeaders">有表头</el-checkbox>
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="parse">解析</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>CSV 输入 ({{ input.length }} 字符)</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="input" type="textarea" :rows="12" resize="none" spellcheck="false" />
        </div>
        <div v-if="result" class="tool-panel__pane-header" style="border-top:1px solid #e4e7ed;">
          <span>列统计</span>
        </div>
        <div v-if="result" class="tool-panel__pane-body" style="padding:8px 12px;">
          <el-table :data="result.stats" size="small" border>
            <el-table-column prop="name" label="列" />
            <el-table-column prop="non_empty" label="非空" width="80" />
            <el-table-column prop="empty" label="空" width="60" />
            <el-table-column prop="unique" label="唯一" width="80" />
            <el-table-column label="示例" min-width="180">
              <template #default="{ row }">
                <span style="font-size:11px;color:#606266;">{{ row.sample_values.join(', ') }}</span>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span v-if="result">表格预览 (前 {{ displayRows.length }} 行,共 {{ result.total_rows }} 行)</span>
          <span v-else>结果</span>
        </div>
        <div class="tool-panel__pane-body" style="overflow:auto;">
          <div v-if="!result" style="color:#909399;padding:8px;">点击「解析」</div>
          <el-table v-else :data="displayRows" size="small" border style="width:max-content;min-width:100%;">
            <el-table-column v-for="(h, i) in result.headers" :key="i" :prop="String(i)" :label="h" min-width="120" />
          </el-table>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
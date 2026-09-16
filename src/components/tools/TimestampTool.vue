<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/timestamp';
import { copyText } from '@/utils/format';

const TOOL_ID = 'timestamp';
const tab = ref<'now' | 'parse' | 'to'>('now');

// now
const nowResult = ref<any>(null);
const fetchNow = async () => {
  nowResult.value = await api.timestampNow();
};

// parse
const inputTs = ref('');
const unit = ref<'s' | 'ms'>('ms');
const parsed = ref<any>(null);
const parseTs = async () => {
  if (!inputTs.value) { ElMessage.warning('请输入时间戳'); return; }
  try { parsed.value = await api.timestampParse({ input: inputTs.value, unit: unit.value }); }
  catch (e: any) { ElMessage.error('解析失败: ' + (e?.message ?? e)); }
};

// to
const inputDate = ref('');
const toResult = ref<any>(null);
const toTs = async () => {
  if (!inputDate.value) { ElMessage.warning('请输入日期'); return; }
  try { toResult.value = await api.timestampTo({ input: inputDate.value }); }
  catch (e: any) { ElMessage.error('解析失败: ' + (e?.message ?? e)); }
};

const copy = async (s: string) => { await copyText(s); ElMessage.success('已复制'); };
fetchNow();
</script>

<template>
  <ToolPanel tool-id="timestamp" title="时间戳转换" description="当前时间 ↔ Unix 时间戳 ↔ 人类可读日期。">
    <template #options>
      <div class="tool-panel__options">
        <el-radio-group v-model="tab" size="small">
          <el-radio-button value="now">当前</el-radio-button>
          <el-radio-button value="parse">时间戳 → 日期</el-radio-button>
          <el-radio-button value="to">日期 → 时间戳</el-radio-button>
        </el-radio-group>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane" v-if="tab === 'now'">
        <div class="tool-panel__pane-header">
          <span>当前时间</span>
          <el-button size="small" text @click="fetchNow">刷新</el-button>
        </div>
        <div class="tool-panel__pane-body" style="padding:12px;">
          <div v-if="nowResult">
            <el-descriptions :column="1" border>
              <el-descriptions-item label="Unix 毫秒">{{ nowResult.ms }} <el-button size="small" text @click="copy(String(nowResult.ms))">复制</el-button></el-descriptions-item>
              <el-descriptions-item label="Unix 秒">{{ nowResult.seconds }} <el-button size="small" text @click="copy(String(nowResult.seconds))">复制</el-button></el-descriptions-item>
              <el-descriptions-item label="ISO (UTC)">{{ nowResult.iso_utc }} <el-button size="small" text @click="copy(nowResult.iso_utc)">复制</el-button></el-descriptions-item>
              <el-descriptions-item label="ISO (本地)">{{ nowResult.iso_local }} <el-button size="small" text @click="copy(nowResult.iso_local)">复制</el-button></el-descriptions-item>
            </el-descriptions>
          </div>
        </div>
      </div>
      <div v-else-if="tab === 'parse'" class="tool-panel__pane" style="grid-column:span 2;">
        <div class="tool-panel__pane-header">
          <span>时间戳 → 日期</span>
        </div>
        <div class="tool-panel__pane-body" style="padding:12px;">
          <div style="display:flex;gap:8px;margin-bottom:12px;">
            <el-radio-group v-model="unit" size="small">
              <el-radio-button value="ms">毫秒</el-radio-button>
              <el-radio-button value="s">秒</el-radio-button>
            </el-radio-group>
            <el-input v-model="inputTs" placeholder="1700000000000" style="flex:1;" />
            <el-button type="primary" @click="parseTs">解析</el-button>
          </div>
          <el-descriptions v-if="parsed" :column="2" border>
            <el-descriptions-item label="毫秒">{{ parsed.ms }}</el-descriptions-item>
            <el-descriptions-item label="秒">{{ parsed.seconds }}</el-descriptions-item>
            <el-descriptions-item label="ISO UTC">{{ parsed.iso_utc }} <el-button size="small" text @click="copy(parsed.iso_utc)">复制</el-button></el-descriptions-item>
            <el-descriptions-item label="ISO 本地">{{ parsed.iso_local }} <el-button size="small" text @click="copy(parsed.iso_local)">复制</el-button></el-descriptions-item>
            <el-descriptions-item label="RFC 2822">{{ parsed.rfc2822 }}</el-descriptions-item>
            <el-descriptions-item label="时区">{{ parsed.timezone_offset }} · {{ parsed.weekday }}</el-descriptions-item>
          </el-descriptions>
        </div>
      </div>
      <div v-else class="tool-panel__pane" style="grid-column:span 2;">
        <div class="tool-panel__pane-header">
          <span>日期 → 时间戳</span>
        </div>
        <div class="tool-panel__pane-body" style="padding:12px;">
          <div style="display:flex;gap:8px;margin-bottom:12px;">
            <el-input v-model="inputDate" placeholder="2024-01-01 12:00:00" style="flex:1;" />
            <el-button type="primary" @click="toTs">解析</el-button>
          </div>
          <div style="font-size:12px;color:#909399;margin-bottom:8px;">支持格式:YYYY-MM-DD HH:MM:SS / YYYY-MM-DDTHH:MM:SS / RFC3339 / YYYY-MM-DD</div>
          <el-descriptions v-if="toResult" :column="3" border>
            <el-descriptions-item label="毫秒">{{ toResult.ms }}</el-descriptions-item>
            <el-descriptions-item label="秒">{{ toResult.seconds }}</el-descriptions-item>
            <el-descriptions-item label="ISO UTC">{{ toResult.iso_utc }}</el-descriptions-item>
          </el-descriptions>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
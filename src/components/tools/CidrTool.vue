<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/cidr';
import { copyText } from '@/utils/format';

const TOOL_ID = 'cidr';
const input = ref('192.168.1.0/24');
const result = ref<any>(null);
const busy = ref(false);

const compute = async () => {
  busy.value = true;
  try {
    result.value = await api.cidrInfo({ input: input.value });
  } catch (e: any) {
    ElMessage.error('解析失败: ' + (e?.message ?? e));
    result.value = null;
  } finally { busy.value = false; }
};
compute();

const copy = async (s: string) => { await copyText(s); ElMessage.success('已复制'); };
</script>

<template>
  <ToolPanel tool-id="cidr" title="CIDR 子网计算" description="解析 CIDR 块,显示网络地址、广播、子网掩码、可分配主机。">
    <template #options>
      <div class="tool-panel__options">
        <el-input v-model="input" placeholder="192.168.1.0/24" style="width:240px;" />
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="compute">计算</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane" style="grid-column:span 2;">
        <div class="tool-panel__pane-header">
          <span>结果</span>
          <span v-if="result" style="font-size:11px;color:#909399;">IPv{{ result.version }} {{ result.is_private ? '· 私有' : '' }}</span>
        </div>
        <div class="tool-panel__pane-body" style="padding:12px;">
          <div v-if="!result" style="color:#909399;">点击「计算」</div>
          <el-descriptions v-else :column="3" border>
            <el-descriptions-item label="网络">
              <code>{{ result.network }}</code>
              <el-button size="small" text @click="copy(result.network)">复制</el-button>
            </el-descriptions-item>
            <el-descriptions-item label="广播" v-if="result.broadcast">
              <code>{{ result.broadcast }}</code>
              <el-button size="small" text @click="copy(result.broadcast)">复制</el-button>
            </el-descriptions-item>
            <el-descriptions-item label="版本">IPv{{ result.version }}</el-descriptions-item>
            <el-descriptions-item label="子网掩码">{{ result.netmask }}</el-descriptions-item>
            <el-descriptions-item label="通配符">{{ result.wildcard }}</el-descriptions-item>
            <el-descriptions-item label="总地址数">{{ result.total_addresses }}</el-descriptions-item>
            <el-descriptions-item label="可用主机数">{{ result.usable_addresses }}</el-descriptions-item>
            <el-descriptions-item label="首主机" v-if="result.first_host">
              <code>{{ result.first_host }}</code>
            </el-descriptions-item>
            <el-descriptions-item label="末主机" v-if="result.last_host">
              <code>{{ result.last_host }}</code>
            </el-descriptions-item>
          </el-descriptions>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
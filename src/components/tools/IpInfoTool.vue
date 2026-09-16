<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/cidr';
import { copyText } from '@/utils/format';

const TOOL_ID = 'ip-info';
const input = ref('8.8.8.8');
const result = ref<any>(null);
const busy = ref(false);

const lookup = async () => {
  busy.value = true;
  try {
    result.value = await api.ipInfo({ ip: input.value });
  } catch (e: any) {
    ElMessage.error('查询失败: ' + (e?.message ?? e));
    result.value = null;
  } finally { busy.value = false; }
};
lookup();

const copy = async (s: string) => { await copyText(s); ElMessage.success('已复制'); };

const tags = (r: any) => {
  const out: string[] = [];
  if (r.is_private) out.push('private');
  if (r.is_loopback) out.push('loopback');
  if (r.is_link_local) out.push('link-local');
  if (r.is_multicast) out.push('multicast');
  if (r.is_unspecified) out.push('unspecified');
  if (r.is_documentation) out.push('documentation');
  return out;
};
</script>

<template>
  <ToolPanel tool-id="ip-info" title="IP 信息" description="解析 IPv4/IPv6 地址,显示类型与反向 DNS 提示。">
    <template #options>
      <div class="tool-panel__options">
        <el-input v-model="input" placeholder="8.8.8.8 或 2001:db8::1" style="width:280px;" />
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="lookup">查询</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane" style="grid-column:span 2;">
        <div class="tool-panel__pane-header">
          <span>结果</span>
          <span v-if="result" style="font-size:11px;color:#909399;">IPv{{ result.version }}</span>
        </div>
        <div class="tool-panel__pane-body" style="padding:12px;">
          <div v-if="!result" style="color:#909399;">点击「查询」</div>
          <div v-else>
            <div style="margin-bottom:12px;">
              <code style="font-size:14px;padding:4px 8px;background:#f5f7fa;border-radius:3px;">{{ result.ip }}</code>
              <el-button size="small" text @click="copy(result.ip)">复制</el-button>
            </div>
            <div style="margin-bottom:12px;">
              <el-tag v-for="t in tags(result)" :key="t" type="info" size="small" style="margin-right:4px;">{{ t }}</el-tag>
            </div>
            <el-descriptions :column="1" border>
              <el-descriptions-item label="反向 DNS 查询">{{ result.reverse_dns_hint }}</el-descriptions-item>
            </el-descriptions>
          </div>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
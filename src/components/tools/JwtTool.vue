<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import ToolPanel from '@/components/common/ToolPanel.vue';
import * as api from '@/api/jwt';

const TOOL_ID = 'jwt';
const token = ref('eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c');
const verify = ref(false);
const secret = ref('');
const algorithm = ref('HS256');
const result = ref<any>(null);
const busy = ref(false);

const decode = async () => {
  if (!token.value) { ElMessage.warning('请输入 JWT'); return; }
  busy.value = true;
  try {
    result.value = await api.jwtDecode({ token: token.value, verify: verify.value, secret: secret.value || undefined, algorithm: verify.value ? algorithm.value : undefined });
  } catch (e: any) {
    ElMessage.error('解析失败: ' + (e?.message ?? e));
    result.value = null;
  } finally { busy.value = false; }
};

const formatJson = (obj: any) => JSON.stringify(obj, null, 2);
</script>

<template>
  <ToolPanel tool-id="jwt" title="JWT 解码" description="解析 header / payload,可选用 HS256/384/512 验签。">
    <template #options>
      <div class="tool-panel__options">
        <el-checkbox v-model="verify">启用验签</el-checkbox>
        <template v-if="verify">
          <el-select v-model="algorithm" size="small" style="width:140px;">
            <el-option value="HS256" label="HS256" />
            <el-option value="HS384" label="HS384" />
            <el-option value="HS512" label="HS512" />
          </el-select>
          <el-input v-model="secret" placeholder="shared secret" style="width:240px;" size="small" />
        </template>
        <div style="flex:1" />
        <el-button type="primary" :loading="busy" @click="decode">解析</el-button>
      </div>
    </template>
    <template #default>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header"><span>JWT Token</span></div>
        <div class="tool-panel__pane-body mono-input">
          <el-input v-model="token" type="textarea" :rows="6" resize="none" spellcheck="false" placeholder="eyJ..." />
        </div>
        <div v-if="result" class="tool-panel__pane-header" style="border-top:1px solid #e4e7ed;">
          <span>Header</span>
        </div>
        <div v-if="result" class="tool-panel__pane-body mono-input">
          <pre style="margin:0;padding:12px;font-size:12px;">{{ formatJson(result.header.decoded) }}</pre>
        </div>
        <div v-if="result" class="tool-panel__pane-header" style="border-top:1px solid #e4e7ed;">
          <span>Payload</span>
        </div>
        <div v-if="result" class="tool-panel__pane-body mono-input">
          <pre style="margin:0;padding:12px;font-size:12px;">{{ formatJson(result.payload.decoded) }}</pre>
        </div>
      </div>
      <div class="tool-panel__pane">
        <div class="tool-panel__pane-header">
          <span v-if="result">
            签名 (hex,{{ result.signature_bytes }} bytes)
            <span v-if="verify" :style="{ marginLeft: '8px', color: result.verified ? '#67c23a' : '#f56c6c' }">
              {{ result.verified ? '✓ 签名匹配' : '✗ 签名不匹配' }}
            </span>
          </span>
          <span v-else>信息</span>
        </div>
        <div class="tool-panel__pane-body mono-input">
          <div v-if="!result" style="color:#909399;padding:8px;">点击「解析」</div>
          <pre v-else style="margin:0;padding:12px;font-size:11px;word-break:break-all;white-space:pre-wrap;">{{ result.signature_hex }}</pre>
        </div>
        <div v-if="result?.error" class="tool-panel__pane-header" style="border-top:1px solid #e4e7ed;">
          <span style="color:#f56c6c;">错误</span>
        </div>
        <div v-if="result?.error" class="tool-panel__pane-body">
          <div style="padding:12px;color:#f56c6c;">{{ result.error }}</div>
        </div>
      </div>
    </template>
  </ToolPanel>
</template>
<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';

import { useAppStore } from '@/stores/app';

const router = useRouter();
const appStore = useAppStore();

const groups = computed(() => appStore.groupedTools());

const favoritesList = computed(() =>
  appStore.tools.filter((t) => appStore.isFavorite(t.tool_id))
);

const go = (route: string) => router.push(route);

const recommended = computed(() => appStore.tools.slice(0, 6));
</script>

<template>
  <div class="page">
    <h2 style="margin-top:0;">欢迎使用 Mavis Code Toolbox</h2>
    <p style="color:var(--text-secondary);">
      从左侧菜单选择工具,或从下方快速入口开始。所有数据保存在本地 SQLite,不上传任何内容。
    </p>

    <section style="margin-top:24px;">
      <h3 style="font-size:15px;margin:0 0 12px 0;">⭐ 我的收藏</h3>
      <el-empty
        v-if="!favoritesList.length"
        description="尚未收藏任何工具,在工具页点击「收藏」即可加入此处。"
      />
      <div v-else style="display:grid;grid-template-columns:repeat(auto-fill,minmax(180px,1fr));gap:12px;">
        <el-card
          v-for="t in favoritesList"
          :key="t.tool_id"
          shadow="hover"
          style="cursor:pointer;"
          @click="go(t.route)"
        >
          <div style="display:flex;align-items:center;gap:8px;">
            <el-icon v-if="t.icon" :size="20" color="var(--el-color-primary)">
              <component :is="t.icon" />
            </el-icon>
            <span style="font-weight:500;">{{ t.name }}</span>
          </div>
        </el-card>
      </div>
    </section>

    <section style="margin-top:32px;">
      <h3 style="font-size:15px;margin:0 0 12px 0;">🚀 推荐工具</h3>
      <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(220px,1fr));gap:12px;">
        <el-card
          v-for="t in recommended"
          :key="t.tool_id"
          shadow="hover"
          style="cursor:pointer;"
          @click="go(t.route)"
        >
          <div style="display:flex;align-items:center;gap:8px;">
            <el-icon v-if="t.icon" :size="22" color="var(--el-color-primary)">
              <component :is="t.icon" />
            </el-icon>
            <span style="font-weight:500;">{{ t.name }}</span>
          </div>
          <div style="margin-top:6px;font-size:12px;color:var(--text-secondary);">
            {{ t.category }} · {{ t.tool_id }}
          </div>
        </el-card>
      </div>
    </section>

    <section style="margin-top:32px;">
      <h3 style="font-size:15px;margin:0 0 12px 0;">📂 全部工具 ({{ appStore.tools.length }})</h3>
      <div v-for="group in groups" :key="group.category.id" style="margin-bottom:20px;">
        <div style="font-size:13px;color:var(--text-secondary);margin-bottom:8px;">
          {{ group.category.name }}
        </div>
        <div style="display:flex;flex-wrap:wrap;gap:8px;">
          <el-tag
            v-for="t in group.tools"
            :key="t.tool_id"
            effect="plain"
            style="cursor:pointer;"
            @click="go(t.route)"
          >
            {{ t.name }}
          </el-tag>
        </div>
      </div>
    </section>
  </div>
</template>
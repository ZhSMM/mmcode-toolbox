<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import SideMenu from './SideMenu.vue';
import CommandPalette from '@/components/common/CommandPalette.vue';
import { useAppStore } from '@/stores/app';

const route = useRoute();
const router = useRouter();
const appStore = useAppStore();

watch(
  () => route.path,
  (path) => {
    const tool = appStore.tools.find((t) => t.route === path);
    appStore.currentToolId = tool?.tool_id ?? null;
  },
  { immediate: true }
);

const headerTitle = computed(() => (route.meta?.title as string) ?? 'Mavis Code Toolbox');

const goHome = () => router.push({ name: 'home' });
const goSettings = () => router.push({ name: 'settings' });

// 命令面板
const commandPaletteRef = ref<InstanceType<typeof CommandPalette> | null>(null);
const openPalette = () => commandPaletteRef.value?.open();

// 主题
const theme = ref<'system' | 'light' | 'dark'>((localStorage.getItem('mmcode-theme') as any) || 'system');
const cycleTheme = () => {
  const order: Array<'system' | 'light' | 'dark'> = ['system', 'light', 'dark'];
  const i = order.indexOf(theme.value);
  theme.value = order[(i + 1) % order.length];
  applyTheme(theme.value);
};
const applyTheme = (mode: 'system' | 'light' | 'dark') => {
  localStorage.setItem('mmcode-theme', mode);
  const el = document.documentElement;
  el.classList.remove('theme-light', 'theme-dark');
  if (mode === 'light') el.classList.add('theme-light');
  else if (mode === 'dark') el.classList.add('theme-dark');
  else {
    const dark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    el.classList.add(dark ? 'theme-dark' : 'theme-light');
  }
};
const themeIcon = computed(() => theme.value === 'dark' ? 'Moon' : theme.value === 'light' ? 'Sunny' : 'Sunny');
const themeLabel = computed(() => theme.value === 'system' ? '跟随系统' : theme.value === 'light' ? '浅色' : '深色');
</script>

<template>
  <div class="app-layout">
    <aside class="app-side">
      <div class="app-side__brand">
        <el-icon :size="20"><Tools /></el-icon>
        <span>Code Toolbox</span>
      </div>
      <SideMenu class="app-side__menu" />
    </aside>

    <main class="app-main">
      <header class="app-header">
        <div class="app-header__title">
          <el-breadcrumb separator="/">
            <el-breadcrumb-item :to="{ name: 'home' }">首页</el-breadcrumb-item>
            <el-breadcrumb-item>{{ headerTitle }}</el-breadcrumb-item>
          </el-breadcrumb>
        </div>
        <div class="app-header__actions">
          <el-tooltip content="命令面板 (Cmd/Ctrl+K)">
            <el-button text @click="openPalette" :icon="'Search'" />
          </el-tooltip>
          <el-tooltip :content="`主题:${themeLabel} (点击切换)`">
            <el-button text @click="cycleTheme">
              <el-icon><component :is="themeIcon" /></el-icon>
            </el-button>
          </el-tooltip>
          <el-tooltip content="回到首页">
            <el-button text @click="goHome" :icon="'HomeFilled'" />
          </el-tooltip>
          <el-tooltip content="设置">
            <el-button text @click="goSettings" :icon="'Setting'" />
          </el-tooltip>
        </div>
      </header>

      <section class="app-content">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </section>
    </main>

    <CommandPalette ref="commandPaletteRef" />
  </div>
</template>

<style scoped>
.app-header__title {
  display: flex;
  align-items: center;
  gap: 12px;
}
.app-header__actions {
  display: flex;
  align-items: center;
  gap: 4px;
}
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
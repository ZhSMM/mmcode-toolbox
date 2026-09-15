<script setup lang="ts">
import { computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import SideMenu from './SideMenu.vue';
import { useAppStore } from '@/stores/app';

const route = useRoute();
const router = useRouter();
const appStore = useAppStore();

// 监听路由,把当前工具 ID 写入 store(供工具页判断是否收藏、给历史页过滤用)
watch(
  () => route.path,
  (path) => {
    const tool = appStore.tools.find((t) => t.route === path);
    appStore.currentToolId = tool?.tool_id ?? null;
  },
  { immediate: true }
);

const headerTitle = computed(() => {
  return (route.meta?.title as string) ?? 'Mavis Code Toolbox';
});

const goHome = () => router.push({ name: 'home' });
const goSettings = () => router.push({ name: 'settings' });
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
          <el-button text @click="goHome" :icon="'HomeFilled'" />
          <el-button text @click="goSettings" :icon="'Setting'" />
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
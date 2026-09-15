<script setup lang="ts">
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { useAppStore } from '@/stores/app';

const router = useRouter();
const route = useRoute();
const appStore = useAppStore();

const groups = computed(() => appStore.groupedTools());

const activeMenu = computed(() => route.path);

const handleSelect = (index: string) => {
  // index = /tools/xxx
  if (index.startsWith('/')) router.push(index);
};

// 让 Element Plus 的 ElMenu 支持按 path 激活
const defaultActive = computed(() => route.path);
</script>

<template>
  <el-menu
    class="app-side__menu"
    background-color="transparent"
    text-color="rgba(255,255,255,0.85)"
    active-text-color="#fff"
    :default-active="defaultActive"
    :active="activeMenu"
    @select="handleSelect"
  >
    <el-sub-menu
      v-for="group in groups"
      :key="group.category.id"
      :index="`cat-${group.category.id}`"
    >
      <template #title>
        <el-icon v-if="group.category.icon">
          <component :is="group.category.icon" />
        </el-icon>
        <span>{{ group.category.name }}</span>
      </template>

      <el-menu-item
        v-for="tool in group.tools"
        :key="tool.tool_id"
        :index="tool.route"
      >
        <el-icon v-if="tool.icon"><component :is="tool.icon" /></el-icon>
        <template #title>{{ tool.name }}</template>
      </el-menu-item>
    </el-sub-menu>
  </el-menu>
</template>

<style scoped>
.app-side__menu :deep(.el-menu) {
  border-right: none;
}
.app-side__menu :deep(.el-sub-menu__title:hover),
.app-side__menu :deep(.el-menu-item:hover) {
  background-color: rgba(255, 255, 255, 0.08) !important;
}
.app-side__menu :deep(.el-menu-item.is-active) {
  background-color: var(--el-color-primary) !important;
  color: #fff !important;
}
</style>
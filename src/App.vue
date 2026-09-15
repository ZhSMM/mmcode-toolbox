<script setup lang="ts">
import { onMounted } from 'vue';
import { useAppStore } from '@/stores/app';

const appStore = useAppStore();

onMounted(async () => {
  // 启动即拉菜单和收藏
  await appStore.loadMenu();
  await appStore.loadFavorites();
});
</script>

<template>
  <router-view />
</template>

<style>
#app {
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}
</style>
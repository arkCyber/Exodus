<!--
  Exodus Browser — root Vue shell (router outlet only).
-->
<template>
  <router-view />
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { logStartup } from '@/lib/startupLog';

onMounted(() => {
  logStartup('App.vue mounted');
});
</script>

<style>
html,
body,
#app {
  width: 100%;
  height: 100%;
  margin: 0;
  overflow: hidden;
}
</style>

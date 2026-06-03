<!--
  Exodus Browser — unified window title bar drag zone (tabs + toolbar).
  macOS overlay / frameless chrome: drag the top chrome row to move the window.
-->
<template>
  <header
    id="exodus-window-titlebar"
    class="exodus-window-titlebar window-titlebar"
    data-tauri-drag-region
    @mousedown.capture="onTitleBarMouseDown"
  >
    <slot />
  </header>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — title bar wrapper; forwards blank-area drags to native window move.
 */
import { startWindowDragFromMouseDown } from '$lib/windowDrag';

function onTitleBarMouseDown(e: MouseEvent): void {
  startWindowDragFromMouseDown(e);
}
</script>

<style scoped>
.exodus-window-titlebar {
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  -webkit-app-region: drag;
  app-region: drag;
}
</style>

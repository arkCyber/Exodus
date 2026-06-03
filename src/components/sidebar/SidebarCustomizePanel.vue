<!--
  Exodus Browser — Customize sidebar (Firefox 136 gear panel).
-->
<template>
  <div class="exodus-customize-panel">
    <section class="exodus-customize-section">
      <h4>Sidebar position</h4>
      <label class="exodus-customize-option">
        <input
          type="radio"
          name="sidebar-position"
          value="left"
          :checked="prefs.position === 'left'"
          @change="emit('position-change', 'left')"
        />
        Left
      </label>
      <label class="exodus-customize-option">
        <input
          type="radio"
          name="sidebar-position"
          value="right"
          :checked="prefs.position === 'right'"
          @change="emit('position-change', 'right')"
        />
        Right
      </label>
    </section>

    <section class="exodus-customize-section">
      <h4>Layout</h4>
      <label class="exodus-customize-option">
        <input
          type="checkbox"
          :checked="prefs.verticalTabsInSidebar"
          @change="onVerticalTabsChange"
        />
        Vertical tabs in sidebar
      </label>
      <p class="muted exodus-customize-hint">Hides the top tab strip and shows tabs here (Firefox 136).</p>
    </section>

    <section class="exodus-customize-section">
      <h4>Tools in sidebar</h4>
      <label v-for="tool in catalog" :key="tool.id" class="exodus-customize-option">
        <input
          type="checkbox"
          :checked="prefs.enabledTools.includes(tool.id)"
          @change="emit('toggle-tool', tool.id)"
        />
        {{ tool.firefoxLabel }}
      </label>
    </section>
  </div>
</template>

<script setup lang="ts">
import { SIDEBAR_TOOL_CATALOG, type SidebarPreferences, type SidebarToolId } from '$lib/sidebarPreferences';

defineProps<{ prefs: SidebarPreferences }>();

const emit = defineEmits<{
  'position-change': [position: 'left' | 'right'];
  'vertical-tabs-change': [enabled: boolean];
  'toggle-tool': [tool: SidebarToolId];
}>();

const catalog = SIDEBAR_TOOL_CATALOG;

function onVerticalTabsChange(e: Event): void {
  emit('vertical-tabs-change', (e.target as HTMLInputElement).checked);
}
</script>

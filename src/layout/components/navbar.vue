<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import Setting from "@iconify-icons/ri/settings-3-line";
import Sun from "@iconify-icons/ri/sun-line";
import Moon from "@iconify-icons/ri/moon-line";
import Subtract from "@iconify-icons/ri/subtract-line";
import Fullscreen from "@iconify-icons/ri/fullscreen-line";
import Close from "@iconify-icons/ri/close-line";
import { useNav } from "@/layout/hooks/useNav";
import { useDataThemeChange } from "@/layout/hooks/useDataThemeChange";

const { onPanel } = useNav();
const { dataTheme, dataThemeChange } = useDataThemeChange();
const appWindow = getCurrentWindow();

function themeChange() {
  dataTheme.value = !dataTheme.value;
  dataThemeChange();
}
</script>

<template>
  <div
    class="navbar bg-[#fff] shadow-sm shadow-[rgba(0, 21, 41, 0.08)] dark:shadow-[#0d0d0d]"
  >
    <div class="vertical-header-right">
      <span @click="themeChange" class="set-icon navbar-bg-hover">
        <IconifyIconOffline v-if="dataTheme" :icon="Moon" />
        <IconifyIconOffline v-else :icon="Sun" />
      </span>
      <span class="set-icon navbar-bg-hover" @click="onPanel">
        <IconifyIconOffline :icon="Setting" />
      </span>
      <span class="set-icon navbar-bg-hover" @click="appWindow.minimize">
        <IconifyIconOffline :icon="Subtract" />
      </span>
      <span class="set-icon navbar-bg-hover" @click="appWindow.toggleMaximize">
        <IconifyIconOffline :icon="Fullscreen" />
      </span>
      <span class="set-icon navbar-bg-hover" @click="appWindow.close">
        <IconifyIconOffline :icon="Close" />
      </span>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.navbar {
  width: 100%;
  height: 36px;

  .vertical-header-right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    min-width: 280px;
    height: 36px;
  }
}
</style>

<script setup lang="ts">
import { computed, ref } from "vue";
import { onClickOutside } from "@vueuse/core";
import { useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Setting from "@iconify-icons/ri/settings-3-line";
import Sun from "@iconify-icons/ri/sun-line";
import Moon from "@iconify-icons/ri/moon-line";
import Subtract from "@iconify-icons/ri/subtract-line";
import Fullscreen from "@iconify-icons/ri/fullscreen-line";
import Close from "@iconify-icons/ri/close-line";
import { useNav } from "@/layout/hooks/useNav";
import { useDataThemeChange } from "@/layout/hooks/useDataThemeChange";
import { useCommandStore } from "@/store/modules/commands";

const { onPanel } = useNav();
const { dataTheme, dataThemeChange } = useDataThemeChange();
const appWindow = getCurrentWindow();
const router = useRouter();
const commandStore = useCommandStore();
const { commands } = storeToRefs(commandStore);
const searchQuery = ref("");
const showCmdList = ref(false);
const containerRef = ref(null);
const filterCmd = computed(() => {
  return commands.value.filter(cmd =>
    cmd.title.toLowerCase().includes(searchQuery.value.toLowerCase())
  );
});

function themeChange() {
  dataTheme.value = !dataTheme.value;
  dataThemeChange();
}
function navigateToCommand(routePath) {
  router.push({ path: routePath });
  searchQuery.value = "";
  showCmdList.value = false;
}
onClickOutside(containerRef, () => {
  showCmdList.value = false;
});
</script>

<template>
  <div
    class="navbar bg-[#fff] shadow-sm shadow-[rgba(0, 21, 41, 0.08)] dark:shadow-[#0d0d0d]"
    ref="containerRef"
  >
    <div class="vertical-header-right">
      <div class="search-container">
        <el-input
          v-model="searchQuery"
          placeholder="Search for InsightSQL"
          @focus="showCmdList = true"
        />
        <el-scrollbar v-if="showCmdList && filterCmd.length" class="cmd-list">
          <el-form
            v-for="cmd in filterCmd"
            :key="cmd.route"
            @click="navigateToCommand(cmd.route)"
            class="cmd-item"
          >
            <span>{{ cmd.title }}</span>
          </el-form>
        </el-scrollbar>
      </div>
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

.search-container {
  position: relative;
}
.cmd-list {
  border: 1px solid #ddd;
  height: 200px;
  position: absolute;
  width: 100%;
  z-index: 1000;
}
.cmd-item {
  padding: 8px 16px;
  cursor: pointer;
}
.cmd-item:hover {
  background-color: #f6dada;
}
</style>

<script setup lang="ts">
import { ref, watch, nextTick, computed } from "vue";
import { onClickOutside } from "@vueuse/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Setting from "@iconify-icons/ri/settings-3-line";
import Sun from "@iconify-icons/ri/sun-line";
import Moon from "@iconify-icons/ri/moon-line";
import Subtract from "@iconify-icons/ri/subtract-line";
import Fullscreen from "@iconify-icons/ri/fullscreen-line";
import Close from "@iconify-icons/ri/close-line";
import SidebarItem from "./sidebarItem.vue";
import { useNav } from "@/layout/hooks/useNav";
import { usePermissionStoreHook } from "@/store/modules/permission";
import { useDataThemeChange } from "@/layout/hooks/useDataThemeChange";
import { useCommandStore } from "@/store/modules/commands";
import { storeToRefs } from "pinia";
import { useRouter } from "vue-router";

const menuRef = ref();
const { route, routers, onPanel, menuSelect } = useNav();
const { dataTheme, dataThemeChange } = useDataThemeChange();
const appWindow = getCurrentWindow();
const containerRef = ref(null);

function themeChange() {
  dataTheme.value = !dataTheme.value;
  dataThemeChange();
}

nextTick(() => {
  menuRef.value?.handleResize();
});

watch(
  () => route.path,
  () => {
    menuSelect(route.path, routers);
  }
);

const router = useRouter();
const commandStore = useCommandStore();
const { commands } = storeToRefs(commandStore);
const searchQuery = ref("");
const showCmdList = ref(false);
const filterCmd = computed(() => {
  return commands.value.filter(command =>
    command.title.toLowerCase().includes(searchQuery.value.toLowerCase())
  );
});

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
    v-loading="usePermissionStoreHook().wholeMenus.length === 0"
    class="horizontal-header"
    ref="containerRef"
  >
    <el-menu
      router
      ref="menuRef"
      mode="horizontal"
      class="horizontal-header-menu"
      :default-active="route.path"
      @select="indexPath => menuSelect(indexPath, routers)"
    >
      <sidebar-item
        v-for="route in usePermissionStoreHook().wholeMenus"
        :key="route.path"
        :item="route"
        :base-path="route.path"
      />
    </el-menu>
    <div class="search-container">
      <el-input
        v-model="searchQuery"
        placeholder="Search for InsightSQL"
        @focus="showCmdList = true"
      />
      <el-scrollbar v-if="showCmdList && filterCmd.length" class="cmd-list">
        <div
          v-for="cmd in filterCmd"
          :key="cmd.route"
          @click="navigateToCommand(cmd.route)"
          class="cmd-item"
        >
          <span>{{ cmd.title }}</span>
        </div>
      </el-scrollbar>
    </div>
    <div class="horizontal-header-right">
      <span @click="themeChange" class="set-icon navbar-bg-hover">
        <IconifyIconOffline v-if="dataTheme" :icon="Moon" />
        <IconifyIconOffline v-else :icon="Sun" />
      </span>
      <span class="set-icon navbar-bg-hover" title="setting" @click="onPanel">
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
:deep(.el-loading-mask) {
  opacity: 0.45;
}

.logout {
  max-width: 120px;

  ::v-deep(.el-dropdown-menu__item) {
    display: inline-flex;
    flex-wrap: wrap;
    min-width: 100%;
  }
}
.search-container {
  position: relative;
  width: 160px;
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

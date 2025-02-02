<script setup lang="ts">
import { ref, watch, nextTick, computed } from "vue";
import SidebarItem from "./sidebarItem.vue";
import { useNav } from "@/layout/hooks/useNav";
import { usePermissionStoreHook } from "@/store/modules/permission";
import Setting from "@iconify-icons/ri/settings-3-line";
import { useDataThemeChange } from "@/layout/hooks/useDataThemeChange";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Minus,
  CopyDocument,
  Close,
  Sunny,
  Moon
} from "@element-plus/icons-vue";
import { useCommandStore } from "@/store/modules/commands";
import { storeToRefs } from "pinia";
import { useRouter } from "vue-router";

const menuRef = ref();

const { route, routers, onPanel, menuSelect } = useNav();
const { dataTheme, dataThemeChange } = useDataThemeChange();
const appWindow = getCurrentWindow();

function themeChange() {
  dataTheme.value = !dataTheme.value;
  dataThemeChange();
}

const handleMouseDown = e => {
  if (
    e.target.closest(".horizontal-header-menu") ||
    e.target.closest(".set-icon") ||
    e.target.closest(".search-container")
  ) {
    return;
  }
  if (e.buttons === 1) {
    appWindow.startDragging();
  }
};

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
const filteredCommands = computed(() => {
  return commands.value.filter(command =>
    command.title.toLowerCase().includes(searchQuery.value.toLowerCase())
  );
});
function navigateToCommand(routePath) {
  router.push({ path: routePath });
  searchQuery.value = "";
}
</script>

<template>
  <div
    v-loading="usePermissionStoreHook().wholeMenus.length === 0"
    class="horizontal-header"
    @mousedown="handleMouseDown"
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
        @click="filteredCommands"
      />

      <el-scrollbar
        v-if="searchQuery && filteredCommands.length"
        class="command-list"
      >
        <el-form
          v-for="command in filteredCommands"
          :key="command.route"
          @click="navigateToCommand(command.route)"
          class="command-item"
        >
          <span>{{ command.title }}</span>
        </el-form>
      </el-scrollbar>
    </div>

    <div class="horizontal-header-right">
      <span @click="themeChange" class="set-icon navbar-bg-hover">
        <el-icon v-if="dataTheme"><Moon /></el-icon>
        <el-icon v-else><Sunny /></el-icon>
      </span>
      <span class="set-icon navbar-bg-hover" title="setting" @click="onPanel">
        <IconifyIconOffline :icon="Setting" />
      </span>
      <span
        class="set-icon navbar-bg-hover"
        id="minimize"
        title="zoom out"
        @click="appWindow.minimize"
      >
        <el-icon><Minus /></el-icon>
      </span>
      <span
        class="set-icon navbar-bg-hover"
        id="maximize"
        title="zoom in"
        @click="appWindow.toggleMaximize"
      >
        <el-icon><CopyDocument /></el-icon>
      </span>
      <span
        class="set-icon navbar-bg-hover"
        id="close"
        title="close"
        @click="appWindow.close"
      >
        <el-icon><Close /></el-icon>
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
}
.command-list {
  border: 1px solid #ddd;
  height: 200px;
  position: absolute;
  width: 100%;
  z-index: 1000;
}
.command-item {
  padding: 8px 16px;
  cursor: pointer;
}
.command-item:hover {
  background-color: #f6dada;
}
</style>

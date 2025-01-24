<script setup lang="ts">
import extraIcon from "./extraIcon.vue";
import { useNav } from "@/layout/hooks/useNav";
import { ref, toRaw, watch, onMounted, nextTick } from "vue";
import { useRenderIcon } from "@/components/ReIcon/src/hooks";
import { getParentPaths, findRouteByPath } from "@/router/utils";
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

const { dataTheme, dataThemeChange } = useDataThemeChange();
const appWindow = getCurrentWindow();

function themeChange() {
  dataTheme.value = !dataTheme.value;
  dataThemeChange();
}

const handleMouseDown = e => {
  if (e.target.closest(".set-icon")) {
    return;
  }
  if (e.buttons === 1) {
    appWindow.startDragging();
  }
};

const menuRef = ref();
const defaultActive = ref(null);

const {
  route,
  device,
  routers,
  onPanel,
  menuSelect,
  resolvePath,
  getDivStyle
} = useNav();

function getDefaultActive(routePath) {
  const wholeMenus = usePermissionStoreHook().wholeMenus;
  /** 当前路由的父级路径 */
  const parentRoutes = getParentPaths(routePath, wholeMenus)[0];
  defaultActive.value = findRouteByPath(
    parentRoutes,
    wholeMenus
  )?.children[0]?.path;
}

onMounted(() => {
  getDefaultActive(route.path);
});

nextTick(() => {
  menuRef.value?.handleResize();
});

watch(
  () => [route.path, usePermissionStoreHook().wholeMenus],
  () => {
    getDefaultActive(route.path);
  }
);
</script>

<template>
  <div
    v-if="device !== 'mobile'"
    class="horizontal-header"
    v-loading="usePermissionStoreHook().wholeMenus.length === 0"
  >
    <el-menu
      router
      ref="menuRef"
      mode="horizontal"
      class="horizontal-header-menu"
      :default-active="defaultActive"
      @select="indexPath => menuSelect(indexPath, routers)"
    >
      <el-menu-item
        v-for="route in usePermissionStoreHook().wholeMenus"
        :key="route.path"
        :index="resolvePath(route) || route.redirect"
      >
        <template #title>
          <div
            v-if="toRaw(route.meta.icon)"
            :class="['sub-menu-icon', route.meta.icon]"
          >
            <component
              :is="useRenderIcon(route.meta && toRaw(route.meta.icon))"
            />
          </div>
          <div :style="getDivStyle">
            <span class="select-none">
              {{ route.meta.title }}
            </span>
            <extraIcon :extraIcon="route.meta.extraIcon" />
          </div>
        </template>
      </el-menu-item>
    </el-menu>
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
</style>

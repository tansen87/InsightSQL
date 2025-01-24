<script setup lang="ts">
import mixNav from "./sidebar/mixNav.vue";
import { useNav } from "@/layout/hooks/useNav";
import Breadcrumb from "./sidebar/breadCrumb.vue";
import topCollapse from "./sidebar/topCollapse.vue";
import Setting from "@iconify-icons/ri/settings-3-line";
import { useDataThemeChange } from "@/layout/hooks/useDataThemeChange";
import dayIcon from "@/assets/svg/day.svg?component";
import darkIcon from "@/assets/svg/dark.svg?component";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Minus,
  CopyDocument,
  Close,
  Sunny,
  Moon
} from "@element-plus/icons-vue";

const { layout, device, onPanel, pureApp, toggleSideBar } = useNav();
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
</script>

<template>
  <div @mousedown="handleMouseDown">
    <div
      class="navbar bg-[#fff] shadow-sm shadow-[rgba(0, 21, 41, 0.08)] dark:shadow-[#0d0d0d]"
    >
      <topCollapse
        v-if="device === 'mobile'"
        class="hamburger-container"
        :is-active="pureApp.sidebar.opened"
        @toggleClick="toggleSideBar"
      />

      <Breadcrumb
        v-if="layout !== 'mix' && device !== 'mobile'"
        class="breadcrumb-container"
      />

      <mixNav v-if="layout === 'mix'" />

      <div v-if="layout === 'vertical'" class="vertical-header-right">
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
  </div>
</template>

<style lang="scss" scoped>
.navbar {
  width: 100%;
  height: 48px;
  overflow: hidden;

  .hamburger-container {
    float: left;
    height: 100%;
    line-height: 48px;
    cursor: pointer;
  }

  .vertical-header-right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    min-width: 280px;
    height: 48px;
    color: #000000d9;

    .el-dropdown-link {
      display: flex;
      align-items: center;
      justify-content: space-around;
      height: 48px;
      padding: 10px;
      color: #000000d9;
      cursor: pointer;

      p {
        font-size: 14px;
      }

      img {
        width: 22px;
        height: 22px;
        border-radius: 50%;
      }
    }
  }

  .breadcrumb-container {
    float: left;
    margin-left: 16px;
  }
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

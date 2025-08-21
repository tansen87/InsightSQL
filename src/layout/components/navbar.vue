<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Setting from "@iconify-icons/ri/settings-3-line";
import Sun from "@iconify-icons/ri/sun-line";
import Moon from "@iconify-icons/ri/moon-line";
import Search from "@iconify-icons/ri/search-2-line";
import Subtract from "@iconify-icons/ri/subtract-line";
import Fullscreen from "@iconify-icons/ri/fullscreen-line";
import Close from "@iconify-icons/ri/close-line";
import Breadcrumb from "./sidebar/breadCrumb.vue";
import topCollapse from "./sidebar/topCollapse.vue";
import { useNav } from "@/layout/hooks/useNav";
import { useDataThemeChange } from "@/layout/hooks/useDataThemeChange";
import { useCommandStore } from "@/store/modules/commands";

const { layout, device, onPanel, pureApp, toggleSideBar } = useNav();
const { dataTheme, dataThemeChange } = useDataThemeChange();
const appWindow = getCurrentWindow();
const router = useRouter();
const commandStore = useCommandStore();
const { commands } = storeToRefs(commandStore);
const searchQuery = ref("");
const showDialog = ref(false);
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
  showDialog.value = false;
}
</script>

<template>
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

    <div v-if="layout === 'vertical'" class="vertical-header-right">
      <div class="search-container">
        <span class="set-icon navbar-bg-hover" @click="showDialog = true">
          <IconifyIconOffline :icon="Search" />
        </span>
        <el-dialog v-model="showDialog">
          <el-input v-model="searchQuery" placeholder="Search for InsightSQL" />
          <el-scrollbar class="cmd-list">
            <el-form
              v-for="cmd in filterCmd"
              :key="cmd.route"
              @click="navigateToCommand(cmd.route)"
              class="cmd-item"
            >
              <span>{{ cmd.title }}</span>
            </el-form>
          </el-scrollbar>
        </el-dialog>
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
  height: 40px;
  overflow: hidden;

  .hamburger-container {
    float: left;
    height: 100%;
    line-height: 40px;
    cursor: pointer;
  }

  .vertical-header-right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    min-width: 280px;
    height: 40px;
    color: #000000d9;
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

.search-container {
  position: relative;
}
.cmd-list {
  border: 1px solid #ddd;
  height: 200px;
  position: absolute;
  left: 0;
  right: 0;
  top: 100%;
  z-index: 1000;
  background: #fff;
}
.cmd-item {
  padding: 8px 16px;
  cursor: pointer;
}
.cmd-item:hover {
  background-color: #f6dada;
}
</style>

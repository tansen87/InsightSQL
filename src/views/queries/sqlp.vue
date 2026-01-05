<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, toRef } from "vue";
import {
  FolderOpened,
  Download,
  View,
  Hide,
  Smoking,
  NoSmoking,
  Delete,
  CopyDocument,
  ArrowRight,
  DArrowRight
} from "@element-plus/icons-vue";
import { Icon } from "@iconify/vue";
import { VAceEditor } from "vue3-ace-editor";
import { useDark } from "@pureadmin/utils";
import "@/utils/sql/aceConfig";
import { useDynamicHeight } from "@/utils/utils";
import { useSqlHistory } from "@/store/modules/sqlHistory";
import { useSqlTabManager } from "@/utils/sql/sqlTabManager";
import { useSqlFileTree } from "@/utils/sql/sqlFileTree";

const data = reactive({
  write: false,
  limit: true,
  varchar: true
});
const [isRunningCurrent, isRunningNew, isExporting] = [
  ref(false),
  ref(false),
  ref(false)
];
const limitContent = ref("limit 500");
const varcharContent = ref("dtype: string");
const sqlQuery = ref('select\n*\nfrom "filename"\nlimit 100');

const { dynamicHeight } = useDynamicHeight(36);
const { isDark } = useDark();
const sqlHistory = useSqlHistory();
const {
  tabs,
  activeTabId,
  activeTab,
  pagedTableData,
  tablePanelSize,
  runCurrentTab,
  runNewTab,
  sizeChange,
  currentChange,
  exportActiveTab,
  removeTab
} = useSqlTabManager({
  sqlQuery,
  path: computed(() => sqlHistory.path),
  varchar: toRef(data, "varchar"),
  limit: toRef(data, "limit")
});
const {
  fileTreeData,
  getNodeIcon,
  selectFile,
  contextMenuVisible,
  contextMenuPosition,
  contextMenuItem,
  closeContextMenu,
  rightClick,
  copyPath,
  copyFileName,
  copyFieldName,
  deleteFile
} = useSqlFileTree();
const theme = computed(() => (isDark.value ? "monokai" : "chrome"));

function limitRows() {
  data.limit = !data.limit;
  limitContent.value = data.limit ? "limit 500" : "not limit";
}

function allVarchar() {
  data.varchar = !data.varchar;
  varcharContent.value = data.varchar ? "dtype: string" : "dtype: auto";
}

// tree右键菜单
const contextMenuRef = ref<HTMLElement | null>(null);

function clickOutside(event: MouseEvent) {
  if (
    contextMenuVisible.value &&
    contextMenuRef.value &&
    !contextMenuRef.value.contains(event.target as Node)
  ) {
    closeContextMenu();
  }
}

onMounted(() => {
  document.addEventListener("click", clickOutside);
});

onUnmounted(() => {
  document.removeEventListener("click", clickOutside);
});
</script>

<template>
  <el-form class="page-container" :style="{ height: dynamicHeight + 'px' }">
    <el-splitter>
      <el-splitter-panel size="150">
        <el-splitter layout="vertical">
          <el-tooltip content="Add data" effect="light">
            <el-button @click="selectFile()" :icon="FolderOpened" circle text />
          </el-tooltip>

          <el-splitter-panel>
            <el-scrollbar class="flex-1">
              <el-tree
                :data="fileTreeData"
                @node-contextmenu="rightClick"
                empty-text=""
              >
                <template #default="{ data }">
                  <span class="flex items-center">
                    <Icon
                      :icon="getNodeIcon(data)"
                      width="14"
                      height="14"
                      class="mr-1"
                    />
                    <span>{{ data.label }}</span>
                  </span>
                </template>
              </el-tree>
            </el-scrollbar>
          </el-splitter-panel>
        </el-splitter>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-splitter layout="vertical">
          <div class="flex items-center">
            <el-tooltip content="Run" effect="light">
              <el-button
                @click="runCurrentTab"
                :loading="isRunningCurrent"
                :icon="ArrowRight"
                circle
                text
              >
                <template #icon>
                  <Icon icon="mdi:menu-right-outline" />
                </template>
              </el-button>
            </el-tooltip>
            <el-tooltip content="Run in new tab" effect="light">
              <el-button
                @click="runNewTab"
                :loading="isRunningNew"
                :icon="DArrowRight"
                circle
                text
              />
            </el-tooltip>
            <el-tooltip :content="limitContent" effect="light">
              <el-button @click="limitRows" circle text>
                <el-icon>
                  <Hide v-if="data.limit" />
                  <View v-else />
                </el-icon>
              </el-button>
            </el-tooltip>
            <el-tooltip :content="varcharContent" effect="light">
              <el-button @click="allVarchar" circle text>
                <el-icon>
                  <NoSmoking v-if="data.varchar" />
                  <Smoking v-else />
                </el-icon>
              </el-button>
            </el-tooltip>
          </div>

          <el-splitter-panel>
            <VAceEditor
              v-model:value="sqlQuery"
              ref="editor"
              lang="sql"
              :options="{
                useWorker: true,
                enableBasicAutocompletion: true,
                enableSnippets: true,
                enableLiveAutocompletion: true,
                customScrollbar: true,
                showPrintMargin: false,
                fontSize: '1.0rem'
              }"
              :theme="theme"
              style="height: 100%"
            />
          </el-splitter-panel>

          <el-splitter-panel
            v-if="tabs.length > 0"
            :size="tablePanelSize"
            min="35"
            class="flex flex-col"
          >
            <div class="flex flex-col h-full overflow-hidden">
              <el-tabs
                v-model="activeTabId"
                type="card"
                closable
                @tab-remove="removeTab"
                class="mb-[-15px]"
                style="--el-tabs-header-height: 32px"
              >
                <el-tab-pane
                  v-for="tab in tabs"
                  :key="tab.id"
                  :label="tab.title"
                  :name="tab.id"
                />
              </el-tabs>

              <div v-if="activeTab && activeTab.data.length > 0">
                <div class="flex justify-between items-center text-sm">
                  <el-pagination
                    v-model:current-page="activeTab!.currentPage"
                    v-model:page-size="activeTab!.pageSize"
                    :total="activeTab!.total"
                    layout="total, prev, pager, next"
                    size="small"
                    :simplified="true"
                    @size-change="sizeChange"
                    @current-change="currentChange"
                    background
                    :pager-count="5"
                  />
                  <div class="flex items-center gap-2">
                    <el-button
                      @click="exportActiveTab"
                      :loading="isExporting"
                      :icon="Download"
                      circle
                      text
                    />
                  </div>
                </div>
              </div>

              <el-table
                v-if="activeTab && activeTab.data.length > 0"
                :data="pagedTableData"
                height="100%"
                show-overflow-tooltip
                tooltip-effect="light"
                class="flex-1"
              >
                <el-table-column
                  v-for="col in activeTab?.columns"
                  :key="col.prop"
                  :prop="col.prop"
                  :label="col.label"
                  width="150px"
                />
              </el-table>
            </div>
          </el-splitter-panel>
        </el-splitter>
      </el-splitter-panel>
    </el-splitter>

    <div
      v-show="contextMenuVisible"
      ref="contextMenuRef"
      class="context-menu"
      :style="{
        left: contextMenuPosition.x + 'px',
        top: contextMenuPosition.y + 'px'
      }"
      @click.stop
    >
      <div
        v-if="contextMenuItem?.type === 'file'"
        class="context-menu-item"
        type="button"
        @click="copyFileName"
      >
        <el-icon><CopyDocument /></el-icon> Copy File Name
      </div>
      <div
        v-if="contextMenuItem?.type === 'file'"
        class="context-menu-item"
        type="button"
        @click="copyPath"
      >
        <el-icon><CopyDocument /></el-icon> Copy Path
      </div>
      <div
        v-if="contextMenuItem?.type === 'file'"
        class="context-menu-item"
        type="button"
        @click="deleteFile"
      >
        <el-icon><Delete /></el-icon> Delete
      </div>
      <div
        v-if="contextMenuItem?.type === 'field'"
        class="context-menu-item"
        type="button"
        @click="copyFieldName"
      >
        <el-icon><CopyDocument /></el-icon> Copy Field Name
      </div>
    </div>
  </el-form>
</template>

<style>
.ace_gutter {
  background: transparent !important;
}

.context-menu {
  position: fixed;
  z-index: 2000;
  background: var(--el-bg-color-overlay);
  border: 1px solid var(--el-border-color-light);
  border-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  min-width: 120px;
  padding: 4px 0;
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  cursor: pointer;
  font-size: 12px;
  color: var(--el-text-color-primary);
}

.context-menu-item:hover {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}

.context-menu-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  z-index: 1999;
}
</style>

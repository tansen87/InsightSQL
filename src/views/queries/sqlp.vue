<script setup lang="ts">
import {
  ref,
  reactive,
  computed,
  markRaw,
  onMounted,
  onUnmounted,
  triggerRef
} from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import {
  FolderOpened,
  Download,
  View,
  Hide,
  Smoking,
  NoSmoking,
  Delete,
  CopyDocument,
  ArrowRight
} from "@element-plus/icons-vue";
import { v4 as uuidv4 } from "uuid";
import { Icon } from "@iconify/vue";
import fileIcon from "@iconify-icons/ri/file-line";
import fileTextIcon from "@iconify-icons/ri/file-text-line";
import fileExcelIcon from "@iconify-icons/ri/file-excel-2-line";
import fileJsonIcon from "@iconify-icons/ri/file-code-line";
import fileParquetIcon from "@iconify-icons/ri/database-2-line";
import textIcon from "@iconify-icons/ri/text";
import intIcon from "@iconify-icons/ri/number-1";
import floatIcon from "@iconify-icons/ri/hashtag";
import boolIcon from "@iconify-icons/ri/checkbox-circle-line";
import dateIcon from "@iconify-icons/ri/calendar-event-line";
import listIcon from "@iconify-icons/ri/list-unordered";
import unknowIcon from "@iconify-icons/ri/question-mark";
import { VAceEditor } from "vue3-ace-editor";
import { useDark } from "@pureadmin/utils";
import "./ace-config";
import { useDynamicHeight } from "@/utils/utils";
import { message } from "@/utils/message";
import { useSqlHistory } from "@/store/modules/sqlHistory";

const data = reactive({
  write: false,
  writeFormat: "xlsx",
  limit: true,
  varchar: true
});
const tables = ref([]);
const isRunningCurrent = ref(false);
const isRunningNew = ref(false);
const isExporting = ref(false);
const limitContent = ref("limit 500");
const varcharContent = ref("dtype: string");
const sqlQuery = ref("select\n*\nfrom _t_1\nlimit 100");

const { dynamicHeight } = useDynamicHeight(36);
const { isDark } = useDark();
const sqlHistory = useSqlHistory();
const theme = computed(() => (isDark.value ? "monokai" : "chrome"));

// 提取文件名
function extractDisplayName(fullPath: string): string {
  const parts = fullPath.split(/[/\\]/);
  const fileName = parts[parts.length - 1];
  const dotIndex = fileName.lastIndexOf(".");
  return dotIndex > 0 && dotIndex < fileName.length - 1
    ? fileName.substring(0, dotIndex)
    : fileName;
}

// 基于store的路径生成元数据
const viewFileMeta = computed(() => {
  if (!sqlHistory.path) return [];
  return sqlHistory.path.split("|").map(path => {
    const fullFileName = path.split(/[/\\]/).pop() || path;
    const displayName = extractDisplayName(path);
    const ext = fullFileName.includes(".")
      ? fullFileName.slice(fullFileName.lastIndexOf(".") + 1).toLowerCase()
      : "";
    return { fullPath: path, fullFileName, displayName, ext };
  });
});

const fileTreeData = computed(() => {
  return viewFileMeta.value
    .map(fileMeta => {
      const schema = sqlHistory.dtypesByFile[fileMeta.displayName];
      if (!schema || Object.keys(schema).length === 0) return null;

      const children = Object.entries(schema).map(([field, dtype]) => ({
        label: field,
        dtype,
        key: `${fileMeta.displayName}-${field}`,
        type: "field"
      }));

      return {
        label: fileMeta.displayName,
        ext: fileMeta.ext,
        children,
        key: fileMeta.displayName,
        type: "file"
      };
    })
    .filter(Boolean) as any[];
});

interface ResultTab {
  id: string;
  title: string;
  columns: { prop: string; label: string }[];
  data: any[];
  currentPage: number;
  pageSize: number;
  total: number;
}

const tabs = ref<ResultTab[]>([]);
const activeTabId = ref<string | null>(null);

const activeTab = computed(() => {
  return tabs.value.find(t => t.id === activeTabId.value) || null;
});

// 当前分页数据（仅用于表格渲染）
const pagedTableData = computed(() => {
  if (!activeTab.value) return [];
  const start = (activeTab.value.currentPage - 1) * activeTab.value.pageSize;
  return activeTab.value.data.slice(start, start + activeTab.value.pageSize);
});

const tablePanelSize = computed(() => {
  return pagedTableData.value?.length > 0 ? "40%" : "30";
});

function createEmptyTab(): ResultTab {
  const count = tabs.value.length + 1;
  return {
    id: uuidv4(),
    title: `Query ${count}`,
    columns: [],
    data: [],
    currentPage: 1,
    pageSize: 50,
    total: 0
  };
}

// 图标逻辑
const getFileIcon = (ext: string) => {
  switch (ext) {
    case "csv":
    case "tsv":
    case "txt":
    case "dat":
      return fileTextIcon;
    case "xlsx":
    case "xls":
    case "xlsb":
    case "xlsm":
    case "ods":
      return fileExcelIcon;
    case "json":
    case "jsonl":
    case "ndjson":
      return fileJsonIcon;
    case "parquet":
      return fileParquetIcon;
    default:
      return fileIcon;
  }
};

const getFieldIcon = (dtype: string) => {
  const d = dtype.toLowerCase();
  if (d.includes("str") || d.includes("utf8")) return textIcon;
  if (d.includes("i64")) return intIcon;
  if (d.includes("f64")) return floatIcon;
  if (d.includes("bool")) return boolIcon;
  if (d.includes("date") || d.includes("time")) return dateIcon;
  if (d.includes("list") || d.includes("struct")) return listIcon;
  return unknowIcon;
};

const getNodeIcon = (node: any) => {
  return node.type === "file"
    ? getFileIcon(node.ext || "")
    : getFieldIcon(node.dtype || "");
};

const defaultProps = {
  children: "children",
  label: "label"
};

async function executeQuery(tab: ResultTab, write: boolean): Promise<boolean> {
  if (sqlHistory.path === "" || sqlQuery.value.trim() === "") {
    message("File not selected or SQL is empty", { type: "warning" });
    return false;
  }

  try {
    const rawResult = await invoke("query", {
      path: sqlHistory.path,
      sqlQuery: sqlQuery.value,
      write,
      writeFormat: data.writeFormat,
      varchar: data.varchar,
      limit: data.limit
    });

    if (!write) {
      const result =
        typeof rawResult === "string" ? JSON.parse(rawResult) : rawResult;
      const jsonData = JSON.parse(result.data);
      const arrayData = Array.isArray(jsonData) ? jsonData : [jsonData];

      tab.columns = result.columns.map(key => ({ prop: key, label: key }));
      tab.data = markRaw(arrayData);
      tab.total = arrayData.length;
      tab.currentPage = 1;
      // 强制触发响应式更新
      triggerRef(tabs);
    } else {
      message("Export done", { type: "success" });
    }
    return true;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 5000 });
    return false;
  }
}

async function runInCurrentTab() {
  if (!activeTab.value) {
    // 自动创建第一个 tab
    const newTab = createEmptyTab();
    tabs.value.push(newTab);
    activeTabId.value = newTab.id;
    isRunningCurrent.value = true;
    await executeQuery(newTab, false);
    isRunningCurrent.value = false;
  } else {
    // 清空当前 tab 并重新查询
    activeTab.value.columns = [];
    activeTab.value.data = [];
    activeTab.value.total = 0;
    await executeQuery(activeTab.value, false);
  }
}

async function runInNewTab() {
  const newTab = createEmptyTab();
  tabs.value.push(newTab);
  activeTabId.value = newTab.id;
  isRunningNew.value = true;
  await executeQuery(newTab, false);
  isRunningNew.value = false;
}

function sizeChange(newSize: number) {
  if (activeTab.value) {
    activeTab.value.pageSize = newSize;
    activeTab.value.currentPage = 1;
  }
}

function currentChange(newPage: number) {
  if (activeTab.value) {
    activeTab.value.currentPage = newPage;
  }
}

async function exportActiveTab() {
  if (!activeTab.value) return;
  isExporting.value = true;
  await executeQuery(activeTab.value, true);
  isExporting.value = false;
}

function removeTab(targetId: string) {
  const index = tabs.value.findIndex(t => t.id === targetId);
  if (index !== -1) {
    const tab = tabs.value[index];

    if (tab.data && Array.isArray(tab.data)) {
      tab.data.length = 0;
      tab.data = null;
    }

    tab.columns = [];
    tabs.value.splice(index, 1);

    if (activeTabId.value === targetId) {
      activeTabId.value = tabs.value.length > 0 ? tabs.value[0].id : null;
    }
  }
}

// 文件选择(更新store)
async function selectFile() {
  const selected = await open({
    multiple: true,
    filters: [
      { name: "All", extensions: ["*"] },
      { name: "csv", extensions: ["csv", "tsv", "psv", "txt", "dat"] },
      { name: "excel", extensions: ["xls", "xlsx", "xlsb", "xlsm", "ods"] },
      { name: "json", extensions: ["json"] },
      { name: "jsonl", extensions: ["jsonl", "ndjson"] },
      { name: "parquet", extensions: ["parquet"] }
    ]
  });

  if (!selected) return;

  const newPaths = Array.isArray(selected) ? selected : [selected];
  const existingPaths = sqlHistory.path ? sqlHistory.path.split("|") : [];
  const allPathsSet = new Set([...existingPaths, ...newPaths]);
  sqlHistory.path = Array.from(allPathsSet).join("|");

  // 加载新文件的schema
  await Promise.all(
    newPaths.map(async path => {
      const basename = extractDisplayName(path);
      if (sqlHistory.dtypesByFile[basename]) return;

      try {
        const rawResult = await invoke("query", {
          path,
          sqlQuery: `select * from "${basename}" limit 10`,
          write: false,
          writeFormat: "csv",
          varchar: false,
          limit: true
        });
        const result =
          typeof rawResult === "string" ? JSON.parse(rawResult) : rawResult;
        sqlHistory.dtypesByFile[basename] = result.schema;
      } catch (err) {
        message(`Failed to load schema for ${basename}: ${err}`, {
          type: "error"
        });
      }
    })
  );
}

const selectViewFile = async () => {
  await selectFile();
};

function limitRows() {
  data.limit = !data.limit;
  limitContent.value = data.limit ? "limit 500" : "not limit";
}

function allVarchar() {
  data.varchar = !data.varchar;
  varcharContent.value = data.varchar ? "dtype: string" : "dtype: auto";
}

// Ace Editor初始化
const initializeEditor = editor => {
  editor.completers.push({
    getCompletions: (editor, session, pos, prefix, callback) => {
      callback(
        null,
        tables.value.map(table => ({
          caption: table.name,
          value: table.name,
          meta: "table"
        }))
      );
    }
  });
  tables.value.forEach(item => {
    editor.completers.push({
      getCompletions: (editor, session, pos, prefix, callback) => {
        callback(
          null,
          item.children.map(col => ({
            caption: col.label,
            value: col.label,
            meta: "column"
          }))
        );
      }
    });
  });
};

// tree右键菜单
const contextMenuRef = ref<HTMLElement | null>(null);
const contextMenuVisible = ref(false);
const contextMenuPosition = reactive({ x: 0, y: 0 });
const contextMenuItem = ref<any>(null);

function closeContextMenu() {
  contextMenuVisible.value = false;
  contextMenuItem.value = null;
}

function openContextMenu(event: MouseEvent, nodeData: any) {
  event.preventDefault();
  contextMenuItem.value = nodeData;
  contextMenuPosition.x = event.clientX;
  contextMenuPosition.y = event.clientY;
  contextMenuVisible.value = true;
}

async function copyPath() {
  if (!contextMenuItem.value?.fullPath) return;
  try {
    await navigator.clipboard.writeText(contextMenuItem.value.fullPath);
    message("Copied file path", { type: "success" });
    closeContextMenu();
  } catch (err) {
    message("Failed to copy", { type: "error" });
  }
}

async function copyFileName() {
  if (!contextMenuItem.value?.label) return;
  try {
    await navigator.clipboard.writeText(contextMenuItem.value.label);
    message("Copied file name", { type: "success" });
    closeContextMenu();
  } catch (err) {
    message("Failed to copy", { type: "error" });
  }
}

async function copyFieldName() {
  if (!contextMenuItem.value?.label) return;
  try {
    await navigator.clipboard.writeText(contextMenuItem.value.label);
    message("Copied field name", { type: "success" });
    closeContextMenu();
  } catch (err) {
    message("Failed to copy", { type: "error" });
  }
}

function deleteFile() {
  const item = contextMenuItem.value;
  if (!item || item.type !== "file") return;

  const displayName = item.label;
  const fullPath = item.fullPath;

  const paths = sqlHistory.path.split("|").filter(p => p !== fullPath);
  sqlHistory.path = paths.join("|");

  delete sqlHistory.dtypesByFile[displayName];

  message(`Deleted "${displayName}"`, { type: "success" });
  closeContextMenu();
}

function rightClick(event: Event, data: any) {
  const nodeData = { ...data };
  if (data.type === "file") {
    // 从 viewFileMeta 中找 fullPath
    const meta = viewFileMeta.value.find(m => m.displayName === data.label);
    if (meta) {
      nodeData.fullPath = meta.fullPath;
    }
  }
  openContextMenu(event as MouseEvent, nodeData);
}

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
            <el-button
              @click="selectViewFile()"
              :icon="FolderOpened"
              circle
              text
            />
          </el-tooltip>

          <el-splitter-panel>
            <el-scrollbar class="flex-1">
              <el-tree
                :data="fileTreeData"
                :props="defaultProps"
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
                @click="runInCurrentTab"
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
                @click="runInNewTab"
                :loading="isRunningNew"
                circle
                text
              >
                <template #icon>
                  <Icon icon="mdi:tab-plus" />
                </template>
              </el-button>
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
              @init="initializeEditor"
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
                    <el-select
                      v-model="data.writeFormat"
                      size="small"
                      style="width: 90px"
                    >
                      <el-option label="csv" value="csv" />
                      <el-option label="xlsx" value="xlsx" />
                      <el-option label="parquet" value="parquet" />
                      <el-option label="json" value="json" />
                      <el-option label="jsonl" value="jsonl" />
                    </el-select>
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

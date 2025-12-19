<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Event } from "@tauri-apps/api/event";
import { FolderOpened, ArrowRight, Files } from "@element-plus/icons-vue";
import { useDark } from "@pureadmin/utils";
import { useDynamicHeight } from "@/utils/utils";
import { message } from "@/utils/message";
import { viewOpenFile } from "@/utils/view";
import { mdRename, useMarkdown } from "@/utils/markdown";

const mode = ref("nil");
const modeOptions = [
  { label: "Nil", value: "nil" },
  { label: "Idx", value: "idx" }
];
const tableData = ref([]);
const [search, path] = [ref(""), ref("")];
const [currentRows, totalRows] = [ref(0), ref(0)];
const [dialog, isLoading] = [ref(false), ref(false)];
const { dynamicHeight } = useDynamicHeight(146);
const { mdShow } = useMarkdown(mdRename);
const { isDark } = useDark();
const filterTableData = computed(() =>
  tableData.value.filter(
    (data: any) =>
      !search.value ||
      data.col1.toLowerCase().includes(search.value.toLowerCase())
  )
);

listen("update-rows", (event: Event<number>) => {
  currentRows.value = event.payload;
});
listen("total-rows", (event: Event<number>) => {
  totalRows.value = event.payload;
});

async function selectFile() {
  tableData.value = [];
  search.value = "";
  path.value = "";
  totalRows.value = 0;

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

  try {
    const headers: string[] = await invoke("from_headers", {
      path: path.value
    });
    for (let i = 0; i < headers.length; i++) {
      const colData = {
        col1: headers[i],
        col2: headers[i % headers.length]
      };
      tableData.value.push(colData);
    }
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke rename
async function renameData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const headersStringArray = tableData.value.map((row: any) => row.col2);
    const headersString = headersStringArray.join(",");
    const rtime: string = await invoke("rename", {
      path: path.value,
      headers: headersString,
      mode: mode.value
    });
    message(`Rename done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}

async function headerEdit(row: any) {
  return row;
}
</script>

<template>
  <el-form class="page-container">
    <el-splitter>
      <el-splitter-panel size="140" :resizable="false">
        <div class="splitter-container">
          <el-tooltip content="Add data" effect="light" placement="right">
            <el-button @click="selectFile()" :icon="FolderOpened" text circle />
          </el-tooltip>

          <el-tooltip
            content="if nil, no progress bar"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle">
              <span
                v-for="item in modeOptions"
                :key="item.value"
                class="mode-item"
                :class="{
                  active: mode === item.value,
                  'active-dark': isDark && mode === item.value
                }"
                @click="mode = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <div style="margin-top: auto; display: flex; flex-direction: column">
            <el-progress
              v-if="totalRows !== 0 && isFinite(currentRows / totalRows)"
              :percentage="Math.round((currentRows / totalRows) * 100)"
              style="margin-bottom: 8px; margin-left: 8px"
            />
            <el-link @click="dialog = true">
              <span class="link-text">Rename</span>
            </el-link>
          </div>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-tooltip content="Run" effect="light" placement="right">
          <el-button
            @click="renameData()"
            :loading="isLoading"
            :icon="ArrowRight"
            circle
            text
          />
        </el-tooltip>

        <el-table
          :data="filterTableData"
          :height="dynamicHeight"
          show-overflow-tooltip
        >
          <el-table-column prop="col1" label="headers" />
          <el-table-column prop="col2" label="new headers">
            <template #default="{ row }">
              <el-input
                v-model="row.col2"
                placeholder="new header"
                @blur="headerEdit(row)"
              />
            </template>
          </el-table-column>
          <el-table-column>
            <template #header>
              <el-input
                v-model="search"
                size="small"
                placeholder="Type to search headers"
              />
            </template>
          </el-table-column>
        </el-table>

        <el-text>
          <el-icon style="margin-left: 8px">
            <Files />
          </el-icon>
          {{ path }}
        </el-text>
      </el-splitter-panel>
    </el-splitter>

    <el-dialog
      v-model="dialog"
      title="Rename - Rename the columns of a CSV"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

<style scoped>
.mode-toggle {
  width: 120px;
}
</style>

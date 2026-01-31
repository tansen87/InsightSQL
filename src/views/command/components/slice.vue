<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Files, SwitchButton } from "@element-plus/icons-vue";
import { useDark } from "@pureadmin/utils";
import { useDynamicHeight } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { mdSlice, useMarkdown } from "@/utils/markdown";
import { useFlexible, useQuoting, useSkiprows } from "@/store/modules/options";

const mode = ref("lines");
const modeOptions = [
  { label: "Lines", value: "lines" },
  { label: "Index", value: "index" }
];
const [path, start, end] = [ref(""), ref("1"), ref("10")];
const [isLoading, dialog] = [ref(false), ref(false)];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const { dynamicHeight } = useDynamicHeight(98);
const { mdShow } = useMarkdown(mdSlice);
const { isDark } = useDark();
const quotingStore = useQuoting();
const flexibleStore = useFlexible();
const skiprowsStore = useSkiprows();

async function selectFile() {
  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

  try {
    tableHeader.value = await mapHeaders(path.value, skiprowsStore.skiprows);
    const { columnView, dataView } = await toJson(
      path.value,
      skiprowsStore.skiprows
    );
    tableColumn.value = columnView;
    tableData.value = dataView;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke slice
async function sliceData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("slice", {
      path: path.value,
      quoting: quotingStore.quoting,
      flexible: flexibleStore.flexible,
      start: start.value,
      end: end.value,
      skiprows: skiprowsStore.skiprows,
      mode: mode.value
    });
    message(`Slice done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container">
    <el-splitter>
      <el-splitter-panel size="180" :resizable="false">
        <div class="splitter-container">
          <el-button @click="selectFile()" :icon="FolderOpened" text round>
            Open File
          </el-button>

          <div class="mode-toggle mt-2 mb-2 w-40">
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

          <el-tooltip
            content="The index of the row to slice from"
            effect="light"
            placement="right"
          >
            <el-input v-model="start" class="ml-2" style="width: 160px" />
          </el-tooltip>

          <el-tooltip
            content="The index of the row to slice to"
            effect="light"
            placement="right"
          >
            <el-input v-model="end" class="ml-2 mt-2" style="width: 160px" />
          </el-tooltip>

          <el-link @click="dialog = true" class="mt-auto">
            <span class="link-text">Slice</span>
          </el-link>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-button
          @click="sliceData()"
          :loading="isLoading"
          :icon="SwitchButton"
          text
          round
          >Run
        </el-button>

        <el-table
          :data="tableData"
          :height="dynamicHeight"
          show-overflow-tooltip
          tooltip-effect="light"
        >
          <el-table-column
            v-for="column in tableColumn"
            :prop="column.prop"
            :label="column.label"
            :key="column.prop"
          />
        </el-table>

        <el-text>
          <el-icon class="ml-2"><Files /></el-icon>
          {{ path }}
        </el-text>
      </el-splitter-panel>
    </el-splitter>

    <el-dialog
      v-model="dialog"
      title="Slice - Returns rows of a CSV file in the specified range"
      width="70%"
    >
      <el-scrollbar :height="dynamicHeight * 0.7">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

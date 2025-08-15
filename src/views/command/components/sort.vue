<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Refresh, Link } from "@element-plus/icons-vue";
import { useDynamicHeight } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { mdSort, useMarkdown } from "@/utils/markdown";

const mode = ref("Sort");
const [column, path] = [ref(""), ref("")];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const [isLoading, dialog, numeric, reverse] = [
  ref(false),
  ref(false),
  ref(false),
  ref(false)
];
const { dynamicHeight } = useDynamicHeight(153);
const { mdShow } = useMarkdown(mdSort);

async function selectFile() {
  column.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

  try {
    tableHeader.value = await mapHeaders(path.value, "0");
    const { columnView, dataView } = await toJson(path.value);
    tableColumn.value = columnView;
    tableData.value = dataView;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke sort
async function sortData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (column.value.length === 0 && mode.value !== "index") {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    let rtime: string;
    if (mode.value == "Sort") {
      rtime = await invoke("sort", {
        path: path.value,
        column: column.value,
        numeric: numeric.value,
        reverse: reverse.value
      });
    } else if (mode.value == "ExtSort") {
      rtime = await invoke("extsort", {
        path: path.value,
        column: column.value,
        reverse: reverse.value
      });
    } else if (mode.value == "Index") {
      rtime = await invoke("idx", {
        path: path.value
      });
    }
    message(`${mode.value} done, elapsed time: ${rtime} s`, {
      type: "success"
    });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <div class="page-container">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
        <el-select
          v-model="column"
          filterable
          style="width: 140px; margin-left: 8px"
          placeholder="Select column"
        >
          <el-option
            v-for="item in tableHeader"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
        <el-tooltip content="Sort, ExtSort or create index" effect="light">
          <el-select v-model="mode" style="margin-left: 8px; width: 90px">
            <el-option label="Sort" value="Sort" />
            <el-option label="ExtSort" value="ExtSort" />
            <el-option label="Index" value="Index" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="Numeric" effect="light">
          <el-select v-model="numeric" style="margin-left: 8px; width: 80px">
            <el-option label="true" :value="true" />
            <el-option label="false" :value="false" />
          </el-select>
        </el-tooltip>
        <el-tooltip
          content="Reverse (when set to false, sort from small to large)"
          effect="light"
        >
          <el-select v-model="reverse" style="margin-left: 8px; width: 80px">
            <el-option label="true" :value="true" />
            <el-option label="false" :value="false" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button @click="sortData()" :loading="isLoading" :icon="Refresh">
        {{ mode }}
      </el-button>
    </div>
    <el-table
      :data="tableData"
      :height="dynamicHeight"
      border
      empty-text=""
      style="margin-top: 10px; width: 100%"
      show-overflow-tooltip
    >
      <el-table-column
        v-for="column in tableColumn"
        :prop="column.prop"
        :label="column.label"
        :key="column.prop"
      />
    </el-table>
    <div class="custom-container1">
      <div class="custom-container2" />
      <el-link @click="dialog = true" :icon="Link">
        <el-tooltip :content="path" effect="light">
          <span>
            About
            <span style="color: skyblue; font-weight: bold">Sort</span>
          </span>
        </el-tooltip>
      </el-link>
    </div>
    <el-dialog
      v-model="dialog"
      title="Sort - Sorts CSV data lexicographically"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

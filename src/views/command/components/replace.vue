<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Refresh, Link } from "@element-plus/icons-vue";
import { useDynamicHeight } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { replaceContent, useMarkdown } from "@/utils/markdown";

const [isLoading, dialog] = [ref(false), ref(false)];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const [selectColumn, path, regexPattern, replacement] = [
  ref(""),
  ref(""),
  ref(""),
  ref("")
];
const { dynamicHeight } = useDynamicHeight(199);
const { compiledMarkdown } = useMarkdown(replaceContent);

async function selectFile() {
  selectColumn.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) {
    return;
  }

  try {
    tableHeader.value = await mapHeaders(path.value, "0");
    const { columnView, dataView } = await toJson(path.value);
    tableColumn.value = columnView;
    tableData.value = dataView;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke replace
async function replaceData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (selectColumn.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("replace", {
      path: path.value,
      selectColumn: selectColumn.value,
      regexPattern: regexPattern.value,
      replacement: replacement.value
    });
    message(`Replace done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <div class="page-container">
    <div class="custom-container1" style="margin-bottom: 12px">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
        <el-select
          v-model="selectColumn"
          filterable
          style="width: 150px; margin-left: 10px"
          placeholder="Select column"
        >
          <el-option
            v-for="item in tableHeader"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
        <el-input
          style="margin-left: 10px; width: 150px"
          placeholder="regex pattern"
          v-model="regexPattern"
        />
      </div>
      <el-button @click="replaceData()" :loading="isLoading" :icon="Refresh">
        Replace
      </el-button>
    </div>
    <el-input v-model="replacement" autosize placeholder="replacement" />
    <el-table
      :data="tableData"
      :height="dynamicHeight"
      border
      empty-text=""
      style="margin-top: 12px; width: 100%"
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
            <span style="color: skyblue; font-weight: bold">Replace</span>
          </span>
        </el-tooltip>
      </el-link>
    </div>
    <el-dialog
      v-model="dialog"
      title="Replace - Replace CSV data using a regex"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="compiledMarkdown" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

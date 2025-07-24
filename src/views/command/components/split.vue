<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { IceCreamRound, FolderOpened, Link } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { viewOpenFile, toJson } from "@/utils/view";
import { splitContent, useMarkdown } from "@/utils/markdown";
import { message } from "@/utils/message";

const [path, size, mode] = [ref(""), ref(1000000), ref("rows")];
const [tableColumn, tableData] = [ref([]), ref([])];
const [isLoading, dialog] = [ref(false), ref(false)];
const { dynamicHeight } = useDynamicHeight(155);
const { compiledMarkdown } = useMarkdown(splitContent);

async function selectFile() {
  path.value = "";
  tableColumn.value = [];
  tableData.value = [];

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

  try {
    const { columnView, dataView } = await toJson(path.value);
    tableColumn.value = columnView;
    tableData.value = dataView;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke split
async function splitData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("split", {
      path: path.value,
      size: size.value,
      mode: mode.value
    });
    message(`Split done, elapsed time: ${rtime} s`, { type: "success" });
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
        <el-tooltip content="Split rows" effect="light">
          <el-input-number
            v-model="size"
            controls-position="right"
            style="width: 172px; margin-left: 10px"
          />
        </el-tooltip>
        <el-tooltip content="Split mode" effect="light">
          <el-select v-model="mode" style="margin-left: 10px; width: 80px">
            <el-option label="Rows" value="rows" />
            <el-option label="Lines" value="lines" />
            <el-option label="Index" value="index" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button
        @click="splitData()"
        :loading="isLoading"
        :icon="IceCreamRound"
      >
        {{ mode }}-Split
      </el-button>
    </div>
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
      <div class="custom-container2">
        <el-tooltip :content="path" effect="light">
          <el-text>{{ shortFileName(path) }}</el-text>
        </el-tooltip>
      </div>
      <el-link @click="dialog = true" :icon="Link">
        <span>
          About
          <span style="color: skyblue; font-weight: bold">Split</span>
        </span>
      </el-link>
    </div>
    <el-dialog
      v-model="dialog"
      title="Split - Split one CSV file into many CSV files"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="compiledMarkdown" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

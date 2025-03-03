<script setup lang="ts">
import { ref, reactive } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import {
  FolderOpened,
  Connection,
  Link,
  Loading
} from "@element-plus/icons-vue";
import { useDynamicHeight } from "@/utils/utils";
import { catContent, useMarkdown } from "@/utils/markdown";
import { message, closeAllMessage } from "@/utils/message";
import { trimOpenFile } from "@/utils/view";

const [
  columns,
  selectedFiles,
  originalColumns,
  isLoading,
  backendCompleted,
  backendInfo,
  infoDialog
] = [ref(""), ref([]), ref([]), ref(false), ref(false), ref(""), ref(false)];
const data = reactive({
  filePath: "",
  mode: "Memory",
  skipRows: "0",
  useCols: ""
});
const { dynamicHeight } = useDynamicHeight(181);

async function selectFile() {
  columns.value = "";
  selectedFiles.value = [];
  originalColumns.value = [];
  backendInfo.value = "";
  backendCompleted.value = false;
  try {
    const trimFile = await trimOpenFile(true, "", ["*"], {
      includeStatus: false
    });
    data.filePath = trimFile.filePath;
    selectedFiles.value = trimFile.fileInfo;
    message("fetching headers...", {
      type: "info",
      duration: 0,
      icon: Loading
    });
    const headers: string[] = await invoke("inter_headers", {
      path: data.filePath,
      skipRows: data.skipRows
    });
    originalColumns.value = headers.map(header => ({
      label: header,
      value: header
    }));
    closeAllMessage();
    backendInfo.value = "headers fetched successfully";
    backendCompleted.value = true;
  } catch (err) {
    closeAllMessage();
    message(err.toString(), { type: "error" });
  }
}

// invoke concat
async function concatData() {
  if (data.filePath === "") {
    message("File not selected", { type: "warning" });
    return;
  }

  const outputPath = await save({
    title: "Export",
    defaultPath: `cat_${new Date().getTime()}`,
    filters: [
      { name: "CSV", extensions: ["csv"] },
      { name: "Excel", extensions: ["xlsx"] }
    ]
  });

  if (outputPath === "" || outputPath === null) {
    message("Save file not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const saveFileType = outputPath.split(".").pop();
    const useCols = Object.values(columns.value).join("|");
    const res: string = await invoke("concat", {
      filePath: data.filePath,
      outputPath: outputPath,
      fileType: saveFileType,
      mode: data.mode,
      skipRows: data.skipRows,
      useCols: useCols
    });
    backendInfo.value = `Cat done, elapsed time: ${res} s`;
    backendCompleted.value = true;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
  isLoading.value = false;
}

const { compiledMarkdown } = useMarkdown(catContent);
</script>

<template>
  <el-form class="page-container" :style="dynamicHeight">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
        <el-tooltip
          content="Polars memory or stream, Csv stream Cat"
          effect="light"
        >
          <el-select
            v-model="data.mode"
            style="margin-left: 10px; width: 100px"
          >
            <el-option label="Memory" value="memory" />
            <el-option label="Stream" value="stream" />
            <el-option label="Csv" value="csv" />
          </el-select>
        </el-tooltip>

        <el-tooltip content="skip rows" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 50px"
          />
        </el-tooltip>

        <el-button
          @click="concatData()"
          :loading="isLoading"
          :icon="Connection"
          style="margin-left: 10px"
        >
          Cat
        </el-button>
      </div>

      <el-link @click="infoDialog = true" :icon="Link">
        <span v-if="backendCompleted"> {{ backendInfo }} </span>
        <span v-else>
          About
          <span style="color: skyblue; font-weight: bold">Cat</span>
        </span>
      </el-link>
    </div>

    <el-select
      v-model="columns"
      multiple
      filterable
      style="margin-top: 12px; width: 100%"
      placeholder="Cat specific column (If column is empty, files have no common headers)"
    >
      <el-option
        v-for="item in originalColumns"
        :key="item.value"
        :label="item.label"
        :value="item.value"
      />
    </el-select>

    <el-table
      :data="selectedFiles"
      :height="dynamicHeight"
      empty-text=""
      style="width: 100%"
    >
      <el-table-column type="index" width="50" />
      <el-table-column prop="filename" />
    </el-table>

    <el-dialog
      v-model="infoDialog"
      title="Cat - Merge multiple CSV or Excel files into one CSV or xlsx file"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="compiledMarkdown" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

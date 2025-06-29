<script setup lang="ts">
import { ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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

const [mode, skipRows] = [ref("polars"), ref("0")];
const [columns, backendInfo, path] = [ref(""), ref(""), ref("")];
const [selectedFiles, originalColumns] = [ref([]), ref([])];
const [isLoading, backendCompleted, infoDialog] = [
  ref(false),
  ref(false),
  ref(false)
];
const { dynamicHeight } = useDynamicHeight(166);

listen("dupler_msg", (event: any) => {
  const duplerMsg: any = event.payload;
  const dupler = duplerMsg.split("|")[2];
  selectedFiles.value.forEach(file => {
    if (file.filename === duplerMsg.split("|")[0]) {
      file.infoMsg = dupler === "{}" ? "" : dupler;
    }
  });
});
listen("dupler_err", (event: any) => {
  const duplerErr: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === duplerErr.split("|")[0]) {
      file.infoMsg = duplerErr.split("|")[1];
    }
  });
});

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
    path.value = trimFile.filePath;
    selectedFiles.value = trimFile.fileInfo;
    message("fetching headers...", {
      type: "info",
      duration: 0,
      icon: Loading
    });
    const headers: string[] = await invoke("inter_headers", {
      path: path.value,
      skipRows: skipRows.value
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
  if (path.value === "") {
    message("File not selected", { type: "warning" });
    return;
  }
  if (mode.value === "duplicate") {
    message("find duplicate headers...", {
      type: "info",
      duration: 0,
      icon: Loading
    });
    await invoke("dupli_headers", {
      path: path.value,
      skipRows: skipRows.value
    });
    backendInfo.value = "find duplicate headers done";
    backendCompleted.value = true;
    closeAllMessage();
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
    const rtime: string = await invoke("concat", {
      filePath: path.value,
      outputPath: outputPath,
      fileType: saveFileType,
      mode: mode.value,
      skipRows: skipRows.value,
      useCols: useCols
    });
    backendInfo.value = `${mode.value} done, elapsed time: ${rtime} s`;
    backendCompleted.value = true;
  } catch (err) {
    message(err.toString(), { type: "error" });
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
          content="Polars memory Cat, Csv stream Cat, or find duplicate headers"
          effect="light"
        >
          <el-select v-model="mode" style="margin-left: 10px; width: 100px">
            <el-option label="Polars" value="polars" />
            <el-option label="Csv" value="csv" />
            <el-option label="Duplicate" value="duplicate" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="skip rows" effect="light">
          <el-input v-model="skipRows" style="margin-left: 10px; width: 50px" />
        </el-tooltip>
        <el-button
          @click="concatData()"
          :loading="isLoading"
          :icon="Connection"
          style="margin-left: 10px"
        >
          {{ mode }}
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
      show-overflow-tooltip
      style="width: 100%"
    >
      <el-table-column type="index" width="50" />
      <el-table-column
        prop="filename"
        label="file"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 60%"
      />
      <el-table-column
        prop="infoMsg"
        label="duplicate headers"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 40%"
      >
        <template #default="scope">
          {{ scope.row.infoMsg }}
        </template>
      </el-table-column>
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

<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  FolderOpened,
  Loading,
  Select,
  CloseBold,
  Delete
} from "@element-plus/icons-vue";
import { useDynamicHeight } from "@/utils/utils";
import { message } from "@/utils/message";
import { trimOpenFile } from "@/utils/view";

const [isLoading, selectedFiles] = [ref(false), ref([])];
const data = reactive({
  path: "",
  skipRows: "1",
  mode: "nil"
});
const { dynamicHeight } = useDynamicHeight(122);

listen("update-rows", (event: any) => {
  const [backFilename, rows] = event.payload.split("|");
  selectedFiles.value.forEach(file => {
    if (file.filename === backFilename) {
      file.currentRows = rows;
    }
  });
});
listen("total-rows", (event: any) => {
  const [backFilename, rows] = event.payload.split("|");
  selectedFiles.value.forEach(file => {
    if (file.filename === backFilename) {
      file.totalRows = rows;
    }
  });
});
listen("start_convert", (event: any) => {
  const startConvert: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert) {
      file.status = "loading";
    }
  });
});
listen("skip_err", (event: any) => {
  const beheadErr: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === beheadErr.split("|")[0]) {
      file.status = "error";
      file.errorMessage = beheadErr.split("|")[1];
    }
  });
  isLoading.value = false;
});
listen("skip_msg", (event: any) => {
  const skipMsg: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === skipMsg) {
      file.status = "completed";
    }
  });
});

async function selectFile() {
  selectedFiles.value = [];
  const trimFile = await trimOpenFile(true, "csv", ["*"], {
    includeStatus: true
  });
  data.path = trimFile.filePath;
  selectedFiles.value = trimFile.fileInfo;
  if (data.path === null) return;
}

// invoke skip
async function skipLines() {
  if (data.path === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const result: string = await invoke("skip", {
      path: data.path,
      mode: data.mode,
      skipRows: data.skipRows
    });
    message(`Skip done, elapsed time: ${result} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container" :style="dynamicHeight">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
        <el-tooltip content="skip rows" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 50px"
          />
        </el-tooltip>
        <el-tooltip
          content="Do you want to add a progress bar? If nil, do not add it"
          effect="light"
        >
          <el-select v-model="data.mode" style="margin-left: 10px; width: 70px">
            <el-option label="idx" value="idx" />
            <el-option label="std" value="std" />
            <el-option label="nil" value="nil" />
          </el-select>
        </el-tooltip>
        <el-button
          @click="skipLines()"
          :loading="isLoading"
          :icon="Delete"
          style="margin-left: 10px"
        >
          Skip
        </el-button>
      </div>

      <el-text>
        <span>Skip rows from CSV</span>
      </el-text>
    </div>

    <el-table
      :data="selectedFiles"
      :height="dynamicHeight"
      style="width: 100%"
      empty-text=""
    >
      <el-table-column type="index" width="50" />
      <el-table-column prop="filename" label="file" style="width: 80%" />
      <el-table-column prop="status" label="status" width="100">
        <template #default="scope">
          <ElIcon v-if="scope.row.status === 'loading'" class="is-loading">
            <Loading />
          </ElIcon>
          <ElIcon v-else-if="scope.row.status === 'completed'" color="#00CD66">
            <Select />
          </ElIcon>
          <ElIcon v-else-if="scope.row.status === 'error'" color="#FF0000">
            <CloseBold />
          </ElIcon>
        </template>
      </el-table-column>
      <el-table-column
        prop="errorMessage"
        label="Info"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 60%"
      >
        <template #default="scope">
          <span v-if="scope.row.status === 'error'">
            {{ scope.row.errorMessage }}
          </span>
          <el-progress
            v-if="
              scope.row.totalRows !== 0 &&
              isFinite(scope.row.currentRows / scope.row.totalRows)
            "
            :percentage="
              Math.round((scope.row.currentRows / scope.row.totalRows) * 100)
            "
          />
        </template>
      </el-table-column>
    </el-table>
  </el-form>
</template>

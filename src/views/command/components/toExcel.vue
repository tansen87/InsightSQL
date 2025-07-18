<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElIcon } from "element-plus";
import {
  FolderOpened,
  SwitchFilled,
  Loading,
  Select,
  CloseBold
} from "@element-plus/icons-vue";
import { useDynamicHeight, filterFileStatus } from "@/utils/utils";
import { message } from "@/utils/message";
import { trimOpenFile } from "@/utils/view";

const [chunkSize, mode] = [ref("700000"), ref("csv")];
const isLoading = ref(false);
const selectedFiles = ref([]);
const path = ref("");
const { dynamicHeight } = useDynamicHeight(123);

listen("start-to", (event: any) => {
  const startConvert: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert) {
      file.status = "loading";
    }
  });
});
listen("rows-err", (event: any) => {
  const csvRowsErr: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === csvRowsErr.split("|")[0]) {
      file.status = "error";
      file.errorMessage = csvRowsErr.split("|")[1];
    }
  });
});
listen("c2x-msg", (event: any) => {
  const c2xMsg: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === c2xMsg) {
      file.status = "completed";
    }
  });
});

async function selectFile() {
  selectedFiles.value = [];

  const result = await trimOpenFile(true, "csv", ["*"], {
    includeStatus: true
  });
  path.value = result.filePath;
  selectedFiles.value = result.fileInfo;
}

// invoke csv2xlsx
async function csvToxlsx() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("csv2xlsx", {
      path: path.value,
      mode: mode.value,
      chunkSize: chunkSize.value
    });
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
  <el-form class="page-container" :style="dynamicHeight">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
        <el-tooltip content="Polars or Csv engine" effect="light">
          <el-select v-model="mode" style="margin-left: 10px; width: 85px">
            <el-option label="Csv" value="csv" />
            <el-option label="Polars" value="polars" />
          </el-select>
        </el-tooltip>
        <el-tooltip
          content="Split every N rows into a sheet, only used for CSV engine"
          effect="light"
        >
          <el-input
            v-model="chunkSize"
            style="margin-left: 10px; width: 80px"
          />
        </el-tooltip>
        <el-button
          @click="csvToxlsx()"
          :loading="isLoading"
          :icon="SwitchFilled"
          style="margin-left: 10px"
        >
          {{ mode }}
        </el-button>
      </div>
      <el-text> Batch convert csv to xlsx </el-text>
    </div>
    <el-table
      :data="selectedFiles"
      :height="dynamicHeight"
      style="width: 100%"
      show-overflow-tooltip
      empty-text=""
    >
      <el-table-column type="index" width="50" />
      <el-table-column
        prop="filename"
        label="File"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 30%"
      />
      <el-table-column
        prop="status"
        label="Status"
        :filters="[
          { text: 'x', value: 'error' },
          { text: '√', value: 'completed' }
        ]"
        :filter-method="filterFileStatus"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 10%"
      >
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
        </template>
      </el-table-column>
    </el-table>
  </el-form>
</template>

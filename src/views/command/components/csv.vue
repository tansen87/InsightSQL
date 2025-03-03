<script setup lang="ts">
import { ref, reactive } from "vue";
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

const [isLoading, selectedFiles] = [ref(false), ref([])];
const data = reactive({
  path: "",
  skipRows: "0",
  mode: "Csv",
  chunkSize: "1000000"
});
const { dynamicHeight } = useDynamicHeight(134);

listen("start_convert", (event: any) => {
  const startConvert: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert) {
      file.status = "loading";
    }
  });
});
listen("rows_err", (event: any) => {
  const csvRowsErr: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === csvRowsErr.split("|")[0]) {
      file.status = "error";
      file.errorMessage = csvRowsErr.split("|")[1];
    }
  });
});
listen("c2x_msg", (event: any) => {
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
  data.path = result.filePath;
  selectedFiles.value = result.fileInfo;
}

// invoke switch_csv
async function csvToxlsx() {
  if (data.path === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const result: string = await invoke("switch_csv", {
      path: data.path,
      skipRows: data.skipRows,
      mode: data.mode,
      chunkSize: data.chunkSize
    });
    message(`Convert done, elapsed time: ${result} s`);
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
        <el-tooltip content="Polars or Csv engine" effect="light">
          <el-select v-model="data.mode" style="margin-left: 10px; width: 85px">
            <el-option label="Polars" value="polars" />
            <el-option label="Csv" value="csv" />
          </el-select>
        </el-tooltip>
        <el-tooltip
          content="Split every N rows into a sheet, only used for CSV engine"
          effect="light"
        >
          <el-input
            v-model="data.chunkSize"
            style="margin-left: 10px; width: 80px"
          />
        </el-tooltip>
        <el-tooltip content="skip rows" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 50px"
          />
        </el-tooltip>

        <el-button
          @click="csvToxlsx()"
          :loading="isLoading"
          :icon="SwitchFilled"
          style="margin-left: 10px"
        >
          Convert
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

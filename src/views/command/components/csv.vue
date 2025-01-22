<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification, ElIcon, TableColumnCtx } from "element-plus";
import {
  FolderOpened,
  SwitchFilled,
  Loading,
  Select,
  CloseBold
} from "@element-plus/icons-vue";
import { shortFileName, useDynamicFormHeight } from "@/utils/utils";

interface FileStatus {
  filename: string;
  status: string;
}

const isLoading = ref(false);
const progress = ref(0);
const selectedFiles = ref([]);
const tableRef = ref(null);
const customColors = [
  { color: "#98FB98", percentage: 20 },
  { color: "#7CFC00", percentage: 40 },
  { color: "#7FFF00", percentage: 60 },
  { color: "#ADFF2F", percentage: 80 },
  { color: "#9ACD32", percentage: 100 }
];
const filterFileStatus = (
  value: string,
  row: FileStatus,
  column: TableColumnCtx<FileStatus>
) => {
  const property = column["property"];
  return row[property] === value;
};
const data = reactive({
  filePath: "",
  fileFormats: ["*"],
  skipRows: "0",
  mode: "Polars"
});
const { formHeight } = useDynamicFormHeight(220);

listen("start_convert", (event: any) => {
  const startConvert: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (shortFileName(file.filename) === shortFileName(startConvert)) {
      file.status = "loading";
    }
  });
});
listen("c2x_progress", (event: any) => {
  const pgs: any = event.payload;
  progress.value = pgs;
});
listen("rows_err", (event: any) => {
  const csvRowsErr: any = event.payload;
  const basename = shortFileName(csvRowsErr.split("|")[0]);
  const errorDetails = csvRowsErr.split("|")[1];
  selectedFiles.value.forEach(file => {
    if (shortFileName(file.filename) === basename) {
      file.status = "error";
      file.errorMessage = errorDetails;
    }
  });
  isLoading.value = false;
});
listen("c2x_msg", (event: any) => {
  const c2xMsg: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (shortFileName(file.filename) === shortFileName(c2xMsg)) {
      file.status = "completed";
    }
  });
});

// open file
async function selectFile() {
  selectedFiles.value = [];
  isLoading.value = false;
  progress.value = 0;

  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "csv",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.filePath = selected.join("|").toString();
    const nonEmptyRows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = nonEmptyRows.map((file: any) => {
      return { filename: shortFileName(file), status: "" };
    });
  } else if (selected === null) {
    return;
  } else {
    data.filePath = selected;
  }
}

// convert csv to xlsx
async function csvToxlsx() {
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  isLoading.value = true;

  try {
    const result: string = await invoke("switch_csv", {
      path: data.filePath,
      skipRows: data.skipRows,
      mode: data.mode
    });

    if (result.startsWith("csv to xlsx failed:")) {
      throw result.toString();
    }

    ElNotification({
      message: `Convert done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 5000
    });
  } catch (err) {
    ElNotification({
      title: "Invoke switch_csv Error",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container" :style="formHeight">
    <el-form>
      <div
        style="
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
        "
      >
        <div style="display: flex; align-items: flex-start">
          <el-button
            type="default"
            @click="selectFile()"
            :icon="FolderOpened"
            plain
          >
            Open File
          </el-button>
          <el-tooltip
            content="Polars or Csv engine"
            placement="top"
            effect="light"
          >
            <el-select
              v-model="data.mode"
              style="margin-left: 12px; width: 85px"
            >
              <el-option label="Polars" value="polars" />
              <el-option label="Csv" value="csv" />
            </el-select>
          </el-tooltip>
          <el-tooltip content="skip rows" placement="top" effect="light">
            <el-input
              v-model="data.skipRows"
              style="margin-left: 12px; width: 80px"
              placeholder="skip rows"
            />
          </el-tooltip>
          <el-button
            type="default"
            @click="csvToxlsx()"
            :loading="isLoading"
            :icon="SwitchFilled"
            plain
            style="margin-left: 12px"
          >
            Convert
          </el-button>
        </div>
        <el-text type="default" size="large">
          Batch convert csv to xlsx
        </el-text>
      </div>
    </el-form>
    <el-table
      ref="tableRef"
      :data="selectedFiles"
      :height="formHeight"
      style="width: 100%"
      show-overflow-tooltip
    >
      <el-table-column type="index" width="50" />
      <el-table-column
        prop="filename"
        label="File"
        class-name="file-column"
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
        class-name="status-column"
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
        class-name="info-column"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 60%"
      >
        <template #default="scope">
          <span v-if="scope.row.status === 'error'">{{
            scope.row.errorMessage
          }}</span>
        </template>
      </el-table-column>
    </el-table>

    <el-progress
      v-if="isLoading"
      :percentage="progress"
      :color="customColors"
    />
  </el-form>
</template>

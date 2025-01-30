<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification, ElIcon } from "element-plus";
import {
  FolderOpened,
  SwitchFilled,
  Loading,
  Select,
  CloseBold
} from "@element-plus/icons-vue";
import {
  shortFileName,
  useDynamicFormHeight,
  customColors,
  filterFileStatus
} from "@/utils/utils";

const [isLoading, progress, selectedFiles] = [ref(false), ref(0), ref([])];
const data = reactive({
  path: "",
  fileFormats: ["*"],
  skipRows: "0",
  mode: "Polars"
});
const { formHeight } = useDynamicFormHeight(134);

listen("start_convert", (event: any) => {
  const startConvert: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert) {
      file.status = "loading";
    }
  });
});
listen("c2x_progress", (event: any) => {
  const pgs: any = event.payload;
  progress.value = pgs;
});
listen("rows_err", (event: any) => {
  const csvRowsErr: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === csvRowsErr.split("|")[0]) {
      file.status = "error";
      file.errorMessage = csvRowsErr.split("|")[1];
    }
  });
  isLoading.value = false;
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
    data.path = selected.join("|").toString();
    const nonEmptyRows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = nonEmptyRows.map((file: any) => {
      return { filename: shortFileName(file), status: "" };
    });
  } else if (selected === null) {
    return;
  } else {
    data.path = selected;
  }
}

// invoke switch_csv
async function csvToxlsx() {
  if (data.path === "") {
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
      path: data.path,
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
      title: "Csv to xlsx failed",
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
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened" plain>
          Open File
        </el-button>

        <el-tooltip
          content="Polars or Csv engine"
          placement="top"
          effect="light"
        >
          <el-select v-model="data.mode" style="margin-left: 10px; width: 85px">
            <el-option label="Polars" value="polars" />
            <el-option label="Csv" value="csv" />
          </el-select>
        </el-tooltip>

        <el-tooltip content="skip rows" placement="top" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 80px"
            placeholder="skip rows"
          />
        </el-tooltip>

        <el-button
          @click="csvToxlsx()"
          :loading="isLoading"
          :icon="SwitchFilled"
          plain
          style="margin-left: 10px"
        >
          Convert
        </el-button>
      </div>

      <el-text> Batch convert csv to xlsx </el-text>
    </div>

    <el-table
      :data="selectedFiles"
      :height="formHeight"
      style="width: 100%"
      show-overflow-tooltip
      empty-text=""
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
          <span v-if="scope.row.status === 'error'">
            {{ scope.row.errorMessage }}
          </span>
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

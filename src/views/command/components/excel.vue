<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { TableColumnCtx } from "element-plus";
import { ElNotification, ElIcon } from "element-plus";
import {
  CloseBold,
  Select,
  FolderOpened,
  SwitchFilled,
  Loading
} from "@element-plus/icons-vue";

interface FileStatus {
  filename: string;
  status: string;
}
const selectedFiles = ref([]);
const isLoading = ref(false);
const progress = ref(0);
const runtime = ref(0.0);
const tableRef = ref(null);
const windowHeight = ref(window.innerHeight);
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
  fileFormats: ["xlsx", "xls", "xlsb", "xlsm", "xlam", "xla", "ods"],
  skipRows: "0"
});

const formHeight = computed(() => {
  const height = 240;
  return windowHeight.value - height;
});

const updateWindowHeight = () => {
  windowHeight.value = window.innerHeight;
};

onMounted(() => {
  window.addEventListener("resize", updateWindowHeight);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", updateWindowHeight);
});

listen("start_convert", (event: any) => {
  const startConvert: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert) {
      file.status = "loading";
    }
  });
});
listen("runtime", (event: any) => {
  runtime.value = event.payload;
});
listen("row_count_err", (event: any) => {
  const excelRowCountErr: any = event.payload;
  ElNotification({
    title: "Rows Count",
    message: excelRowCountErr,
    position: "bottom-right",
    type: "warning",
    duration: 10000
  });
  selectedFiles.value.forEach(file => {
    if (file.filename === excelRowCountErr.split("|")[0]) {
      file.status = "error";
    }
  });
  isLoading.value = false;
});
listen("e2c_msg", (event: any) => {
  const e2cMsg: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === e2cMsg) {
      file.status = "completed";
    }
  });
});
listen("e2c_progress", (event: any) => {
  const pgs: any = event.payload;
  progress.value = pgs;
});
listen("e2c_err", (event: any) => {
  const writeCsvErr = event.payload;
  ElNotification({
    title: "Switch_excel Error",
    message: writeCsvErr,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});

// open file
async function selectFile() {
  isLoading.value = false;
  selectedFiles.value = [];
  progress.value = 0;

  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "Excel",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.filePath = selected.join("|").toString();
    const nonEmptyRows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = nonEmptyRows.map((file: any) => {
      return { filename: file, status: "" };
    });
  } else if (selected === null) {
    ElNotification({
      title: "File not found",
      message: "未选择Excel文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  } else {
    data.filePath = selected;
  }
}

// convert excel to csv
async function excelToCsv() {
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择Excel文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  if (data.filePath !== "") {
    isLoading.value = true;

    await invoke("switch_excel", {
      path: data.filePath,
      skipRows: data.skipRows
    });

    ElNotification({
      message: "Convert done, elapsed time: " + runtime.value,
      position: "bottom-right",
      type: "success",
      duration: 5000
    });
    isLoading.value = false;
  }
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
          position: sticky;
        "
      >
        <el-form-item>
          <el-button
            type="primary"
            @click="selectFile()"
            :icon="FolderOpened"
            plain
          >
            Open File
          </el-button>
          <el-tooltip content="with header row" placement="top" effect="light">
            <el-input
              v-model="data.skipRows"
              style="margin-left: 16px; margin-right: 16px; width: 80px"
              placeholder="skip rows"
            />
          </el-tooltip>
          <el-button
            type="success"
            @click="excelToCsv()"
            :loading="isLoading"
            :icon="SwitchFilled"
            plain
          >
            Convert
          </el-button>
        </el-form-item>
        <el-text type="primary" size="large">
          <el-icon> <SwitchFilled /> </el-icon>
          Exports Excel to a csv file
        </el-text>
      </div>
    </el-form>
    <el-table
      ref="tableRef"
      :data="selectedFiles"
      :height="formHeight"
      style="width: 100%"
    >
      <el-table-column prop="filename" label="file" style="width: 80%" />
      <el-table-column
        prop="status"
        label="status"
        :filters="[
          { text: 'x', value: 'error' },
          { text: '√', value: 'completed' }
        ]"
        :filter-method="filterFileStatus"
        width="100"
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
    </el-table>
    <el-progress
      v-if="isLoading"
      :percentage="progress"
      :color="customColors"
    />
  </el-form>
</template>

<style lang="scss">
.page-container {
  margin-bottom: 20px;
  padding: 20px;
  border-radius: 10px;
  background-color: #fff;
}
</style>

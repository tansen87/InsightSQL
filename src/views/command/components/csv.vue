<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from "vue";
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

interface FileStatus {
  filename: string;
  status: string;
}

const isLoading = ref(false);
const progress = ref(0);
const selectedFiles = ref([]);
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
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"],
  skipRows: "0"
});

const formHeight = computed(() => {
  const height = 225;
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
listen("c2x_progress", (event: any) => {
  const pgs: any = event.payload;
  progress.value = pgs;
});
listen("rows_err", (event: any) => {
  const csvRowsErr: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename.split("\\").pop() === csvRowsErr.split("|")[0]) {
      file.status = "error";
    }
  });
  ElNotification({
    title: "Write Error",
    message: csvRowsErr,
    position: "bottom-right",
    type: "error",
    duration: 10000
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
      return { filename: file, status: "" };
    });
  } else if (selected === null) {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
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

  if (data.filePath !== "") {
    isLoading.value = true;

    try {
      const result: string = await invoke("switch_csv", {
        path: data.filePath,
        skipRows: data.skipRows
      });

      if (result.startsWith("csv to xlsx failed:")) {
        throw result.toString();
      }
      ElNotification({
        message: "Convert done, elapsed time: " + result + " s",
        position: "bottom-right",
        type: "success",
        duration: 5000
      });
      isLoading.value = false;
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
              style="margin-left: 16px; width: 80px"
              placeholder="skip rows"
            />
          </el-tooltip>
          <el-button
            type="success"
            @click="csvToxlsx()"
            :loading="isLoading"
            :icon="SwitchFilled"
            plain
            style="margin-left: 16px"
          >
            Convert
          </el-button>
        </div>
        <el-text type="primary" size="large">
          Exports csv to a xlsx file
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

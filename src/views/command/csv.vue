<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { ElMessage, ElIcon, TableColumnCtx } from "element-plus";
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
  sep: ","
});

listen("start_convert", (event: any) => {
  const startConvert: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert.split("|")[0]) {
      file.status = "loading";
    }
  });
});
listen("c2x_err", (event: any) => {
  const error: any = "c2x_err: " + event.payload;
  ElMessage({
    showClose: true,
    message: error,
    type: "error",
    duration: 0
  });
  isLoading.value = false;
});
listen("c2x_progress", (event: any) => {
  const pgs: any = event.payload;
  progress.value = pgs;
});
listen("read_err", (event: any) => {
  const error: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === error.split("|")[0]) {
      file.status = "error";
    }
  });
  ElMessage({
    showClose: true,
    message: "read_err: " + error,
    type: "error",
    duration: 0
  });
  isLoading.value = false;
});
listen("rows_err", (event: any) => {
  const error: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename.split("\\").pop() === error.split("|")[0]) {
      file.status = "error";
    }
  });
  ElMessage({
    showClose: true,
    message: "rows_err: " + error,
    type: "error",
    duration: 0
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

// convert csv to xlsx
async function csvToxlsx() {
  if (data.filePath == "") {
    ElMessage.warning("未选择csv文件");
    return;
  }

  if (data.filePath != "") {
    ElMessage.info("Running...");
    isLoading.value = true;
    await invoke("switch_csv", {
      path: data.filePath,
      sep: data.sep
    });
    ElMessage.success("convert done.");
  }
}

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
    ElMessage.warning("未选择文件");
    return;
  } else {
    data.filePath = selected;
  }
}
</script>

<template>
  <div class="page-container">
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
          <el-select v-model="data.sep" style="margin-left: 16px; width: 100px">
            <el-option label="," value="," />
            <el-option label="|" value="|" />
            <el-option label="\t" value="\t" />
            <el-option label=";" value=";" />
          </el-select>
          <el-button
            type="success"
            @click="csvToxlsx()"
            :icon="SwitchFilled"
            plain
            style="margin-left: 16px"
          >
            Convert
          </el-button>
        </div>
        <el-text type="primary" size="large">
          <el-icon> <SwitchFilled /> </el-icon>
          Exports csv to a xlsx file
        </el-text>
      </div>
    </el-form>
    <el-table :data="selectedFiles" height="760" style="width: 100%">
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
  </div>
</template>

<style lang="scss">
.page-container {
  margin-bottom: 20px;
  padding: 20px;
  border-radius: 10px;
  background-color: #fff;
}
</style>

<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { FolderOpened, Connection } from "@element-plus/icons-vue";

const selectedFiles = ref([]);
const isLoading = ref(false);
const data = reactive({
  filePath: "",
  fileFormats: [
    "csv",
    "txt",
    "tsv",
    "spext",
    "dat",
    "parquet",
    "xls",
    "xlsx",
    "xlsm",
    "xlsb",
    "ods"
  ],
  sep: ","
});

listen("cat_err", (event: any) => {
  ElNotification({
    title: "Concat Error",
    message: event.payload,
    position: "bottom-right",
    type: "error",
    duration: 0
  });
  isLoading.value = false;
});

// open file
async function selectFile() {
  selectedFiles.value = [];
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.filePath = selected.join("|").toString();
    console.log(data.filePath);
    const nonEmptyRows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = nonEmptyRows.map((file: any) => {
      return { filename: file };
    });
  } else if (selected === null) {
    ElNotification({
      title: "File not found",
      message: "未选择文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  } else {
    data.filePath = selected;
  }
}

// data concat
async function concatData() {
  if (data.filePath == "") {
    ElNotification({
      title: "File not found",
      message: "未选择文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }
  if (data.filePath != "") {
    isLoading.value = true;
    await invoke("concat", {
      filePath: data.filePath,
      sep: data.sep
    });
    isLoading.value = false;
    ElNotification({
      title: "",
      message: "Concat done.",
      position: "bottom-right",
      type: "success",
      duration: 0
    });
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
            @click="concatData()"
            :loading="isLoading"
            :icon="Connection"
            plain
            style="margin-left: 16px"
          >
            Concat
          </el-button>
        </div>
        <el-text type="primary" size="large">
          <el-icon> <Connection /> </el-icon>
          Concatenate CSV and Excel files by column
        </el-text>
      </div>
    </el-form>
    <el-table :data="selectedFiles" height="700" style="width: 100%">
      <el-table-column prop="filename" />
    </el-table>
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

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { FolderOpened, Connection } from "@element-plus/icons-vue";

const selectedFiles = ref([]);
const isLoading = ref(false);
const runtime = ref(0.0);
const tableRef = ref(null);
const windowHeight = ref(window.innerHeight);
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
  sep: ",",
  memory: true
});

const formHeight = computed(() => {
  const height = 205;
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

listen("runtime", (event: any) => {
  runtime.value = event.payload;
});
listen("cat_err", (event: any) => {
  const catErr = event.payload;
  ElNotification({
    title: "Concat Error",
    message: catErr,
    position: "bottom-right",
    type: "error",
    duration: 10000
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
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择文件",
      position: "bottom-right",
      type: "warning"
    });
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
    ElNotification({
      title: "File not found",
      message: "未选择保存文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  if (data.filePath !== "" && outputPath !== null) {
    isLoading.value = true;

    await invoke("concat", {
      filePath: data.filePath,
      sep: data.sep,
      outputPath: outputPath,
      memory: data.memory
    });

    isLoading.value = false;
    ElNotification({
      message: "Cat done, elapsed time: " + runtime.value,
      position: "bottom-right",
      type: "success",
      duration: 0
    });
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
          <el-select v-model="data.sep" style="margin-left: 16px; width: 100px">
            <el-option label="," value="," />
            <el-option label="|" value="|" />
            <el-option label="\t" value="\t" />
            <el-option label=";" value=";" />
          </el-select>
          <el-select
            v-model="data.memory"
            style="margin-left: 16px; width: 100px"
          >
            <el-option label="Memory" :value="true" />
            <el-option label="Stream" :value="false" />
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
          Cat CSV and Excel files
        </el-text>
      </div>
    </el-form>
    <el-table
      ref="tableRef"
      :data="selectedFiles"
      :height="formHeight"
      style="width: 100%"
    >
      <el-table-column prop="filename" />
    </el-table>
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

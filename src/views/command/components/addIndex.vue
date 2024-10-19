<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { IceCreamRound, FolderOpened } from "@element-plus/icons-vue";

const isLoading = ref(false);
const columns = ref([]);
const isPath = ref(false);
const runtime = ref(0.0);
const tableData = ref([]);
const tableRef = ref(null);
const windowHeight = ref(window.innerHeight);
const data = reactive({
  filePath: "",
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"]
});

const formHeight = computed(() => {
  const height = 220;
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
listen("show", (event: any) => {
  const df: any = event.payload;
  const jsonData = JSON.parse(df);
  const isJsonArray = Array.isArray(jsonData);
  const data = isJsonArray ? jsonData : [jsonData];
  columns.value = Object.keys(data[0]).map(key => ({
    name: key,
    label: key,
    prop: key
  }));
  tableData.value = data;
});
listen("index_err", (event: any) => {
  const indexErr = event.payload;
  ElNotification({
    title: "Index Error",
    message: indexErr,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});

async function selectFile() {
  isLoading.value = false;
  isPath.value = false;
  columns.value = [];
  tableData.value = [];

  const selected = await open({
    multiple: false,
    filters: [
      {
        name: "csv",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.filePath = selected.toString();
  } else if (selected === null) {
    return;
  } else {
    data.filePath = selected;
  }
  isPath.value = true;

  await invoke("query", {
    path: data.filePath,
    sqlsrc: "select * from _t_1 limit 5",
    write: false,
    writeFormat: "csv",
    lowMemory: false
  });
}

// add index
async function addIndex() {
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

    await invoke("index", {
      filePath: data.filePath
    });

    isLoading.value = false;
    ElNotification({
      message: "Index done, elapsed time: " + runtime.value,
      position: "bottom-right",
      type: "success",
      duration: 5000
    });
  }
}
</script>

<template>
  <div class="page-container">
    <div
      style="
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        position: sticky;
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
        <el-button
          type="success"
          @click="addIndex()"
          :loading="isLoading"
          :icon="IceCreamRound"
          plain
          style="margin-left: 16px"
        >
          Index
        </el-button>
      </div>
      <el-text type="primary" size="large">
        <el-icon> <IceCreamRound /> </el-icon>
        <span v-if="isPath">{{ data.filePath }}</span>
        <span v-else>Add an index for a CSV</span>
      </el-text>
    </div>
    <div
      style="
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        position: sticky;
      "
    >
      <el-table
        ref="tableRef"
        :data="tableData"
        :height="formHeight"
        border
        style="margin-top: 15px; width: 100%"
      >
        <el-table-column
          v-for="column in columns"
          :prop="column.prop"
          :label="column.label"
          :key="column.prop"
        />
      </el-table>
    </div>
  </div>
</template>

<style>
.page-container {
  margin-bottom: 20px;
  padding: 20px;
  border-radius: 10px;
  background-color: #fff;
}
.is-loading {
  font-size: 20px;
}
</style>

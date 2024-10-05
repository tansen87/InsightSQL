<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { Cpu, FolderOpened } from "@element-plus/icons-vue";

const isLoading = ref(false);
const isPath = ref(false);
const fillRows = ref(0);
const runtime = ref(0.0);
const columns = ref("");
const originalColumns = ref([]);
const data = reactive({
  filePath: "",
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"],
  sep: ",",
  value: "0"
});

listen("runtime", (event: any) => {
  runtime.value = event.payload;
});
listen("fill_rows", (event: any) => {
  const count: any = event.payload;
  fillRows.value = count;
});
listen("fill_err", (event: any) => {
  const fillErr = event.payload;
  ElNotification({
    title: "Fill Error",
    message: fillErr,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});

// open file
async function selectFile() {
  isLoading.value = false;
  isPath.value = false;

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

  const header: any = await invoke("get_fill_headers", {
    path: data.filePath,
    sep: data.sep
  });
  originalColumns.value = header;
}

// fill data
async function fillData() {
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }
  if (columns.value.length === 0) {
    ElNotification({
      title: "Column not defined",
      message: "未选择columns",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  const cols = Object.values(columns.value).join("|");

  if (data.filePath !== "") {
    isLoading.value = true;

    await invoke("fill", {
      path: data.filePath,
      sep: data.sep,
      columns: cols,
      values: data.value
    });

    isLoading.value = false;
    ElNotification({
      message:
        "Fill done, fill rows: " +
        fillRows.value +
        " lines, elapsed time: " +
        runtime.value,
      position: "bottom-right",
      type: "success",
      duration: 10000
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
        <el-select v-model="data.sep" style="margin-left: 16px; width: 100px">
          <el-option label="," value="," />
          <el-option label="|" value="|" />
          <el-option label="\t" value="\t" />
          <el-option label=";" value=";" />
        </el-select>
      </div>

      <el-text type="primary" size="large">
        <el-icon> <Cpu /> </el-icon>
        <span v-if="isPath">{{ data.filePath }}</span>
        <span v-else>Fill empty fields in selected columns of a CSV</span>
      </el-text>
    </div>
    <p />
    <el-select
      v-model="columns"
      multiple
      filterable
      style="margin-top: 15px; width: 100%"
      placeholder="please choose column"
    >
      <el-option
        v-for="item in originalColumns"
        :key="item.value"
        :label="item.label"
        :value="item.value"
      />
    </el-select>
    <div
      style="
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        position: sticky;
      "
    >
      <div style="margin-top: 15px; display: flex; align-items: flex-start">
        <el-input
          v-model="data.value"
          style="width: 120px; margin-right: 16px"
          clearable
        />
        <el-button
          type="success"
          @click="fillData()"
          :loading="isLoading"
          :icon="Cpu"
          plain
        >
          Fill
        </el-button>
      </div>
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
</style>

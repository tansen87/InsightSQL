<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { ElMessage } from "element-plus";
import {
  SuccessFilled,
  Loading,
  Cpu,
  FolderOpened
} from "@element-plus/icons-vue";

const isLoading = ref(false);
const isFinish = ref(false);
const isWrite = ref(false);
const isPath = ref(false);
const fillRows = ref(0);
const columns = ref("");
const originalColumns = ref([]);
const data = reactive({
  filePath: "",
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"],
  sep: ",",
  value: "0"
});

listen("fill_rows", (event: any) => {
  const count: any = event.payload;
  fillRows.value = count;
});
listen("fill_err", (event: any) => {
  const wtrMsg = event.payload;
  ElMessage.error("fill_err: " + wtrMsg);
  isLoading.value = false;
});

async function selectFile() {
  isFinish.value = false;
  isLoading.value = false;
  isWrite.value = false;
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
  if (data.filePath == "") {
    ElMessage.warning("未选择csv文件");
    return;
  }
  if (columns.value.length === 0) {
    ElMessage.warning("未选择columns");
    return;
  }

  const cols = Object.values(columns.value).join("|");
  console.log(cols);

  if (data.filePath != "") {
    ElMessage.info("Running...");
    isLoading.value = true;
    isFinish.value = false;
    await invoke("fill", {
      path: data.filePath,
      sep: data.sep,
      columns: cols,
      values: data.value
    });
    isLoading.value = false;
    isFinish.value = true;
    isWrite.value = true;
    ElMessage.success("fill done.");
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
        <el-button type="success" @click="fillData()" :icon="Cpu" plain>
          Fill
        </el-button>
      </div>
      <el-form-item style="margin-top: 15px">
        <el-icon v-if="isLoading" color="#FF8C00" class="is-loading">
          <Loading />
        </el-icon>
        <el-icon v-if="isFinish" color="#32CD32"> <SuccessFilled /> </el-icon>
        <el-text v-if="isWrite" class="mx-1">
          fill rows: {{ fillRows }} lines
        </el-text>
      </el-form-item>
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

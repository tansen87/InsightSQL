<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { Cpu, FolderOpened } from "@element-plus/icons-vue";

const isLoading = ref(false);
const isPath = ref(false);
const columns = ref("");
const originalColumns = ref([]);
const data = reactive({
  filePath: "",
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"],
  value: "0"
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

  try {
    const headers: any = await invoke("get_fill_headers", {
      path: data.filePath
    });
    if (JSON.stringify(headers).startsWith("get header error:")) {
      throw JSON.stringify(headers).toString();
    }
    originalColumns.value = headers;
  } catch (err) {
    ElNotification({
      title: "Open file error",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
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

  isLoading.value = true;

  try {
    const result: string = await invoke("fill", {
      path: data.filePath,
      columns: cols,
      values: data.value
    });

    if (JSON.stringify(result).startsWith("fill failed:")) {
      throw JSON.stringify(result).toString();
    }

    isLoading.value = false;
    ElNotification({
      message: `Fill done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
  } catch (err) {
    ElNotification({
      title: "Invoke Fill Error",
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
      </div>

      <el-text type="primary" size="large">
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
      placeholder="please choose columns"
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
        <el-tooltip
          content="The value of fill"
          placement="bottom"
          effect="light"
        >
          <el-input
            v-model="data.value"
            style="width: 120px; margin-right: 16px"
            clearable
          />
        </el-tooltip>
      </div>
      <el-button
        style="margin-top: 15px"
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
</template>

<style>
.page-container {
  margin-bottom: 20px;
  padding: 20px;
  border-radius: 10px;
  background-color: #fff;
}
</style>

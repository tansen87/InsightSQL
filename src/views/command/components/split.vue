<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { IceCreamRound, FolderOpened } from "@element-plus/icons-vue";

const isLoading = ref(false);
const isPath = ref(false);
const runtime = ref(0.0);
const data = reactive({
  filePath: "",
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"],
  size: 1000000
});

listen("runtime", (event: any) => {
  runtime.value = event.payload;
});
listen("split_err", (event: any) => {
  const splitErr = event.payload;
  ElNotification({
    title: "Split Error",
    message: splitErr,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});

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
}

// split data
async function splitData() {
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

    await invoke("split", {
      filePath: data.filePath,
      size: data.size
    });

    isLoading.value = false;
    ElNotification({
      message: "Split done, elapsed time: " + runtime.value,
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
      </div>
      <el-text type="primary" size="large">
        <el-icon> <IceCreamRound /> </el-icon>
        <span v-if="isPath">{{ data.filePath }}</span>
        <span v-else>Split one CSV file into many CSV files</span>
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
      <div style="display: flex; align-items: flex-start; margin-top: 10px">
        <el-tooltip content="Split rows" placement="bottom" effect="light">
          <el-input-number
            v-model="data.size"
            controls-position="right"
            style="width: 150px"
          />
        </el-tooltip>
      </div>
      <el-button
        style="margin-top: 10px"
        type="success"
        @click="splitData()"
        :loading="isLoading"
        :icon="IceCreamRound"
        plain
      >
        Split
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
.is-loading {
  font-size: 20px;
}
</style>

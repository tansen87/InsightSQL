<script setup lang="ts">
import { ref, reactive } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { appConfigDir } from "@tauri-apps/api/path";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { FolderOpened, Cherry } from "@element-plus/icons-vue";

const isPath = ref(false);
const isLoading = ref(false);
const data = reactive({
  folderPath: ""
});

listen("traverse_err", (event: any) => {
  const traverseErr: any = event.payload;
  ElNotification({
    title: "Traverse Error",
    message: traverseErr,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});
listen("traverse_write_err", (event: any) => {
  const traverseWriteErr: any = event.payload;
  ElNotification({
    title: "Write Error",
    message: traverseWriteErr,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});

async function selectFolder() {
  isPath.value = false;
  isLoading.value = false;

  const selected = await open({
    directory: true,
    multiple: false,
    defaultPath: await appConfigDir()
  });
  if (Array.isArray(selected)) {
    data.folderPath = selected.toString();
  } else if (selected === null) {
    return;
  } else {
    data.folderPath = selected;
  }

  isPath.value = true;
}

// traverse directory
async function traverseDirectory() {
  if (data.folderPath === "") {
    ElNotification({
      title: "Folder not found",
      message: "未选择文件夹",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  const output = await save({
    title: "Export",
    defaultPath: `filename_${new Date().getTime()}.xlsx`,
    filters: [{ name: "Excel", extensions: ["xlsx"] }]
  });
  if (output === "" || output === null) {
    ElNotification({
      title: "File not found",
      message: "未选择保存文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  isLoading.value = true;

  await invoke("traverse", {
    folderPath: data.folderPath,
    output: output
  });

  isLoading.value = false;
  ElNotification({
    message: "Traverse done.",
    position: "bottom-right",
    type: "success",
    duration: 10000
  });
}
</script>

<template>
  <el-form class="page-container">
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
          @click="selectFolder()"
          :icon="FolderOpened"
          plain
        >
          Open Folder
        </el-button>
        <el-button
          type="success"
          @click="traverseDirectory()"
          :loading="isLoading"
          :icon="Cherry"
          plain
        >
          Traverse
        </el-button>
      </div>
      <el-text type="primary" size="large">
        <el-icon> <Cherry /> </el-icon>
        <span v-if="isPath">{{ data.folderPath }}</span>
        <span v-else>Traverse the directory to obtain filenames</span>
      </el-text>
    </div>
  </el-form>
</template>

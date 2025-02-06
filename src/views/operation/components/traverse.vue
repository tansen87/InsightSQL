<script setup lang="ts">
import { ref, reactive } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { appConfigDir } from "@tauri-apps/api/path";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, SwitchFilled } from "@element-plus/icons-vue";
import { message } from "@/utils/message";

const [isLoading, isPath] = [ref(false), ref(false)];
const data = reactive({
  folderPath: ""
});

async function selectFolder() {
  isPath.value = false;

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

// invoke traverse
async function traverseDirectory() {
  if (data.folderPath === "") {
    message("No folder selected", { type: "warning" });
    return;
  }

  const output = await save({
    title: "Export",
    defaultPath: `filename_${new Date().getTime()}.xlsx`,
    filters: [{ name: "Excel", extensions: ["xlsx"] }]
  });
  if (output === "" || output === null) {
    message("No file saved selected", { type: "warning" });
    return;
  }

  isLoading.value = true;

  try {
    const result: string = await invoke("traverse", {
      folderPath: data.folderPath,
      output: output
    });

    message(`${result}`, { duration: 5000 });
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFolder()" :icon="FolderOpened">
          Open Folder
        </el-button>
        <el-button
          @click="traverseDirectory()"
          :loading="isLoading"
          :icon="SwitchFilled"
        >
          Traverse
        </el-button>
      </div>
      <el-text>
        <span v-if="isPath">{{ data.folderPath }}</span>
        <span v-else>Traverse the directory to obtain filenames</span>
      </el-text>
    </div>
  </el-form>
</template>

<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, SwitchFilled } from "@element-plus/icons-vue";
import { useDynamicHeight } from "@/utils/utils";
import { message } from "@/utils/message";
import { trimOpenFile } from "@/utils/view";

const [selectedFiles, isLoading] = [ref([]), ref(false)];
const data = reactive({
  path: "",
  sep: "|"
});
const { dynamicHeight } = useDynamicHeight(134);

async function selectFile() {
  selectedFiles.value = [];

  const result = await trimOpenFile(true, "Access", ["mdb", "accdb"], {
    includeStatus: false
  });
  data.path = result.filePath;
  selectedFiles.value = result.fileInfo;
}

// invoke access
async function accessData() {
  if (data.path === "") {
    message("File not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const result: string = await invoke("access", {
      path: data.path,
      sep: data.sep
    });
    message(`Convert done, elapsed time: ${result} s`);
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container" :style="dynamicHeight">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
        <el-select v-model="data.sep" style="margin-left: 10px; width: 100px">
          <el-option label="," value="," />
          <el-option label="|" value="|" />
          <el-option label="\t" value="\t" />
          <el-option label=";" value=";" />
        </el-select>

        <el-button
          @click="accessData()"
          :loading="isLoading"
          :icon="SwitchFilled"
          style="margin-left: 10px"
        >
          Convert
        </el-button>
      </div>

      <el-text> Convert Access Database to CSV </el-text>
    </div>

    <el-table
      :data="selectedFiles"
      :height="dynamicHeight"
      style="width: 100%"
      empty-text=""
    >
      <el-table-column type="index" width="50" />
      <el-table-column prop="filename" />
    </el-table>
  </el-form>
</template>

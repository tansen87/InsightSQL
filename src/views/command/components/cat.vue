<script setup lang="ts">
import { ref, reactive } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { FolderOpened, Connection } from "@element-plus/icons-vue";
import { shortFileName, useDynamicFormHeight } from "@/utils/utils";

const selectedFiles = ref([]);
const isLoading = ref(false);
const tableRef = ref(null);
const data = reactive({
  filePath: "",
  fileFormats: ["*"],
  mode: "Memory",
  skipRows: "0"
});
const { formHeight } = useDynamicFormHeight(205);

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
    const rows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = rows.map((file: any) => {
      return { filename: shortFileName(file) };
    });
  } else if (selected === null) {
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

  isLoading.value = true;

  const saveFileType = outputPath.split(".").pop();

  try {
    const result: string = await invoke("concat", {
      filePath: data.filePath,
      outputPath: outputPath,
      fileType: saveFileType,
      mode: data.mode,
      skipRows: data.skipRows
    });

    if (JSON.stringify(result).startsWith("cat failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Cat done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 20000
    });
  } catch (err) {
    ElNotification({
      title: "Invoke cat error",
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
          <el-button @click="selectFile()" :icon="FolderOpened" plain>
            Open File
          </el-button>
          <el-tooltip
            content="Polars memory or stream, Csv stream Cat"
            placement="top"
            effect="light"
          >
            <el-select
              v-model="data.mode"
              style="margin-left: 12px; width: 100px"
            >
              <el-option label="Memory" value="memory" />
              <el-option label="Stream" value="stream" />
              <el-option label="Csv" value="csv" />
            </el-select>
          </el-tooltip>
          <el-tooltip content="skip rows" placement="top" effect="light">
            <el-input
              v-model="data.skipRows"
              style="margin-left: 12px; width: 80px"
              placeholder="skip rows"
            />
          </el-tooltip>
          <el-button
            @click="concatData()"
            :loading="isLoading"
            :icon="Connection"
            plain
            style="margin-left: 12px"
          >
            Cat
          </el-button>
        </div>
        <el-text> Cat CSV and Excel files </el-text>
      </div>
    </el-form>
    <el-table
      ref="tableRef"
      :data="selectedFiles"
      :height="formHeight"
      style="width: 100%"
    >
      <el-table-column type="index" width="50" />
      <el-table-column prop="filename" />
    </el-table>
  </el-form>
</template>

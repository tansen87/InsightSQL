<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { FolderOpened, SwitchFilled } from "@element-plus/icons-vue";
import { useDynamicFormHeight, shortFileName } from "@/utils/utils";

const [selectedFiles, isLoading] = [ref([]), ref(false)];
const data = reactive({
  path: "",
  fileFormats: ["mdb", "accdb"],
  sep: "|"
});
const { formHeight } = useDynamicFormHeight(134);

async function selectFile() {
  selectedFiles.value = [];

  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "Access",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.path = selected.join("|").toString();
    const nonEmptyRows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = nonEmptyRows.map((file: any) => {
      return { filename: shortFileName(file) };
    });
  } else if (selected === null) {
    return;
  } else {
    data.path = selected;
  }
}

// invoke access
async function accessData() {
  if (data.path === "") {
    ElNotification({
      title: "File not found",
      message: "未选择文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  isLoading.value = true;

  try {
    const result: string = await invoke("access", {
      path: data.path,
      sep: data.sep
    });

    if (JSON.stringify(result).startsWith("access failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Convert done, elapsed time: ${result}`,
      position: "bottom-right",
      type: "success",
      duration: 5000
    });
  } catch (err) {
    ElNotification({
      title: "Access failed",
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
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened" plain>
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
          plain
          style="margin-left: 10px"
        >
          Convert
        </el-button>
      </div>

      <el-text> Convert Access Database to CSV </el-text>
    </div>

    <el-table
      :data="selectedFiles"
      :height="formHeight"
      style="width: 100%"
      empty-text=""
    >
      <el-table-column type="index" width="50" />
      <el-table-column prop="filename" />
    </el-table>
  </el-form>
</template>

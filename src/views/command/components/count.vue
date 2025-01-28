<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification, ElIcon } from "element-plus";
import {
  Loading,
  FolderOpened,
  Grape,
  CloseBold,
  Select
} from "@element-plus/icons-vue";
import { shortFileName, useDynamicFormHeight } from "@/utils/utils";

const [isLoading, selectedFiles] = [ref(false), ref([])];
const data = reactive({
  path: "",
  fileFormats: ["*"]
});
const { formHeight } = useDynamicFormHeight(134);

listen("start_convert", (event: any) => {
  const startConvert: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert) {
      file.status = "";
    }
  });
});
listen("count_err", (event: any) => {
  const countErr: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === countErr.split("|")[0]) {
      file.status = "error";
      file.infoMsg = countErr.split("|")[1];
    }
  });
  isLoading.value = false;
});
listen("count_msg", (event: any) => {
  const countMsg: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === countMsg.split("|")[0]) {
      file.status = "completed";
      file.infoMsg = countMsg.split("|")[1];
    }
  });
});

async function selectFile() {
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "csv",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.path = selected.join("|").toString();
    const nonEmptyRows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = nonEmptyRows.map((file: any) => {
      return { filename: shortFileName(file), status: " " };
    });
  } else if (selected === null) {
    return;
  } else {
    data.path = selected;
  }
}

// invoke count
async function countData() {
  if (data.path === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  isLoading.value = true;

  try {
    const result: string = await invoke("count", {
      path: data.path
    });

    if (JSON.stringify(result).startsWith("count failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Count done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 5000
    });
  } catch (err) {
    ElNotification({
      title: "Count failed",
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

        <el-button
          @click="countData()"
          :loading="isLoading"
          :icon="Grape"
          plain
          style="margin-left: 10px"
        >
          Count
        </el-button>
      </div>

      <el-text> Count the rows of CSV files </el-text>
    </div>

    <el-table
      :data="selectedFiles"
      :height="formHeight"
      style="width: 100%"
      show-overflow-tooltip
      empty-text=""
    >
      <el-table-column type="index" width="50" />
      <el-table-column
        prop="filename"
        label="File"
        class-name="file-column"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 30%"
      />
      <el-table-column
        prop="status"
        label="Status"
        class-name="status-column"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 10%"
      >
        <template #default="scope">
          <ElIcon v-if="scope.row.status === ''" class="is-loading">
            <Loading />
          </ElIcon>
          <ElIcon v-else-if="scope.row.status === 'completed'" color="#00CD66">
            <Select />
          </ElIcon>
          <ElIcon v-else-if="scope.row.status === 'error'" color="#FF0000">
            <CloseBold />
          </ElIcon>
        </template>
      </el-table-column>
      <el-table-column
        prop="infoMsg"
        label="Info"
        class-name="info-column"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 60%"
      >
        <template #default="scope">
          <span v-if="scope.row.status === 'error'">{{
            scope.row.infoMsg
          }}</span>
        </template>
      </el-table-column>
    </el-table>
  </el-form>
</template>

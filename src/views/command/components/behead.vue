<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import {
  FolderOpened,
  Loading,
  Select,
  CloseBold,
  Delete
} from "@element-plus/icons-vue";
import { shortFileName, useDynamicFormHeight } from "@/utils/utils";

const [isLoading, selectedFiles, progress] = [ref(false), ref([]), ref(0)];
const data = reactive({
  path: "",
  fileFormats: ["*"],
  skipRows: "0"
});
const customColors = [
  { color: "#98FB98", percentage: 20 },
  { color: "#7CFC00", percentage: 40 },
  { color: "#7FFF00", percentage: 60 },
  { color: "#ADFF2F", percentage: 80 },
  { color: "#9ACD32", percentage: 100 }
];
const { formHeight } = useDynamicFormHeight(134);

listen("start_convert", (event: any) => {
  const startConvert: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert) {
      file.status = "loading";
    }
  });
});
listen("behead_err", (event: any) => {
  const beheadErr: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === beheadErr.split("|")[0]) {
      file.status = "error";
      file.errorMessage = beheadErr.split("|")[1];
    }
  });
  isLoading.value = false;
});
listen("drop_progress", (event: any) => {
  const pgs: any = event.payload;
  progress.value = pgs;
});
listen("drop_msg", (event: any) => {
  const dropMsg: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === dropMsg) {
      file.status = "completed";
    }
  });
});

async function selectFile() {
  selectedFiles.value = [];
  progress.value = 0;

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
      return { filename: shortFileName(file), status: "" };
    });
  } else if (selected === null) {
    return;
  } else {
    data.path = selected;
  }
}

// invoke behead
async function dropHeaders() {
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
    const result: string = await invoke("behead", {
      path: data.path,
      skipRows: data.skipRows
    });

    if (result.startsWith("behead failed:")) {
      throw result.toString();
    }

    ElNotification({
      message: `Drop done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
  } catch (err) {
    ElNotification({
      title: "Behead failed",
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

        <el-tooltip content="skip rows" placement="top" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 50px"
            placeholder="skip rows"
          />
        </el-tooltip>

        <el-button
          @click="dropHeaders()"
          :loading="isLoading"
          :icon="Delete"
          style="margin-left: 10px"
          plain
        >
          Drop
        </el-button>
      </div>

      <el-text>
        <span>Drop headers from CSV</span>
      </el-text>
    </div>

    <el-table
      :data="selectedFiles"
      :height="formHeight"
      style="width: 100%"
      empty-text=""
    >
      <el-table-column type="index" width="50" />
      <el-table-column prop="filename" label="file" style="width: 80%" />
      <el-table-column prop="status" label="status" width="100">
        <template #default="scope">
          <ElIcon v-if="scope.row.status === 'loading'" class="is-loading">
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
        prop="errorMessage"
        label="Info"
        class-name="info-column"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 60%"
      >
        <template #default="scope">
          <span v-if="scope.row.status === 'error'">
            {{ scope.row.errorMessage }}
          </span>
        </template>
      </el-table-column>
    </el-table>

    <el-progress
      v-if="isLoading"
      :percentage="progress"
      :color="customColors"
    />
  </el-form>
</template>

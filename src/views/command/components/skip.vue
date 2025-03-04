<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  FolderOpened,
  Loading,
  Select,
  CloseBold,
  Delete
} from "@element-plus/icons-vue";
import { shortFileName, useDynamicHeight } from "@/utils/utils";
import { message } from "@/utils/message";

const [isLoading, selectedFiles] = [ref(false), ref([])];
const data = reactive({
  path: "",
  skipRows: "1"
});
const { dynamicHeight } = useDynamicHeight(134);

listen("start_convert", (event: any) => {
  const startConvert: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert) {
      file.status = "loading";
    }
  });
});
listen("skip_err", (event: any) => {
  const beheadErr: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === beheadErr.split("|")[0]) {
      file.status = "error";
      file.errorMessage = beheadErr.split("|")[1];
    }
  });
  isLoading.value = false;
});
listen("skip_msg", (event: any) => {
  const skipMsg: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === skipMsg) {
      file.status = "completed";
    }
  });
});

async function selectFile() {
  selectedFiles.value = [];

  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "csv",
        extensions: ["*"]
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
async function skipLines() {
  if (data.path === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;

    const result: string = await invoke("skip", {
      path: data.path,
      skipRows: data.skipRows
    });

    message(`Skip done, elapsed time: ${result} s`, { type: "success" });
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
        <el-tooltip content="skip rows" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 50px"
          />
        </el-tooltip>
        <el-button
          @click="skipLines()"
          :loading="isLoading"
          :icon="Delete"
          style="margin-left: 10px"
        >
          Skip
        </el-button>
      </div>

      <el-text>
        <span>Skip rows from CSV</span>
      </el-text>
    </div>

    <el-table
      :data="selectedFiles"
      :height="dynamicHeight"
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
  </el-form>
</template>

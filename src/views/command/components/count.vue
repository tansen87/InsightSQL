<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElIcon } from "element-plus";
import {
  Loading,
  FolderOpened,
  CloseBold,
  Select,
  Link
} from "@element-plus/icons-vue";
import { shortFileName, useDynamicHeight } from "@/utils/utils";
import { message } from "@/utils/message";
import { useMarkdown, mdCount } from "@/utils/markdown";

const mode = ref("count");
const path = ref("");
const [dialog, isLoading] = [ref(false), ref(false)];
const selectedFiles = ref([]);
const { dynamicHeight } = useDynamicHeight(143);
const { mdShow } = useMarkdown(mdCount);

listen("start-count", (event: any) => {
  const startConvert: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert) {
      file.status = "";
    }
  });
});
listen("count-err", (event: any) => {
  const countErr: string = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === countErr.split("|")[0]) {
      file.status = "error";
      file.infoMsg = countErr.split("|")[1];
    }
  });
});
listen("count-msg", (event: any) => {
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
        extensions: ["*"]
      }
    ]
  });
  if (Array.isArray(selected)) {
    path.value = selected.join("|").toString();
    const nonEmptyRows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = nonEmptyRows.map((file: any) => {
      return { filename: shortFileName(file), status: " " };
    });
  } else if (selected === null) {
    return;
  } else {
    path.value = selected;
  }
}

// invoke count
async function countData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("count", {
      path: path.value,
      mode: mode.value
    });
    message(`${mode.value} done, elapsed time: ${rtime} s`, {
      type: "success"
    });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <div class="page-container">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
        <el-tooltip content="count mode" effect="light">
          <el-select v-model="mode" style="margin-left: 10px; width: 90px">
            <el-option label="Index" value="index" />
            <el-option label="Count" value="count" />
            <el-option label="Check" value="check" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button
        @click="countData()"
        :loading="isLoading"
        :icon="Select"
        style="margin-left: 8px"
      >
        Count
      </el-button>
    </div>
    <el-table
      :data="selectedFiles"
      :height="dynamicHeight"
      style="width: 100%"
      show-overflow-tooltip
      empty-text=""
    >
      <el-table-column type="index" width="50" />
      <el-table-column
        prop="filename"
        label="File"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 30%"
      />
      <el-table-column
        prop="status"
        label="Status"
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
        label="Message"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 60%"
      >
        <template #default="scope">
          <span v-if="scope.row.status === 'error'">
            {{ scope.row.infoMsg }}
          </span>
        </template>
      </el-table-column>
    </el-table>
    <div class="custom-container1">
      <div class="custom-container2" />
      <el-link @click="dialog = true" :icon="Link">
        <span>
          About
          <span style="color: skyblue; font-weight: bold">Count</span>
        </span>
      </el-link>
    </div>
    <el-dialog
      v-model="dialog"
      title="Count - Count the rows of CSV files"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

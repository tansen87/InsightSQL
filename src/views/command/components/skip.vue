<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Event } from "@tauri-apps/api/event";
import {
  FolderOpened,
  Loading,
  Select,
  CloseBold,
  Delete,
  Link
} from "@element-plus/icons-vue";
import { useDynamicHeight, updateEvent } from "@/utils/utils";
import { message } from "@/utils/message";
import { trimOpenFile } from "@/utils/view";
import { useMarkdown, mdSkip } from "@/utils/markdown";

const path = ref("");
const fileSelect = ref([]);
const [skipRows, progress] = [ref("1"), ref("nil")];
const [dialog, isLoading] = [ref(false), ref(false)];
const { dynamicHeight } = useDynamicHeight(143);
const { mdShow } = useMarkdown(mdSkip);

listen("update-msg", (event: Event<string>) => {
  const [filename, rows] = event.payload.split("|");
  updateEvent(fileSelect, filename, file => {
    file.currentRows = rows;
  });
});
listen("total-msg", (event: Event<string>) => {
  const [filename, rows] = event.payload.split("|");
  updateEvent(fileSelect, filename, file => {
    file.totalRows = rows;
  });
});
listen("info", (event: Event<string>) => {
  const filename = event.payload;
  updateEvent(fileSelect, filename, file => {
    file.status = "loading";
  });
});
listen("err", (event: Event<string>) => {
  const [filename, message] = event.payload.split("|");
  updateEvent(fileSelect, filename, file => {
    file.status = "error";
    file.message = message;
  });
  isLoading.value = false;
});
listen("success", (event: Event<string>) => {
  const filename = event.payload;
  updateEvent(fileSelect, filename, file => {
    file.status = "success";
  });
});

async function selectFile() {
  fileSelect.value = [];
  const trimFile = await trimOpenFile(true, "csv", ["*"], {
    includeStatus: true
  });
  path.value = trimFile.filePath;
  fileSelect.value = trimFile.fileInfo;
}

// invoke skip
async function skipLines() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const result: string = await invoke("skip", {
      path: path.value,
      progress: progress.value,
      skipRows: skipRows.value
    });
    message(`Skip done, elapsed time: ${result} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
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
        <el-tooltip content="skip rows" effect="light">
          <el-input v-model="skipRows" style="margin-left: 8px; width: 50px" />
        </el-tooltip>
        <el-tooltip content="if nil, no progress bar" effect="light">
          <el-select v-model="progress" style="margin-left: 8px; width: 70px">
            <el-option label="idx" value="idx" />
            <el-option label="nil" value="nil" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button
        @click="skipLines()"
        :loading="isLoading"
        :icon="Delete"
        style="margin-left: 8px"
      >
        Skip
      </el-button>
    </div>

    <el-table
      :data="fileSelect"
      :height="dynamicHeight"
      style="width: 100%"
      empty-text=""
    >
      <el-table-column type="index" width="50" />
      <el-table-column prop="filename" label="File" style="width: 80%" />
      <el-table-column prop="status" label="Status" width="70">
        <template #default="scope">
          <ElIcon v-if="scope.row.status === 'loading'" class="is-loading">
            <Loading />
          </ElIcon>
          <ElIcon v-else-if="scope.row.status === 'success'" color="#00CD66">
            <Select />
          </ElIcon>
          <ElIcon v-else-if="scope.row.status === 'error'" color="#FF0000">
            <CloseBold />
          </ElIcon>
        </template>
      </el-table-column>
      <el-table-column
        prop="message"
        label="Message"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 60%"
      >
        <template #default="scope">
          <span v-if="scope.row.status === 'error'">
            {{ scope.row.message }}
          </span>
          <el-progress
            v-if="
              scope.row.totalRows !== 0 &&
              isFinite(scope.row.currentRows / scope.row.totalRows)
            "
            :percentage="
              Math.round((scope.row.currentRows / scope.row.totalRows) * 100)
            "
          />
        </template>
      </el-table-column>
    </el-table>
    <div class="custom-container1">
      <div class="custom-container2" />
      <el-link @click="dialog = true" :icon="Link">
        <span>
          About
          <span style="color: skyblue; font-weight: bold">Skip</span>
        </span>
      </el-link>
    </div>
    <el-dialog v-model="dialog" title="Skip - Skip rows from CSV" width="800">
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

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
  ArrowRight
} from "@element-plus/icons-vue";
import { useDynamicHeight, updateEvent } from "@/utils/utils";
import { useDark } from "@pureadmin/utils";
import { message } from "@/utils/message";
import { trimOpenFile } from "@/utils/view";
import { useMarkdown, mdSkip } from "@/utils/markdown";

const path = ref("");
const fileSelect = ref([]);
const [skipRows, progress] = [ref("1"), ref("nil")];
const pgsOptions = [
  { label: "Nil", value: "nil" },
  { label: "Idx", value: "idx" }
];
const [dialog, isLoading] = [ref(false), ref(false)];
const { dynamicHeight } = useDynamicHeight(122);
const { mdShow } = useMarkdown(mdSkip);
const { isDark } = useDark();

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
  <el-form class="page-container">
    <el-splitter>
      <el-splitter-panel size="160" :resizable="false">
        <div class="splitter-container">
          <el-tooltip content="Add data" effect="light" placement="right">
            <el-button @click="selectFile()" :icon="FolderOpened" circle text />
          </el-tooltip>

          <el-tooltip
            content="if nil, no progress bar"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle">
              <span
                v-for="item in pgsOptions"
                :key="item.value"
                class="mode-item"
                :class="{
                  active: progress === item.value,
                  'active-dark': isDark && progress === item.value
                }"
                @click="progress = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <el-tooltip content="skip rows" effect="light" placement="right">
            <el-input
              v-model="skipRows"
              style="margin-left: 8px; margin-top: 8px; width: 140px"
            />
          </el-tooltip>

          <el-link @click="dialog = true" style="margin-top: auto">
            <span class="link-text">Skip</span>
          </el-link>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-tooltip content="Run" effect="light" placement="right">
          <el-button
            @click="skipLines()"
            :loading="isLoading"
            :icon="ArrowRight"
            circle
            text
          />
        </el-tooltip>

        <el-table
          :data="fileSelect"
          :height="dynamicHeight"
          show-overflow-tooltip
        >
          <el-table-column type="index" width="35" />
          <el-table-column prop="filename" label="File" style="width: 80%" />
          <el-table-column prop="status" label="Status" width="70">
            <template #default="scope">
              <ElIcon v-if="scope.row.status === 'loading'" class="is-loading">
                <Loading />
              </ElIcon>
              <ElIcon
                v-else-if="scope.row.status === 'success'"
                color="#00CD66"
              >
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
                  Math.round(
                    (scope.row.currentRows / scope.row.totalRows) * 100
                  )
                "
              />
            </template>
          </el-table-column>
        </el-table>
      </el-splitter-panel>
    </el-splitter>

    <el-dialog v-model="dialog" title="Skip - Skip rows from CSV" width="800">
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

<style scoped>
.mode-toggle {
  width: 140px;
}
</style>

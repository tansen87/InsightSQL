<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Event } from "@tauri-apps/api/event";
import { ElIcon } from "element-plus";
import {
  Loading,
  FolderOpened,
  CloseBold,
  Select,
  ArrowRight
} from "@element-plus/icons-vue";
import { useDark } from "@pureadmin/utils";
import { shortFileName, useDynamicHeight, updateEvent } from "@/utils/utils";
import { message } from "@/utils/message";
import { useMarkdown, mdCount } from "@/utils/markdown";

const mode = ref("count");
const modeOptions = [
  { label: "Count", value: "count" },
  { label: "Index", value: "index" },
  { label: "Check", value: "check" }
];
const path = ref("");
const [dialog, isLoading] = [ref(false), ref(false)];
const fileSelect = ref([]);
const { dynamicHeight } = useDynamicHeight(122);
const { mdShow } = useMarkdown(mdCount);
const { isDark } = useDark();

listen("info", (event: Event<string>) => {
  const filename = event.payload;
  updateEvent(fileSelect, filename, file => {
    file.status = "";
  });
});
listen("err", (event: Event<string>) => {
  const [filename, message] = event.payload.split("|");
  updateEvent(fileSelect, filename, file => {
    file.status = "error";
    file.message = message;
  });
});
listen("success", (event: Event<string>) => {
  const [filename, message] = event.payload.split("|");
  updateEvent(fileSelect, filename, file => {
    file.status = "success";
    file.message = message;
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
    fileSelect.value = nonEmptyRows.map((file: any) => {
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
  <el-form class="page-container">
    <el-splitter>
      <el-splitter-panel size="200" :resizable="false">
        <div class="splitter-container">
          <el-tooltip content="Add data" effect="light">
            <el-button @click="selectFile()" :icon="FolderOpened" circle text />
          </el-tooltip>

          <div class="mode-toggle">
            <span
              v-for="item in modeOptions"
              :key="item.value"
              class="mode-item"
              :class="{
                active: mode === item.value,
                'active-dark': isDark && mode === item.value
              }"
              @click="mode = item.value"
            >
              {{ item.label }}
            </span>
          </div>

          <el-link @click="dialog = true" style="margin-top: auto">
            <span class="link-text">Count</span>
          </el-link>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-tooltip content="Run" effect="light">
          <el-button
            @click="countData()"
            :loading="isLoading"
            :icon="ArrowRight"
            circle
            text
          />
        </el-tooltip>

        <el-table
          :data="fileSelect"
          :height="dynamicHeight"
          style="width: 100%"
          show-overflow-tooltip
        >
          <el-table-column type="index" width="35" />
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
            </template>
          </el-table-column>
        </el-table>
      </el-splitter-panel>
    </el-splitter>

    <el-dialog
      v-model="dialog"
      title="Count - Count the rows of CSV files"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

<style scoped>
.mode-toggle {
  width: 180px;
}
</style>

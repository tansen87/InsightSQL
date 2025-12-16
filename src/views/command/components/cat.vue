<script setup lang="ts">
import { ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import {
  FolderOpened,
  ArrowRight,
  Link,
  Loading
} from "@element-plus/icons-vue";
import { useDark } from "@pureadmin/utils";
import { useDynamicHeight } from "@/utils/utils";
import { mdCat, useMarkdown } from "@/utils/markdown";
import { message, closeAllMessage } from "@/utils/message";
import { trimOpenFile } from "@/utils/view";

const mode = ref("polars");
const modeOptions = [
  { label: "Polars", value: "polars" },
  { label: "Csv", value: "csv" },
  { label: "Duplicate", value: "duplicate" }
];
const [columns, backendInfo, path] = [ref(""), ref(""), ref("")];
const [fileSelect, originalColumns] = [ref([]), ref([])];
const [isLoading, backendCompleted, dialog] = [
  ref(false),
  ref(false),
  ref(false)
];
const { dynamicHeight } = useDynamicHeight(122);
const { mdShow } = useMarkdown(mdCat);
const { isDark } = useDark();

async function selectFile() {
  columns.value = "";
  fileSelect.value = [];
  originalColumns.value = [];
  backendInfo.value = "";
  backendCompleted.value = false;
  try {
    const trimFile = await trimOpenFile(true, "", ["*"], {
      includeStatus: false
    });
    path.value = trimFile.filePath;
    fileSelect.value = trimFile.fileInfo;
    message("fetching headers...", {
      type: "info",
      duration: 0,
      icon: Loading
    });
    const headers: string[] = await invoke("inter_headers", {
      path: path.value
    });
    originalColumns.value = headers.map(header => ({
      label: header,
      value: header
    }));
    closeAllMessage();
    backendInfo.value = "headers fetched successfully";
    backendCompleted.value = true;
  } catch (err) {
    closeAllMessage();
    message(err.toString(), { type: "error" });
  }
}

// invoke concat
async function concatData() {
  if (path.value === "") {
    message("File not selected", { type: "warning" });
    return;
  }
  if (mode.value === "duplicate") {
    message("find duplicate headers...", {
      type: "info",
      duration: 0,
      icon: Loading
    });
    await invoke("dupli_headers", {
      path: path.value
    });
    backendInfo.value = "find duplicate headers done";
    backendCompleted.value = true;
    closeAllMessage();
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
    message("Save file not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const saveFileType = outputPath.split(".").pop();
    const useCols = Object.values(columns.value).join("|");
    const rtime: string = await invoke("concat", {
      path: path.value,
      outputPath: outputPath,
      fileType: saveFileType,
      mode: mode.value,
      useCols: useCols
    });
    backendInfo.value = `${mode.value} done, elapsed time: ${rtime} s`;
    backendCompleted.value = true;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container" :style="dynamicHeight">
    <el-splitter>
      <el-splitter-panel size="240" :resizable="false">
        <div style="display: flex; flex-direction: column; height: 100%">
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

          <template v-if="['polars'].includes(mode)">
            <el-tooltip
              content="If column is empty, files have no common headers"
              effect="light"
              placement="right"
            >
              <el-select
                v-model="columns"
                multiple
                filterable
                style="margin-top: 8px; margin-left: 8px; width: 220px"
                placeholder="Cat specific column"
              >
                <el-option
                  v-for="item in originalColumns"
                  :key="item.value"
                  :label="item.label"
                  :value="item.value"
                />
              </el-select>
            </el-tooltip>
          </template>

          <el-link @click="dialog = true" :icon="Link" style="margin-top: auto">
            <span v-if="backendCompleted"> {{ backendInfo }} </span>
            <span v-else>
              About
              <span style="color: skyblue; font-weight: bold">Cat</span>
            </span>
          </el-link>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-tooltip content="Run" effect="light">
          <el-button
            @click="concatData()"
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
          style="width: 100%"
        >
          <el-table-column type="index" width="35" />
          <el-table-column
            prop="filename"
            label="file"
            :class="{ 'custom-width': true }"
            style="flex: 0 0 60%"
          />
          <el-table-column
            prop="infoMsg"
            label="duplicate headers"
            :class="{ 'custom-width': true }"
            style="flex: 0 0 40%"
          >
            <template #default="scope">
              {{ scope.row.infoMsg }}
            </template>
          </el-table-column>
        </el-table>
      </el-splitter-panel>
    </el-splitter>

    <el-dialog
      v-model="dialog"
      title="Cat - Merge multiple CSV or Excel files into one CSV or xlsx file"
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
  width: 220px;
}
</style>

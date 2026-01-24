<script setup lang="ts">
import { ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Loading, SwitchButton } from "@element-plus/icons-vue";
import { useDark } from "@pureadmin/utils";
import { useDynamicHeight } from "@/utils/utils";
import { mdCat, useMarkdown } from "@/utils/markdown";
import { message, closeAllMessage } from "@/utils/message";
import { trimOpenFile } from "@/utils/view";
import { useQuoting } from "@/store/modules/options";

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
const { dynamicHeight } = useDynamicHeight(74);
const { mdShow } = useMarkdown(mdCat);
const { isDark } = useDark();
const quotingStore = useQuoting();

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
      useCols: useCols,
      quoting: quotingStore.quoting
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
        <div class="splitter-container">
          <el-button @click="selectFile()" :icon="FolderOpened" text round>
            Open File(s)
          </el-button>

          <div class="mode-toggle w-[220px]">
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
                class="ml-2 mt-2"
                style="width: 220px"
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

          <el-link @click="dialog = true" class="mt-auto">
            <span v-if="backendCompleted"> {{ backendInfo }} </span>
            <span v-else class="link-text">Cat</span>
          </el-link>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-button
          @click="concatData()"
          :loading="isLoading"
          :icon="SwitchButton"
          text
          round
          >Run
        </el-button>

        <el-table
          :data="fileSelect"
          :height="dynamicHeight"
          show-overflow-tooltip
          tooltip-effect="light"
        >
          <el-table-column type="index" width="35" />
          <el-table-column prop="filename" label="file" />
          <el-table-column prop="infoMsg" label="duplicate headers">
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
      width="70%"
    >
      <el-scrollbar :height="dynamicHeight * 0.7">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Files, SwitchButton } from "@element-plus/icons-vue";
import { useDynamicHeight } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { mdReplace, useMarkdown } from "@/utils/markdown";

const [isLoading, dialog] = [ref(false), ref(false)];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const [column, path, regexPattern, replacement] = [
  ref(""),
  ref(""),
  ref(""),
  ref("")
];
const { dynamicHeight } = useDynamicHeight(98);
const { mdShow } = useMarkdown(mdReplace);

async function selectFile() {
  column.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

  try {
    tableHeader.value = await mapHeaders(path.value, "0");
    const { columnView, dataView } = await toJson(path.value);
    tableColumn.value = columnView;
    tableData.value = dataView;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke replace
async function replaceData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (column.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("replace", {
      path: path.value,
      column: column.value,
      regexPattern: regexPattern.value,
      replacement: replacement.value
    });
    message(`Replace done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container">
    <el-splitter>
      <el-splitter-panel size="180" :resizable="false">
        <div class="splitter-container">
          <el-button @click="selectFile()" :icon="FolderOpened" text round>
            Open File
          </el-button>

          <div class="ml-2 w-40 space-y-2">
            <el-select v-model="column" filterable placeholder="Select column">
              <el-option
                v-for="item in tableHeader"
                :key="item.value"
                :label="item.label"
                :value="item.value"
              />
            </el-select>

            <el-input v-model="regexPattern" placeholder="regex pattern" />

            <el-input v-model="replacement" placeholder="replacement" />
          </div>

          <el-link @click="dialog = true" class="mt-auto">
            <span class="link-text">Replace</span>
          </el-link>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-button
          @click="replaceData()"
          :loading="isLoading"
          :icon="SwitchButton"
          text
          round
          >Run
        </el-button>

        <el-table
          :data="tableData"
          :height="dynamicHeight"
          show-overflow-tooltip
          tooltip-effect="light"
        >
          <el-table-column
            v-for="column in tableColumn"
            :prop="column.prop"
            :label="column.label"
            :key="column.prop"
          />
        </el-table>

        <el-text>
          <el-icon class="ml-2">
            <Files />
          </el-icon>
          {{ path }}
        </el-text>
      </el-splitter-panel>
    </el-splitter>

    <el-dialog
      v-model="dialog"
      title="Replace - Replace CSV data using a regex"
      width="70%"
    >
      <el-scrollbar :height="dynamicHeight * 0.7">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

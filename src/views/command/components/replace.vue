<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Files, Link, ArrowRight } from "@element-plus/icons-vue";
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
const { dynamicHeight } = useDynamicHeight(146);
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
          <el-tooltip content="Add data" effect="light" placement="right">
            <el-button @click="selectFile()" :icon="FolderOpened" circle text />
          </el-tooltip>

          <el-select
            v-model="column"
            filterable
            style="width: 160px; margin-left: 8px"
            placeholder="Select column"
          >
            <el-option
              v-for="item in tableHeader"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>

          <el-input
            style="margin-left: 8px; margin-top: 8px; width: 160px"
            placeholder="regex pattern"
            v-model="regexPattern"
          />

          <el-input
            v-model="replacement"
            style="margin-left: 8px; margin-top: 8px; width: 160px"
            placeholder="replacement"
          />

          <el-link @click="dialog = true" :icon="Link" style="margin-top: auto">
            <span>
              About
              <span style="color: skyblue; font-weight: bold">Replace</span>
            </span>
          </el-link>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-tooltip content="Run" effect="light" placement="right">
          <el-button
            @click="replaceData()"
            :loading="isLoading"
            :icon="ArrowRight"
            circle
            text
          />
        </el-tooltip>

        <el-table
          :data="tableData"
          :height="dynamicHeight"
          show-overflow-tooltip
        >
          <el-table-column
            v-for="column in tableColumn"
            :prop="column.prop"
            :label="column.label"
            :key="column.prop"
          />
        </el-table>

        <el-text>
          <el-icon style="margin-left: 8px">
            <Files />
          </el-icon>
          {{ path }}
        </el-text>
      </el-splitter-panel>
    </el-splitter>

    <el-dialog
      v-model="dialog"
      title="Replace - Replace CSV data using a regex"
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
  width: 160px;
}
</style>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Refresh, FolderOpened, Link } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { mdFill, useMarkdown } from "@/utils/markdown";

const [fillChar, mode] = [ref("0"), ref("fill")];
const [isLoading, dialog] = [ref(false), ref(false)];
const [columns, path] = [ref(""), ref("")];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const { dynamicHeight } = useDynamicHeight(196);
const { mdShow } = useMarkdown(mdFill);

async function selectFile() {
  path.value = "";
  columns.value = "";
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

// invoke fill
async function fillData() {
  if (path.value === "") {
    message("File not selected", { type: "warning" });
    return;
  }
  if (columns.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const cols = Object.values(columns.value).join("|");
    const rtime: string = await invoke("fill", {
      path: path.value,
      columns: cols,
      values: fillChar.value,
      mode: mode.value
    });
    message(`Fill done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <div class="page-container">
    <div class="custom-container1">
      <div clas="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
        <el-tooltip content="The value of fill" effect="light">
          <el-input
            v-model="fillChar"
            style="width: 120px; margin-left: 10px"
          />
        </el-tooltip>
        <el-tooltip content="fill mode" effect="light">
          <el-select v-model="mode" style="width: 80px; margin-left: 10px">
            <el-option label="fill" value="fill" />
            <el-option label="f-fill" value="ffill" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button @click="fillData()" :loading="isLoading" :icon="Refresh">
        {{ mode }}
      </el-button>
    </div>
    <el-select
      v-model="columns"
      multiple
      filterable
      style="margin-top: 10px; width: 100%"
      placeholder="Select the columns to be filled in"
    >
      <el-option
        v-for="item in tableHeader"
        :key="item.value"
        :label="item.label"
        :value="item.value"
      />
    </el-select>
    <el-table
      :data="tableData"
      :height="dynamicHeight"
      border
      empty-text=""
      style="margin-top: 10px; width: 100%"
      show-overflow-tooltip
    >
      <el-table-column
        v-for="column in tableColumn"
        :prop="column.prop"
        :label="column.label"
        :key="column.prop"
      />
    </el-table>
    <div class="custom-container1">
      <div class="custom-container2">
        <el-tooltip :content="path" effect="light">
          <el-text>{{ shortFileName(path) }}</el-text>
        </el-tooltip>
      </div>
      <el-link @click="dialog = true" :icon="Link">
        <span>
          About
          <span style="color: skyblue; font-weight: bold">Fill</span>
        </span>
      </el-link>
    </div>
    <el-dialog
      v-model="dialog"
      title="Fill - Fill empty fields in selected columns of a CSV"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

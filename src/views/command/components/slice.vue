<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  FolderOpened,
  Refresh,
  Link,
  TurnOff,
  Open
} from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { sliceContent, useMarkdown } from "@/utils/markdown";
import { message } from "@/utils/message";

const [selectColumn, path] = [ref(""), ref("")];
const [n, length, strSep, mode] = [ref("4"), ref("5"), ref("-"), ref("left")];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const [isLoading, isPath, infoDialog, reverse] = [
  ref(false),
  ref(false),
  ref(false),
  ref(false)
];
const { dynamicHeight } = useDynamicHeight(178);

async function selectFile() {
  isPath.value = false;
  selectColumn.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) {
    return;
  }

  try {
    tableHeader.value = await mapHeaders(path.value, "0");
    const { columnView, dataView } = await toJson(path.value);
    tableColumn.value = columnView;
    tableData.value = dataView;
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke str_slice, str_split
async function sliceData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (selectColumn.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    if (
      mode.value === "left" ||
      mode.value === "right" ||
      mode.value === "startlen"
    ) {
      const rtime: string = await invoke("str_slice", {
        path: path.value,
        selectColumn: selectColumn.value,
        n: n.value,
        length: length.value,
        reverse: reverse.value,
        mode: mode.value
      });
      message(`Slice done, elapsed time: ${rtime} s`, { type: "success" });
    }
    if (mode.value === "nth" || mode.value === "nmax") {
      const rtime: string = await invoke("str_split", {
        path: path.value,
        selectColumn: selectColumn.value,
        n: n.value,
        strSep: strSep.value,
        mode: mode.value
      });
      message(`Split done, elapsed time: ${rtime} s`, { type: "success" });
    }
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}

const { compiledMarkdown } = useMarkdown(sliceContent);
</script>

<template>
  <div class="page-container">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
      </div>
      <el-link @click="infoDialog = true" :icon="Link">
        <span v-if="isPath">
          <el-tooltip :content="path" placement="top" effect="light">
            <span>{{ shortFileName(path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>
          About
          <span style="color: skyblue; font-weight: bold">String</span>
        </span>
      </el-link>
    </div>

    <div class="custom-container1">
      <div class="custom-container1" style="margin-top: 12px">
        <el-select
          v-model="selectColumn"
          filterable
          style="width: 200px"
          placeholder="Select column"
        >
          <el-option
            v-for="item in tableHeader"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
        <!-- 当 mode 为 'left' 或 'right' 时显示 n -->
        <el-tooltip
          v-if="mode === 'left' || mode === 'right'"
          content="Number of the string"
          effect="light"
        >
          <el-input v-model="n" style="margin-left: 10px; width: 50px" />
        </el-tooltip>
        <!-- 当 mode 为 'sl' 时显示 n 和 length -->
        <template v-if="mode === 'startlen'">
          <el-tooltip content="Start index" effect="light">
            <el-input v-model="n" style="margin-left: 10px; width: 50px" />
          </el-tooltip>
          <el-tooltip content="Length of the string" effect="light">
            <el-input v-model="length" style="margin-left: 10px; width: 50px" />
          </el-tooltip>
        </template>
        <!-- 当 mode 为 'nth' 或 'nmax' 时显示 n 和 strSep -->
        <template v-if="['nth', 'nmax'].includes(mode)">
          <el-tooltip content="Number of the string" effect="light">
            <el-input v-model="n" style="margin-left: 10px; width: 50px" />
          </el-tooltip>
          <el-tooltip content="String separator" effect="light">
            <el-input v-model="strSep" style="margin-left: 10px; width: 50px" />
          </el-tooltip>
        </template>
        <el-tooltip content="String mode" effect="light">
          <el-select v-model="mode" style="margin-left: 10px; width: 103px">
            <el-option label="Left" value="left" />
            <el-option label="Right" value="right" />
            <el-option label="StartLength" value="startlen" />
            <el-option label="Nth" value="nth" />
            <el-option label="Nmax" value="nmax" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="Reverse or not" effect="light">
          <el-switch
            v-model="reverse"
            inline-prompt
            style="
              --el-switch-on-color: #43cd80;
              --el-switch-off-color: #b0c4de;
              margin-left: 10px;
            "
            active-text="Y"
            inactive-text="N"
            :active-action-icon="Open"
            :inactive-action-icon="TurnOff"
          />
        </el-tooltip>
      </div>
      <el-button
        @click="sliceData()"
        :loading="isLoading"
        :icon="Refresh"
        style="margin-top: 10px"
      >
        String
      </el-button>
    </div>

    <el-table
      :data="tableData"
      :height="dynamicHeight"
      border
      empty-text=""
      style="margin-top: 12px; width: 100%"
      show-overflow-tooltip
    >
      <el-table-column
        v-for="column in tableColumn"
        :prop="column.prop"
        :label="column.label"
        :key="column.prop"
      />
    </el-table>

    <el-dialog
      v-model="infoDialog"
      title="String - Slicing of csv column"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="compiledMarkdown" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

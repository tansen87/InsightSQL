<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Refresh, Link } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { mapHeaders, viewOpenFile, viewSqlp } from "@/utils/view";
import { sliceContent, useMarkdown } from "@/utils/markdown";
import { message } from "@/utils/message";

const [
  isLoading,
  isPath,
  selectColumn,
  tableHeader,
  tableColumn,
  tableData,
  infoDialog,
  path,
  n,
  m,
  sliceSep,
  mode
] = [
  ref(false),
  ref(false),
  ref(""),
  ref([]),
  ref([]),
  ref([]),
  ref(false),
  ref(""),
  ref("4"),
  ref("5"),
  ref("-"),
  ref("left")
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
    const { columnView, dataView } = await viewSqlp(path.value, "0");
    tableColumn.value = columnView;
    tableData.value = dataView;
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke slice
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
    const rtime: string = await invoke("slice", {
      path: path.value,
      selectColumn: selectColumn.value,
      n: n.value,
      m: m.value,
      sliceSep: sliceSep.value,
      mode: mode.value
    });
    message(`Slice done, elapsed time: ${rtime} s`, { type: "success" });
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
          <span style="color: skyblue; font-weight: bold">Slice</span>
        </span>
      </el-link>
    </div>

    <div class="custom-container1">
      <div class="custom-container1" style="margin-top: 12px">
        <el-select
          v-model="selectColumn"
          filterable
          style="width: 200px"
          placeholder="Slice by column"
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
          content="Number of slice"
          effect="light"
        >
          <el-input v-model="n" style="margin-left: 10px; width: 50px" />
        </el-tooltip>
        <!-- 当 mode 为 'ss' 时显示 n 和 m -->
        <template v-if="mode === 'ss'">
          <el-tooltip content="Number of start" effect="light">
            <el-input v-model="n" style="margin-left: 10px; width: 50px" />
          </el-tooltip>
          <el-tooltip content="Number of stop" effect="light">
            <el-input v-model="m" style="margin-left: 10px; width: 50px" />
          </el-tooltip>
        </template>
        <!-- 当 mode 为 'nth' 或 'nmax' 时显示 n 和 sliceSep -->
        <template v-if="['nth', 'nmax'].includes(mode)">
          <el-tooltip content="Number of slice" effect="light">
            <el-input v-model="n" style="margin-left: 10px; width: 50px" />
          </el-tooltip>
          <el-tooltip content="Slice separator" effect="light">
            <el-input
              v-model="sliceSep"
              style="margin-left: 10px; width: 50px"
            />
          </el-tooltip>
        </template>
        <el-tooltip content="Slice mode" effect="light">
          <el-select v-model="mode" style="margin-left: 10px; width: 103px">
            <el-option label="Left" value="left" />
            <el-option label="Right" value="right" />
            <el-option label="StartStop" value="ss" />
            <el-option label="Nth" value="nth" />
            <el-option label="Nmax" value="nmax" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button
        @click="sliceData()"
        :loading="isLoading"
        :icon="Refresh"
        style="margin-top: 10px"
      >
        Slice
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
      title="Slice - Slicing of csv column"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="compiledMarkdown" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

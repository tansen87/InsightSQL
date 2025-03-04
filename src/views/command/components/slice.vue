<script setup lang="ts">
import { ref, reactive } from "vue";
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
  infoDialog
] = [ref(false), ref(false), ref(""), ref([]), ref([]), ref([]), ref(false)];
const data = reactive({
  path: "",
  skipRows: "0",
  n: "4",
  m: "5",
  sliceSep: "-",
  mode: "left"
});
const { dynamicHeight } = useDynamicHeight(190);

async function selectFile() {
  isPath.value = false;
  selectColumn.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

  data.path = await viewOpenFile(false, "csv", ["*"]);
  if (data.path === null) {
    return;
  }

  try {
    tableHeader.value = await mapHeaders(data.path, data.skipRows);
    const { columnView, dataView } = await viewSqlp(data.path, data.skipRows);
    tableColumn.value = columnView;
    tableData.value = dataView;
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
}

// invoke slice
async function sliceData() {
  if (data.path === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (selectColumn.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const result: string = await invoke("slice", {
      path: data.path,
      skipRows: data.skipRows,
      selectColumn: selectColumn.value,
      n: data.n,
      m: data.m,
      sliceSep: data.sliceSep,
      mode: data.mode
    });
    message(`Slice done, elapsed time: ${result} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
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
        <el-tooltip content="skip rows" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 78px"
          />
        </el-tooltip>
      </div>

      <el-link @click="infoDialog = true" :icon="Link">
        <span v-if="isPath">
          <el-tooltip :content="data.path" placement="top" effect="light">
            <span>{{ shortFileName(data.path) }}</span>
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
        <el-tooltip content="Numer of slice/start" effect="light">
          <el-input v-model="data.n" style="margin-left: 10px; width: 50px" />
        </el-tooltip>
        <el-tooltip content="Numer of stop" effect="light">
          <el-input v-model="data.m" style="margin-left: 10px; width: 50px" />
        </el-tooltip>
        <el-tooltip content="Slice separator" effect="light">
          <el-input
            v-model="data.sliceSep"
            style="margin-left: 10px; width: 50px"
          />
        </el-tooltip>
        <el-tooltip content="Slice mode" effect="light">
          <el-select v-model="data.mode" style="margin-left: 10px; width: 84px">
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

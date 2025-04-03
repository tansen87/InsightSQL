<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { IceCreamRound, FolderOpened, Link } from "@element-plus/icons-vue";
import { useDynamicHeight } from "@/utils/utils";
import { viewOpenFile, viewSqlp } from "@/utils/view";
import { splitContent, useMarkdown } from "@/utils/markdown";
import { message } from "@/utils/message";

const [
  isLoading,
  isPath,
  tableColumn,
  tableData,
  infoDialog,
  path,
  size,
  mode
] = [
  ref(false),
  ref(false),
  ref([]),
  ref([]),
  ref(false),
  ref(""),
  ref(1000000),
  ref("rows")
];
const { dynamicHeight } = useDynamicHeight(176);

async function selectFile() {
  isPath.value = false;
  tableColumn.value = [];
  tableData.value = [];

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) {
    return;
  }

  try {
    const { columnView, dataView } = await viewSqlp(path.value, "0");
    tableColumn.value = columnView;
    tableData.value = dataView;
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke split
async function splitData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("split", {
      path: path.value,
      size: size.value,
      mode: mode.value
    });
    message(`Split done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}

const { compiledMarkdown } = useMarkdown(splitContent);
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
        <span v-if="isPath">{{ path }}</span>
        <span v-else>
          About
          <span style="color: skyblue; font-weight: bold">Split</span>
        </span>
      </el-link>
    </div>
    <div class="custom-container1">
      <div class="custom-container2" style="margin-top: 10px">
        <el-tooltip content="Split rows" effect="light">
          <el-input-number
            v-model="size"
            controls-position="right"
            style="width: 172px"
          />
        </el-tooltip>
        <el-tooltip content="Split mode" effect="light">
          <el-select v-model="mode" style="margin-left: 10px; width: 80px">
            <el-option label="Rows" value="rows" />
            <el-option label="Lines" value="lines" />
            <el-option label="Index" value="index" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button
        style="margin-top: 10px"
        @click="splitData()"
        :loading="isLoading"
        :icon="IceCreamRound"
      >
        {{ mode }}-Split
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
      title="Split - Split one CSV file into many CSV files"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="compiledMarkdown" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

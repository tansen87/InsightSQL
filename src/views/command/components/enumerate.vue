<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { IceCreamRound, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { listen } from "@tauri-apps/api/event";

const mode = ref("nil");
const path = ref("");
const [currentRows, totalRows] = [ref(0), ref(0)];
const [isLoading, isPath] = [ref(false), ref(false)];
const [tableColumn, tableData] = [ref([]), ref([])];
const { dynamicHeight } = useDynamicHeight(148);

listen("update-rows", (event: any) => {
  currentRows.value = event.payload;
});
listen("total-rows", (event: any) => {
  totalRows.value = event.payload;
});

async function selectFile() {
  isPath.value = false;
  tableColumn.value = [];
  tableData.value = [];
  totalRows.value = 0;

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

  try {
    const { columnView, dataView } = await toJson(path.value);
    tableColumn.value = columnView;
    tableData.value = dataView;
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke enumer
async function enumerate() {
  if (path.value === "") {
    message("File not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("enumer", {
      path: path.value,
      mode: mode.value
    });
    message(`Enumerate done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <div class="page-container">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
        <el-tooltip content="if nil, do not add progress bar" effect="light">
          <el-select v-model="mode" style="margin-left: 10px; width: 70px">
            <el-option label="idx" value="idx" />
            <el-option label="std" value="std" />
            <el-option label="nil" value="nil" />
          </el-select>
        </el-tooltip>
        <el-button
          @click="enumerate()"
          :loading="isLoading"
          :icon="IceCreamRound"
          style="margin-left: 10px"
        >
          Enumerate
        </el-button>
      </div>
      <el-text>
        <span v-if="isPath">
          <el-tooltip :content="path" effect="light">
            <span>{{ shortFileName(path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>Add an index for a CSV</span>
      </el-text>
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
    <el-progress
      v-if="totalRows !== 0 && isFinite(currentRows / totalRows)"
      :percentage="Math.round((currentRows / totalRows) * 100)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Refresh, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { mapHeaders, viewOpenFile, viewSqlp } from "@/utils/view";
import { message } from "@/utils/message";

const [isLoading, isPath, tableHeader, tableColumn, tableData, path, mode] = [
  ref(false),
  ref(false),
  ref([]),
  ref([]),
  ref([]),
  ref(""),
  ref("memory")
];
const { dynamicHeight } = useDynamicHeight(178);

async function selectFile() {
  isPath.value = false;
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

// invoke transpopse
async function transposeData() {
  if (path.value === "") {
    message("File not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("transpose", {
      path: path.value,
      mode: mode.value
    });
    message(`Transpose done, elapsed time: ${rtime} s`, { type: "success" });
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
      </div>
      <el-text>
        <span v-if="isPath">
          <el-tooltip :content="path" effect="light">
            <span>{{ shortFileName(path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>Fill empty fields in selected columns of a CSV</span>
      </el-text>
    </div>
    <div class="custom-container1">
      <div clas="custom-container2" style="margin-top: 12px">
        <el-tooltip content="transpose mode" effect="light">
          <el-select v-model="mode" style="width: 110px">
            <el-option label="memory" value="memory" />
            <el-option label="multipass" value="multipass" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button
        style="margin-top: 12px"
        @click="transposeData()"
        :loading="isLoading"
        :icon="Refresh"
      >
        {{ mode }}
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
  </div>
</template>

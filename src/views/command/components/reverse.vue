<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, SwitchFilled } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";

const [isLoading, isPath, tableColumn, tableData, path, mode] = [
  ref(false),
  ref(false),
  ref([]),
  ref([]),
  ref(""),
  ref("reverse")
];
const { dynamicHeight } = useDynamicHeight(134);

async function selectFile() {
  isPath.value = false;
  tableColumn.value = [];
  tableData.value = [];

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) {
    return;
  }

  try {
    const { columnView, dataView } = await toJson(path.value);
    tableColumn.value = columnView;
    tableData.value = dataView;
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
}

// invoke reverse
async function reverseData() {
  if (path.value === "") {
    message("File not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("reverse", {
      path: path.value,
      mode: mode.value
    });
    message(`Reverse done, elapsed time: ${rtime} s`, { type: "success" });
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
        <el-tooltip content="add index or not" effect="light">
          <el-select v-model="mode" style="margin-left: 10px; width: 100px">
            <el-option label="Index" value="index" />
            <el-option label="Reverse" value="reverse" />
          </el-select>
        </el-tooltip>
        <el-button
          @click="reverseData()"
          :loading="isLoading"
          :icon="SwitchFilled"
          style="margin-left: 10px"
        >
          {{ mode }}
        </el-button>
      </div>

      <el-text>
        <span v-if="isPath">
          <el-tooltip :content="path" effect="light">
            <span>{{ shortFileName(path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>Reverses rows of CSV data</span>
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
  </div>
</template>

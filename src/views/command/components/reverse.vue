<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, SwitchFilled, Link } from "@element-plus/icons-vue";
import { useDynamicHeight } from "@/utils/utils";
import { viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { mdReverse, useMarkdown } from "@/utils/markdown";

const path = ref("");
const mode = ref("reverse");
const [tableColumn, tableData] = [ref([]), ref([])];
const [isLoading, dialog] = [ref(false), ref(false)];
const { dynamicHeight } = useDynamicHeight(153);
const { mdShow } = useMarkdown(mdReverse);

async function selectFile() {
  tableColumn.value = [];
  tableData.value = [];

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

  try {
    const { columnView, dataView } = await toJson(path.value);
    tableColumn.value = columnView;
    tableData.value = dataView;
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
        <el-tooltip content="add index or reverse" effect="light">
          <el-select v-model="mode" style="margin-left: 8px; width: 100px">
            <el-option label="Index" value="index" />
            <el-option label="Reverse" value="reverse" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button
        @click="reverseData()"
        :loading="isLoading"
        :icon="SwitchFilled"
      >
        Reverse
      </el-button>
    </div>
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
      <div class="custom-container2" />
      <el-link @click="dialog = true" :icon="Link">
        <el-tooltip :content="path" effect="light">
          <span>
            About
            <span style="color: skyblue; font-weight: bold">Reverse</span>
          </span>
        </el-tooltip>
      </el-link>
    </div>
    <el-dialog
      v-model="dialog"
      title="Reverse - Reverse order of rows in a CSV"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { IceCreamRound, FolderOpened, Link } from "@element-plus/icons-vue";
import { useDynamicHeight } from "@/utils/utils";
import { viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { listen } from "@tauri-apps/api/event";
import { enumerateContent, useMarkdown } from "@/utils/markdown";

const mode = ref("nil");
const path = ref("");
const [currentRows, totalRows] = [ref(0), ref(0)];
const [dialog, isLoading, isPath] = [ref(false), ref(false), ref(false)];
const [tableColumn, tableData] = [ref([]), ref([])];
const { dynamicHeight } = useDynamicHeight(155);
const { compiledMarkdown } = useMarkdown(enumerateContent);

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
        <el-tooltip content="if nil, no progress bar" effect="light">
          <el-select v-model="mode" style="margin-left: 10px; width: 70px">
            <el-option label="idx" value="idx" />
            <el-option label="nil" value="nil" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button
        @click="enumerate()"
        :loading="isLoading"
        :icon="IceCreamRound"
        style="margin-left: 10px"
      >
        Enumerate
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
    <div class="custom-container1">
      <div class="custom-container2">
        <el-progress
          v-if="totalRows !== 0 && isFinite(currentRows / totalRows)"
          :percentage="Math.round((currentRows / totalRows) * 100)"
          style="width: 75%"
        />
      </div>
      <el-tooltip effect="light" :content="path">
        <el-link @click="dialog = true" :icon="Link">
          <span>
            About
            <span style="color: skyblue; font-weight: bold">Enumerate</span>
          </span>
        </el-link>
      </el-tooltip>
    </div>
    <el-dialog
      v-model="dialog"
      title="Enumerate - Add a new column enumerating the lines of a CSV"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="compiledMarkdown" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

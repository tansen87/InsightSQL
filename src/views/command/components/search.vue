<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Search, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { toJson, viewOpenFile, mapHeaders } from "@/utils/view";
import { message } from "@/utils/message";
import { listen } from "@tauri-apps/api/event";

const [
  isLoading,
  isPath,
  columns,
  tableHeader,
  tableColumn,
  tableData,
  path,
  mode,
  condition,
  countMode,
  currentRows,
  totalRows
] = [
  ref(false),
  ref(false),
  ref(""),
  ref([]),
  ref([]),
  ref([]),
  ref(""),
  ref("equal"),
  ref(""),
  ref("nil"),
  ref(0),
  ref(0)
];
const { dynamicHeight } = useDynamicHeight(326);

listen("update-rows", (event: any) => {
  currentRows.value = event.payload;
});
listen("total-rows", (event: any) => {
  totalRows.value = event.payload;
});

async function selectFile() {
  isPath.value = false;
  columns.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];
  totalRows.value = 0;

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

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

// invoke search
async function searchData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (columns.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("search", {
      path: path.value,
      selectColumn: columns.value,
      mode: mode.value,
      condition: condition.value,
      countMode: countMode.value
    });
    message(`Search done, elapsed time: ${rtime} s`, { type: "success" });
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
      </div>
      <el-text>
        <span v-if="isPath">
          <el-tooltip :content="path" placement="top" effect="light">
            <span>{{ shortFileName(path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>Select fields matching rows</span>
      </el-text>
    </div>
    <div class="custom-container1">
      <div class="custom-container2" style="margin-top: 12px">
        <el-tooltip content="Search mode" effect="light">
          <el-select v-model="mode" style="width: 112px">
            <el-option label="Equal" value="equal" />
            <el-option label="EqualMulti" value="equalmulti" />
            <el-option label="Contains" value="contains" />
            <el-option label="ContainsMulti" value="containsmulti" />
            <el-option label="StartsWtih" value="startswith" />
            <el-option label="StartsWithMulti" value="startswithmulti" />
            <el-option label="Regex" value="regex" />
          </el-select>
        </el-tooltip>
        <el-select
          v-model="columns"
          filterable
          style="margin-left: 10px; width: 200px"
          placeholder="Search by column"
        >
          <el-option
            v-for="item in tableHeader"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
        <el-tooltip content="if nil, do not add progress bar" effect="light">
          <el-select v-model="countMode" style="margin-left: 10px; width: 70px">
            <el-option label="idx" value="idx" />
            <el-option label="std" value="std" />
            <el-option label="nil" value="nil" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button
        @click="searchData()"
        :loading="isLoading"
        :icon="Search"
        style="margin-top: 12px"
      >
        Search
      </el-button>
    </div>
    <div class="custom-container1" style="margin-top: 12px">
      <div style="flex: 7; width: 70%">
        <el-input
          v-model="condition"
          :autosize="{ minRows: 6, maxRows: 6 }"
          type="textarea"
          placeholder="Search rows with text...Example: tom|jack|jerry"
        />
      </div>
      <div style="flex: 0; margin-left: 10px">
        <el-progress
          type="circle"
          v-if="totalRows !== 0 && isFinite(currentRows / totalRows)"
          :percentage="Math.round((currentRows / totalRows) * 100)"
        />
      </div>
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

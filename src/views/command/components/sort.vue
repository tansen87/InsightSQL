<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Refresh } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";

const [
  isLoading,
  isPath,
  selectColumn,
  tableHeader,
  tableColumn,
  tableData,
  path,
  numeric,
  reverse,
  mode
] = [
  ref(false),
  ref(false),
  ref(""),
  ref([]),
  ref([]),
  ref([]),
  ref(""),
  ref(false),
  ref(false),
  ref("Sort")
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

// invoke sort
async function sortData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (selectColumn.value.length === 0 && mode.value !== "index") {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    let rtime: string;
    if (mode.value == "Sort") {
      rtime = await invoke("sort", {
        path: path.value,
        selectColumn: selectColumn.value,
        numeric: numeric.value,
        reverse: reverse.value
      });
    } else if (mode.value == "ExtSort") {
      rtime = await invoke("extsort", {
        path: path.value,
        selectColumn: selectColumn.value,
        reverse: reverse.value
      });
    } else if (mode.value == "Index") {
      rtime = await invoke("idx", {
        path: path.value
      });
    }
    message(`${mode.value} done, elapsed time: ${rtime} s`, {
      type: "success"
    });
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
        <span v-else>Sorts CSV data lexicographically</span>
      </el-text>
    </div>
    <div class="custom-container1">
      <div class="custom-container1" style="margin-top: 12px">
        <el-select
          v-model="selectColumn"
          filterable
          style="width: 200px"
          placeholder="Sort by column"
        >
          <el-option
            v-for="item in tableHeader"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
        <el-tooltip content="Sort, ExtSort or create index" effect="light">
          <el-select v-model="mode" style="margin-left: 10px; width: 90px">
            <el-option label="Sort" value="Sort" />
            <el-option label="ExtSort" value="ExtSort" />
            <el-option label="Index" value="Index" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="Numeric" effect="light">
          <el-select v-model="numeric" style="margin-left: 10px; width: 80px">
            <el-option label="true" :value="true" />
            <el-option label="false" :value="false" />
          </el-select>
        </el-tooltip>
        <el-tooltip
          content="Reverse (when set to false, sort from small to large)"
          effect="light"
        >
          <el-select v-model="reverse" style="margin-left: 10px; width: 80px">
            <el-option label="true" :value="true" />
            <el-option label="false" :value="false" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button
        @click="sortData()"
        :loading="isLoading"
        :icon="Refresh"
        style="margin-top: 10px"
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

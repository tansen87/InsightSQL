<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Search, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { toJson, viewOpenFile, mapHeaders } from "@/utils/view";
import { message } from "@/utils/message";
import { listen } from "@tauri-apps/api/event";

const [mode, countMode] = [ref("equal"), ref("nil")];
const [currentRows, totalRows, matchRows] = [ref(0), ref(0), ref(0)];
const [columns, path, condition] = [ref(""), ref(""), ref("")];
const [isLoading, isBtnShow] = [ref(false), ref(false)];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const { dynamicHeight } = useDynamicHeight(324);

listen("update-rows", (event: any) => {
  currentRows.value = event.payload;
});
listen("total-rows", (event: any) => {
  totalRows.value = event.payload;
});

async function selectFile() {
  isBtnShow.value = false;
  path.value = "";
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
    const res: string[] = await invoke("search", {
      path: path.value,
      selectColumn: columns.value,
      mode: mode.value,
      condition: condition.value,
      countMode: countMode.value
    });
    matchRows.value = Number(res[0]);
    isBtnShow.value = true;
    message(`Match ${res[0]}, elapsed time: ${res[1]} s`, { type: "success" });
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
        <el-select
          v-model="columns"
          filterable
          style="margin-left: 10px; margin-right: 10px; width: 140px"
          placeholder="Select column"
        >
          <el-option
            v-for="item in tableHeader"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
        <el-tooltip content="Search mode" effect="light">
          <el-select v-model="mode" filterable style="width: 140px">
            <el-option label="Equal" value="equal" />
            <el-option label="EqualMulti" value="equalmulti" />
            <el-option label="NotEqual" value="notequal" />
            <el-option label="Contains" value="contains" />
            <el-option label="ContainsMulti" value="containsmulti" />
            <el-option label="NotContains" value="notcontains" />
            <el-option label="StartsWith" value="startswith" />
            <el-option label="StartsWithMulti" value="startswithmulti" />
            <el-option label="NotStartsWtih" value="notstartswith" />
            <el-option label="EndsWith" value="endswith" />
            <el-option label="EndsWithMulti" value="endswithmulti" />
            <el-option label="NotEndsWith" value="notendswith" />
            <el-option label="Regex" value="regex" />
            <el-option label="IsNull" value="isnull" />
            <el-option label="IsNotNull" value="isnotnull" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="if nil, no progress bar" effect="light">
          <el-select v-model="countMode" style="margin-left: 10px; width: 70px">
            <el-option label="idx" value="idx" />
            <el-option label="std" value="std" />
            <el-option label="nil" value="nil" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button @click="searchData()" :loading="isLoading" :icon="Search">
        <el-text v-if="isBtnShow"> {{ matchRows }} rows filtered </el-text>
        <el-text v-else> Search </el-text>
      </el-button>
    </div>
    <div class="custom-container1" style="margin-top: 12px">
      <div style="flex: 7; width: 70%">
        <el-input
          v-model="condition"
          :autosize="{ minRows: 7, maxRows: 7 }"
          type="textarea"
          placeholder="Search conditions......Separate by |. (Example: tom|jack|jerry)"
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
    <div class="custom-container1">
      <div class="custom-container2">
        <el-text>Filter rows matching conditions</el-text>
      </div>
      <el-tooltip :content="path" effect="light">
        <el-text>{{ shortFileName(path) }}</el-text>
      </el-tooltip>
    </div>
  </div>
</template>

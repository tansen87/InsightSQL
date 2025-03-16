<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Search, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { viewSqlp, viewOpenFile, mapHeaders } from "@/utils/view";
import { message } from "@/utils/message";

const [
  isLoading,
  isPath,
  columns,
  tableHeader,
  tableColumn,
  tableData,
  searchBtn
] = [ref(false), ref(false), ref(""), ref([]), ref([]), ref([]), ref("Search")];
const data = reactive({
  path: "",
  mode: "equal",
  condition: "",
  skipRows: "0"
});
const { dynamicHeight } = useDynamicHeight(221);

async function selectFile() {
  isPath.value = false;
  columns.value = "";
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

// invoke search
async function searchData() {
  if (data.path === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (columns.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const result: string[] = await invoke("search", {
      path: data.path,
      selectColumn: columns.value,
      mode: data.mode,
      condition: data.condition,
      skipRows: data.skipRows
    });
    message(`Search done, elapsed time: ${result[1]} s`, { type: "success" });
    searchBtn.value = `${result[0]} rows`;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
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
        <el-tooltip content="skip rows" placement="top" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 50px"
          />
        </el-tooltip>
      </div>

      <el-text>
        <span v-if="isPath">
          <el-tooltip :content="data.path" placement="top" effect="light">
            <span>{{ shortFileName(data.path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>Select fields matching rows</span>
      </el-text>
    </div>

    <div class="custom-container1">
      <div class="custom-container2" style="margin-top: 12px">
        <el-tooltip content="Search mode" effect="light">
          <el-select v-model="data.mode" style="width: 112px">
            <el-option label="Equal" value="equal" />
            <el-option label="EqualMulti" value="equalmulti" />
            <el-option label="Contains" value="contains" />
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
      </div>

      <el-button
        @click="searchData()"
        :loading="isLoading"
        :icon="Search"
        style="margin-top: 12px"
      >
        {{ searchBtn }}
      </el-button>
    </div>

    <div style="margin-top: 12px">
      <el-input
        v-model="data.condition"
        autosize
        type="textarea"
        placeholder="Search rows with text...Example: tom|jack|world"
      />
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
  </div>
</template>

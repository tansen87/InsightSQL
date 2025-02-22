<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Search, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";
import { viewSqlp, viewOpenFile } from "@/utils/view";
import { message } from "@/utils/message";

const [isLoading, isPath, columns, tableHeader, tableColumn, tableData] = [
  ref(false),
  ref(false),
  ref(""),
  ref([]),
  ref([]),
  ref([])
];
const data = reactive({
  path: "",
  mode: "equal",
  condition: "",
  skipRows: "0"
});
const { formHeight } = useDynamicFormHeight(233);

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
    const { headerView, columnView, dataView } = await viewSqlp(
      data.path,
      data.skipRows
    );

    tableHeader.value = headerView;
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

    message(
      `Search done, match rows: 
        ${result[0]}
         lines, elapsed time: 
        ${result[1]} s`,
      { duration: 5000 }
    );
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
        <span v-if="isPath">{{ data.path }}</span>
        <span v-else>Select fields matching rows</span>
      </el-text>
    </div>

    <div class="custom-container1">
      <div class="custom-container2" style="margin-top: 12px">
        <el-tooltip content="Search mode" effect="light">
          <el-select v-model="data.mode" style="width: 112px">
            <el-option label="equal" value="equal" />
            <el-option label="contains" value="contains" />
            <el-option label="startswith" value="startswith" />
            <el-option label="regex" value="regex" />
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
        Search
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
      :height="formHeight"
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

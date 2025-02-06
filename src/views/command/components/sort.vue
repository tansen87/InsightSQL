<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Refresh } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";
import { viewOpenFile, viewSqlp } from "@/utils/view";
import { message } from "@/utils/message";

const [isLoading, isPath, selectColumn, tableHeader, tableColumn, tableData] = [
  ref(false),
  ref(false),
  ref(""),
  ref([]),
  ref([]),
  ref([])
];
const data = reactive({
  path: "",
  skipRows: "0",
  numeric: false,
  reverse: false
});
const { formHeight } = useDynamicFormHeight(190);

async function selectFile() {
  isPath.value = false;
  selectColumn.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

  data.path = await viewOpenFile(false, "csv", ["*"]);
  if (data.path === null) {
    return;
  }
  isPath.value = true;

  try {
    const { headerView, columnView, dataView } = await viewSqlp(
      data.path,
      data.skipRows
    );
    tableHeader.value = headerView;
    tableColumn.value = columnView;
    tableData.value = dataView;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
}

// invoke sort
async function sortData() {
  if (data.path === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (selectColumn.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  isLoading.value = true;

  try {
    const result: string = await invoke("sort", {
      path: data.path,
      skipRows: data.skipRows,
      selectColumn: selectColumn.value,
      numeric: data.numeric,
      reverse: data.reverse
    });

    message(`Sort done, elapsed time: ${result} s`, { duration: 5000 });
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
            style="margin-left: 10px; width: 78px"
          />
        </el-tooltip>
      </div>

      <el-text>
        <span v-if="isPath">{{ data.path }}</span>
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

        <el-tooltip content="Numeric" placement="top" effect="light">
          <el-select
            v-model="data.numeric"
            style="margin-left: 10px; width: 80px"
          >
            <el-option label="true" :value="true" />
            <el-option label="false" :value="false" />
          </el-select>
        </el-tooltip>

        <el-tooltip content="Reverse" placement="top" effect="light">
          <el-select
            v-model="data.reverse"
            style="margin-left: 10px; width: 80px"
          >
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
        Sort
      </el-button>
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

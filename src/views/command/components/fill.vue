<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Refresh, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { mapHeaders, viewOpenFile, viewSqlp } from "@/utils/view";
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
  value: "0",
  mode: "fill"
});
const { dynamicHeight } = useDynamicHeight(222);

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
    tableHeader.value = await mapHeaders(data.path, "0");
    const { columnView, dataView } = await viewSqlp(data.path, "0");
    tableColumn.value = columnView;
    tableData.value = dataView;
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
}

// invoke fill
async function fillData() {
  if (data.path === "") {
    message("File not selected", { type: "warning" });
    return;
  }
  if (columns.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const cols = Object.values(columns.value).join("|");
    const result: string = await invoke("fill", {
      path: data.path,
      columns: cols,
      values: data.value,
      mode: data.mode
    });
    message(`Fill done, elapsed time: ${result} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
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
          <el-tooltip :content="data.path" effect="light">
            <span>{{ shortFileName(data.path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>Fill empty fields in selected columns of a CSV</span>
      </el-text>
    </div>

    <el-select
      v-model="columns"
      multiple
      filterable
      style="margin-top: 12px; width: 100%"
      placeholder="Select the columns to be filled in"
    >
      <el-option
        v-for="item in tableHeader"
        :key="item.value"
        :label="item.label"
        :value="item.value"
      />
    </el-select>

    <div class="custom-container1">
      <div clas="custom-container2" style="margin-top: 12px">
        <el-tooltip content="The value of fill" effect="light">
          <el-input v-model="data.value" style="width: 120px" clearable />
        </el-tooltip>
        <el-tooltip content="fill mode" effect="light">
          <el-select v-model="data.mode" style="width: 80px; margin-left: 10px">
            <el-option label="fill" value="fill" />
            <el-option label="f-fill" value="ffill" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button
        style="margin-top: 12px"
        @click="fillData()"
        :loading="isLoading"
        :icon="Refresh"
      >
        Fill
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

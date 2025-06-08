<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Cherry, FolderOpened } from "@element-plus/icons-vue";
import { message } from "@/utils/message";
import { viewOpenFile, mapHeaders, toJson } from "@/utils/view";
import { CheckboxValueType } from "element-plus";
import { useDynamicHeight, shortFileName } from "@/utils/utils";

const [
  originalColumns,
  isLoading,
  isPath,
  checkAll,
  indeterminate,
  tableColumn,
  tableData,
  path
] = [
  ref([]),
  ref(false),
  ref(false),
  ref(false),
  ref(false),
  ref([]),
  ref([]),
  ref("")
];
const selColumns = ref<CheckboxValueType[]>([]);

watch(selColumns, val => {
  if (val.length === 0) {
    checkAll.value = false;
    indeterminate.value = false;
  } else if (val.length === originalColumns.value.length) {
    checkAll.value = true;
    indeterminate.value = false;
  } else {
    indeterminate.value = true;
  }
});
const { dynamicHeight } = useDynamicHeight(178);
const handleCheckAll = (val: CheckboxValueType) => {
  indeterminate.value = false;
  if (val) {
    selColumns.value = originalColumns.value.map(_ => _.value);
  } else {
    selColumns.value = [];
  }
};

async function selectFile() {
  originalColumns.value = [];
  isPath.value = false;
  selColumns.value = [];
  tableColumn.value = [];
  tableData.value = [];

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) {
    return;
  }

  try {
    originalColumns.value = await mapHeaders(path.value, "0");
    const { columnView, dataView } = await toJson(path.value);
    tableColumn.value = columnView;
    tableData.value = dataView;
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke select
async function selectColumns() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (selColumns.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const useCols = Object.values(selColumns.value).join("|");
    const rtime: string = await invoke("select", {
      path: path.value,
      cols: useCols
    });
    message(`Select done, elapsed time: ${rtime} s`, { type: "success" });
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
        <el-button
          @click="selectColumns()"
          :loading="isLoading"
          :icon="Cherry"
          style="margin-left: 10px"
        >
          Select
        </el-button>
      </div>
      <el-text>
        <span v-if="isPath">
          <el-tooltip :content="path" effect="light">
            <span>{{ shortFileName(path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>Select, re-order columns</span>
      </el-text>
    </div>
    <el-select
      v-model="selColumns"
      multiple
      filterable
      style="margin-top: 12px; width: 100%"
      placeholder="Select columns"
    >
      <template #header>
        <el-checkbox
          v-model="checkAll"
          :indeterminate="indeterminate"
          @change="handleCheckAll"
        >
          All
        </el-checkbox>
      </template>
      <el-option
        v-for="item in originalColumns"
        :key="item.value"
        :label="item.label"
        :value="item.value"
      />
    </el-select>
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

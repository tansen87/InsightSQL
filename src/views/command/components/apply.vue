<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  ArrowRight,
  FolderOpened,
  Link,
  CirclePlus,
  Remove
} from "@element-plus/icons-vue";
import { useDynamicHeight } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { CheckboxValueType } from "element-plus";
import { mdApply, useMarkdown } from "@/utils/markdown";

const [
  isLoading,
  checkAll,
  indeterminate,
  newColumn,
  dialog,
  backendCompleted
] = [ref(false), ref(false), ref(false), ref(false), ref(false), ref(false)];
const [operations, tableHeader, tableColumn, tableData] = [
  ref([]),
  ref([]),
  ref([]),
  ref([])
];
const [path, comparand, replacement, formatstr, backendInfo] = [
  ref(""),
  ref(""),
  ref(""),
  ref(""),
  ref("")
];
const mode = ref("operations");
const columnContent = ref("no column");
const columns = ref<CheckboxValueType[]>([]);
const { dynamicHeight } = useDynamicHeight(256);
watch(columns, val => {
  if (val.length === 0) {
    checkAll.value = false;
    indeterminate.value = false;
  } else if (val.length === tableHeader.value.length) {
    checkAll.value = true;
    indeterminate.value = false;
  } else {
    indeterminate.value = true;
  }
});
const handleCheckAll = (val: CheckboxValueType) => {
  indeterminate.value = false;
  if (val) {
    columns.value = tableHeader.value.map(_ => _.value);
  } else {
    columns.value = [];
  }
};
const { mdShow } = useMarkdown(mdApply);

async function selectFile() {
  columns.value = [];
  operations.value = [];
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

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

// invoke apply
async function applyData() {
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
    const result: string = await invoke("apply", {
      path: path.value,
      columns: Object.values(columns.value).join("|"),
      mode: mode.value,
      operations: operations.value.join("|"),
      comparand: comparand.value,
      replacement: replacement.value,
      formatstr: formatstr.value,
      newColumn: newColumn.value
    });
    message(`Apply done, elapsed time: ${result} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}

function addNewColumn() {
  newColumn.value = !newColumn.value;
  if (newColumn.value === true) {
    columnContent.value = "add column";
  } else {
    columnContent.value = "no column";
  }
}
</script>

<template>
  <div class="page-container">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-tabs v-model="mode">
          <el-tab-pane name="operations" label="Operations" />
          <el-tab-pane name="calcconv" label="CalcConv" />
          <el-tab-pane name="cat" label="Concat" />
        </el-tabs>
      </div>
      <el-link @click="dialog = true" :icon="Link">
        <span v-if="backendCompleted"> {{ backendInfo }} </span>
        <el-tooltip :content="path" effect="light">
          <span>
            About
            <span style="color: skyblue; font-weight: bold">Apply</span>
          </span>
        </el-tooltip>
      </el-link>
    </div>
    <div class="custom-container1">
      <div class="custom-container2">
        <el-tooltip content="Add data" effect="light">
          <el-button @click="selectFile()" :icon="FolderOpened" text circle />
        </el-tooltip>
        <el-tooltip :content="columnContent" effect="light">
          <el-button @click="addNewColumn" text circle>
            <el-icon>
              <CirclePlus v-if="newColumn" />
              <Remove v-else />
            </el-icon>
          </el-button>
        </el-tooltip>
      </div>
      <el-tooltip content="Run" effect="light">
        <el-button
          @click="applyData()"
          :loading="isLoading"
          :icon="ArrowRight"
          text
          circle
        />
      </el-tooltip>
    </div>

    <div class="custom-container1">
      <el-select
        v-model="columns"
        filterable
        multiple
        placeholder="Select column(s)"
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
          v-for="item in tableHeader"
          :key="item.value"
          :label="item.label"
          :value="item.value"
        />
      </el-select>
      <el-select
        v-if="mode === 'operations'"
        v-model="operations"
        filterable
        multiple
        placeholder="Operations"
        style="margin-left: 5px"
      >
        <el-option label="Copy" value="copy" />
        <el-option label="Len" value="len" />
        <el-option label="Lower" value="lower" />
        <el-option label="Upper" value="upper" />
        <el-option label="Trim" value="trim" />
        <el-option label="Ltrim" value="ltrim" />
        <el-option label="Rtrim" value="rtrim" />
        <el-option label="Replace" value="replace" />
        <el-option label="Round" value="round" />
        <el-option label="Squeeze" value="squeeze" />
        <el-option label="Strip" value="strip" />
        <el-option label="Reverse" value="reverse" />
        <el-option label="Abs" value="abs" />
        <el-option label="Neg" value="neg" />
      </el-select>
    </div>

    <div class="custom-container1">
      <template v-if="['operations'].includes(mode)">
        <el-tooltip content="replace - from" effect="light">
          <el-input
            v-model="comparand"
            style="margin-top: 8px"
            placeholder="replace - from"
          />
        </el-tooltip>
        <el-tooltip content="replace - to" effect="light">
          <el-input
            v-model="replacement"
            style="margin-left: 5px; margin-top: 8px"
            placeholder="replace - to"
          />
        </el-tooltip>
      </template>
      <div
        v-if="['cat', 'calcconv'].includes(mode)"
        style="width: 100%; margin-top: 8px"
      >
        <el-tooltip content="formatstr with CalcConv or Cat" effect="light">
          <el-input v-model="formatstr" placeholder="{col1} + {col2}" />
        </el-tooltip>
      </div>
    </div>

    <el-table
      :data="tableData"
      :height="dynamicHeight"
      border
      empty-text=""
      style="margin-top: 8px"
      show-overflow-tooltip
    >
      <el-table-column
        v-for="column in tableColumn"
        :prop="column.prop"
        :label="column.label"
        :key="column.prop"
      />
    </el-table>

    <el-dialog
      v-model="dialog"
      title="Apply - Apply a series of transformation functions to given CSV column/s"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

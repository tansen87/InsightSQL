<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { CheckboxValueType } from "element-plus";
import { FolderOpened, Select, Link } from "@element-plus/icons-vue";
import { message } from "@/utils/message";
import { viewOpenFile, mapHeaders, toJson } from "@/utils/view";
import { useDynamicHeight } from "@/utils/utils";
import { selectContent, useMarkdown } from "@/utils/markdown";

const path = ref("");
const [currentRows, totalRows] = [ref(0), ref(0)];
const [pgsMode, selMode] = [ref("nil"), ref("include")];
const [originalColumns, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const [isLoading, dialog, checkAll, indeterminate] = [
  ref(false),
  ref(false),
  ref(false),
  ref(false)
];
const { dynamicHeight } = useDynamicHeight(199);
const { compiledMarkdown } = useMarkdown(selectContent);
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

const handleCheckAll = (val: CheckboxValueType) => {
  indeterminate.value = false;
  if (val) {
    selColumns.value = originalColumns.value.map(_ => _.value);
  } else {
    selColumns.value = [];
  }
};

listen("update-rows", (event: any) => {
  currentRows.value = event.payload;
});
listen("total-rows", (event: any) => {
  totalRows.value = event.payload;
});

async function selectFile() {
  originalColumns.value = [];
  path.value = "";
  selColumns.value = [];
  tableColumn.value = [];
  tableData.value = [];
  totalRows.value = 0;

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

  try {
    originalColumns.value = await mapHeaders(path.value, "0");
    const { columnView, dataView } = await toJson(path.value);
    tableColumn.value = columnView;
    tableData.value = dataView;
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
    const selCols = Object.values(selColumns.value).join("|");
    const rtime: string = await invoke("select", {
      path: path.value,
      selCols: selCols,
      selMode: selMode.value,
      pgsMode: pgsMode.value
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
        <el-tooltip content="Select mode" effect="light">
          <el-select v-model="selMode" style="width: 95px; margin-left: 10px">
            <el-option label="Include" value="include" />
            <el-option label="Exclude" value="exclude" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="if nil, no progress bar" effect="light">
          <el-select v-model="pgsMode" style="margin-left: 10px; width: 70px">
            <el-option label="idx" value="idx" />
            <el-option label="nil" value="nil" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button
        @click="selectColumns()"
        :loading="isLoading"
        :icon="Select"
        style="margin-left: 10px"
      >
        Select
      </el-button>
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
    <div class="custom-container1">
      <div class="custom-container2">
        <el-progress
          v-if="totalRows !== 0 && isFinite(currentRows / totalRows)"
          :percentage="Math.round((currentRows / totalRows) * 100)"
          style="width: 75%"
        />
      </div>
      <el-link @click="dialog = true" :icon="Link">
        <el-tooltip :content="path" effect="light">
          <span>
            About
            <span style="color: skyblue; font-weight: bold">Select</span>
          </span>
        </el-tooltip>
      </el-link>
    </div>
    <el-dialog
      v-model="dialog"
      title="Select - Select, drop, re-order columns"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="compiledMarkdown" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

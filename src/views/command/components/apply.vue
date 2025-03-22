<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Refresh, FolderOpened, Link } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { mapHeaders, viewOpenFile, viewSqlp } from "@/utils/view";
import { message } from "@/utils/message";
import { CheckboxValueType } from "element-plus";
import { applyContent, useMarkdown } from "@/utils/markdown";

const [
  isLoading,
  isPath,
  operations,
  tableHeader,
  tableColumn,
  tableData,
  checkAll,
  indeterminate,
  path,
  applyMode,
  comparand,
  replacement,
  formatstr,
  newColumn,
  infoDialog,
  backendCompleted,
  backendInfo
] = [
  ref(false),
  ref(false),
  ref([]),
  ref([]),
  ref([]),
  ref([]),
  ref(false),
  ref(false),
  ref(""),
  ref("Operations"),
  ref(""),
  ref(""),
  ref(""),
  ref(false),
  ref(false),
  ref(false),
  ref("")
];
const selColumns = ref<CheckboxValueType[]>([]);
const { dynamicHeight } = useDynamicHeight(266);
watch(selColumns, val => {
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
    selColumns.value = tableHeader.value.map(_ => _.value);
  } else {
    selColumns.value = [];
  }
};
const { compiledMarkdown } = useMarkdown(applyContent);

async function selectFile() {
  isPath.value = false;
  selColumns.value = [];
  operations.value = [];
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) {
    return;
  }

  try {
    tableHeader.value = await mapHeaders(path.value, "0");
    const { columnView, dataView } = await viewSqlp(path.value, "0");
    tableColumn.value = columnView;
    tableData.value = dataView;
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke apply
async function applyData() {
  console.log(applyContent());
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
    const result: string = await invoke("apply", {
      path: path.value,
      selectColumns: Object.values(selColumns.value).join("|"),
      applyMode: applyMode.value,
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
</script>

<template>
  <div class="page-container">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
      </div>
      <el-link @click="infoDialog = true" :icon="Link">
        <span v-if="backendCompleted"> {{ backendInfo }} </span>
        <span v-if="isPath">
          <el-tooltip :content="path" effect="light">
            <span>{{ shortFileName(path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>
          About
          <span style="color: skyblue; font-weight: bold">Apply</span>
        </span>
      </el-link>
    </div>

    <el-select
      v-model="selColumns"
      filterable
      multiple
      placeholder="Apply by column(s)"
      style="width: 100%; margin-top: 12px"
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
      v-model="operations"
      filterable
      multiple
      placeholder="Operations"
      style="margin-top: 12px; width: 100%"
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
    </el-select>

    <div class="custom-container1">
      <div style="width: 90%; display: flex; align-items: center">
        <div style="flex: 1; margin-top: 12px">
          <el-tooltip content="apply mode" effect="light">
            <el-select v-model="applyMode" style="width: 100%">
              <el-option label="Operations" value="operations" />
              <el-option label="CalcConv" value="calcconv" />
              <el-option label="DynFmt" value="dynfmt" />
            </el-select>
          </el-tooltip>
        </div>

        <div style="flex: 1; margin-left: 5px; margin-top: 12px">
          <el-tooltip content="replace - from" effect="light">
            <el-input
              v-model="comparand"
              style="width: 100%"
              placeholder="replace - from"
              clearable
            />
          </el-tooltip>
        </div>

        <div style="flex: 1; margin-left: 5px; margin-top: 12px">
          <el-tooltip content="replace - to" effect="light">
            <el-input
              v-model="replacement"
              style="width: 100%"
              placeholder="replace - to"
              clearable
            />
          </el-tooltip>
        </div>

        <div style="flex: 3; margin-left: 5px; margin-top: 12px">
          <el-tooltip
            content="formatstr with CalcConv or DynFmt"
            effect="light"
          >
            <el-input
              v-model="formatstr"
              style="width: 100%"
              placeholder="{col1} + {col2}"
              clearable
            />
          </el-tooltip>
        </div>

        <div style="flex: 1; margin-left: 5px">
          <el-switch
            v-model="newColumn"
            class="ml-2"
            inline-prompt
            style="
              --el-switch-on-color: #43cd80;
              --el-switch-off-color: #b0c4de;
              width: 100%;
              margin-top: 12px;
            "
            active-text="column"
            inactive-text="no column"
          />
        </div>
      </div>

      <div style="width: 10%; text-align: right">
        <el-button
          @click="applyData()"
          :loading="isLoading"
          :icon="Refresh"
          style="margin-top: 12px; width: 100%"
        >
          Apply
        </el-button>
      </div>
    </div>

    <div class="custom-container1">
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

    <el-dialog
      v-model="infoDialog"
      title="Apply - Apply a series of transformation functions to given CSV column/s"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="compiledMarkdown" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

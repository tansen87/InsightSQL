<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Refresh, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { mapHeaders, viewOpenFile, viewSqlp } from "@/utils/view";
import { message } from "@/utils/message";

const [
  isLoading,
  isPath,
  selectColumns,
  operations,
  tableHeader,
  tableColumn,
  tableData
] = [ref(false), ref(false), ref([]), ref([]), ref([]), ref([]), ref([])];
const data = reactive({
  path: "",
  applyMode: "Operations",
  comparand: "",
  replacement: "",
  formatstr: "",
  newColumn: false,
  skipRows: "0"
});
const { dynamicHeight } = useDynamicHeight(278);

async function selectFile() {
  isPath.value = false;
  selectColumns.value = [];
  operations.value = [];
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

// invoke apply
async function applyData() {
  if (data.path === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (selectColumns.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const result: string = await invoke("apply", {
      path: data.path,
      selectColumns: selectColumns.value.join("|"),
      applyMode: data.applyMode,
      operations: operations.value.join("|"),
      comparand: data.comparand,
      replacement: data.replacement,
      formatstr: data.formatstr,
      newColumn: data.newColumn,
      skipRows: data.skipRows
    });

    message(`Apply done, elapsed time: ${result} s`);
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
        <el-tooltip content="skip rows" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 50px"
          />
        </el-tooltip>
      </div>

      <el-text>
        <span v-if="isPath">
          <el-tooltip :content="data.path" effect="light">
            <span>{{ shortFileName(data.path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>
          Apply a series of transformation functions to given CSV column/s
        </span>
      </el-text>
    </div>

    <el-select
      v-model="selectColumns"
      filterable
      multiple
      placeholder="Apply by column(s)"
      style="width: 100%; margin-top: 12px"
    >
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
    </el-select>

    <div class="custom-container1">
      <div style="width: 90%; display: flex; align-items: center">
        <div style="flex: 1; margin-top: 12px">
          <el-tooltip content="apply mode" effect="light">
            <el-select v-model="data.applyMode" style="width: 100%">
              <el-option label="Operations" value="operations" />
              <el-option label="CalcConv" value="calcconv" />
              <el-option label="DynFmt" value="dynfmt" />
            </el-select>
          </el-tooltip>
        </div>

        <div style="flex: 1; margin-left: 5px; margin-top: 12px">
          <el-tooltip content="replace - from" effect="light">
            <el-input
              v-model="data.comparand"
              style="width: 100%"
              placeholder="replace - from"
              clearable
            />
          </el-tooltip>
        </div>

        <div style="flex: 1; margin-left: 5px; margin-top: 12px">
          <el-tooltip content="replace - to" effect="light">
            <el-input
              v-model="data.replacement"
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
              v-model="data.formatstr"
              style="width: 100%"
              placeholder="{col1} + {col2}"
              clearable
            />
          </el-tooltip>
        </div>

        <div style="flex: 1; margin-left: 5px">
          <el-switch
            v-model="data.newColumn"
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
      >
        <el-table-column
          v-for="column in tableColumn"
          :prop="column.prop"
          :label="column.label"
          :key="column.prop"
        />
      </el-table>
    </div>
  </div>
</template>

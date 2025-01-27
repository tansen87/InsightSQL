<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { Refresh, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";

const isLoading = ref(false);
const isPath = ref(false);
const selectColumns = ref([]);
const operations = ref([]);
const originalColumns = ref([]);
const data = reactive({
  path: "",
  fileFormats: ["*"],
  applyMode: "Operations",
  comparand: "",
  replacement: "",
  formatstr: "",
  newColumn: false,
  skipRows: "0"
});
const tableColumn = ref([]);
const tableData = ref([]);
const tableRef = ref(null);
const { formHeight } = useDynamicFormHeight(278);

async function selectFile() {
  isLoading.value = false;
  isPath.value = false;
  originalColumns.value = [];
  selectColumns.value = [];

  const selected = await open({
    multiple: false,
    filters: [
      {
        name: "csv",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.path = selected.toString();
  } else if (selected === null) {
    return;
  } else {
    data.path = selected;
  }
  isPath.value = true;

  try {
    const header: string[] = await invoke("get_apply_headers", {
      path: data.path,
      skipRows: data.skipRows
    });
    originalColumns.value = header;

    const result: string = await invoke("query", {
      path: data.path,
      sqlQuery: "select * from _t_1 limit 10",
      write: false,
      writeFormat: "csv",
      lowMemory: false,
      skipRows: data.skipRows
    });

    if (
      result[0].startsWith("execute_query") ||
      result[0].startsWith("prepare_query")
    ) {
      throw result[0].toString();
    }

    const jsonData = JSON.parse(result);
    const isJsonArray = Array.isArray(jsonData);
    const arrayData = isJsonArray ? jsonData : [jsonData];
    tableColumn.value = Object.keys(arrayData[0]).map(key => ({
      name: key,
      label: key,
      prop: key
    }));
    tableData.value = arrayData;
  } catch (err) {
    ElNotification({
      title: "Open file error",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
}

// apply function
async function applyData() {
  if (data.path === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }
  if (selectColumns.value.length === 0) {
    ElNotification({
      title: "Column not found",
      message: "未选择column",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  isLoading.value = true;

  try {
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

    if (JSON.stringify(result).startsWith("apply failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Apply done, elapsed time: ${result} s.`,
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
  } catch (err) {
    ElNotification({
      title: "Invoke Apply Error",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
  isLoading.value = false;
}
</script>

<template>
  <div class="page-container">
    <div
      style="
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        position: sticky;
      "
    >
      <div style="display: flex; align-items: flex-start">
        <el-button @click="selectFile()" :icon="FolderOpened" plain>
          Open File
        </el-button>
        <el-tooltip content="skip rows" placement="top" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 50px"
            placeholder="skip rows"
          />
        </el-tooltip>
      </div>

      <el-text>
        <span v-if="isPath">{{ data.path }}</span>
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
        v-for="item in originalColumns"
        :key="item.value"
        :label="item.label"
        :value="item.value"
      />
    </el-select>
    <el-select
      v-model="operations"
      filterable
      multiple
      placeholder="operations"
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
    <div
      style="
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        position: sticky;
      "
    >
      <div style="width: 90%; display: flex; align-items: center">
        <!-- 这里的内容保持不变 -->
        <div style="flex: 1; margin-top: 12px">
          <el-tooltip content="apply mode" placement="bottom" effect="light">
            <el-select v-model="data.applyMode" style="width: 100%">
              <el-option label="Operations" value="operations" />
              <el-option label="CalcConv" value="calcconv" />
              <el-option label="DynFmt" value="dynfmt" />
            </el-select>
          </el-tooltip>
        </div>
        <div style="flex: 1; margin-left: 5px; margin-top: 12px">
          <el-tooltip
            content="replace - from"
            placement="bottom"
            effect="light"
          >
            <el-input
              v-model="data.comparand"
              style="width: 100%"
              placeholder="replace - from"
              clearable
            />
          </el-tooltip>
        </div>
        <div style="flex: 1; margin-left: 5px; margin-top: 12px">
          <el-tooltip content="replace - to" placement="bottom" effect="light">
            <el-input
              v-model="data.replacement"
              style="width: 100%"
              placeholder="replace - to"
              clearable
            />
          </el-tooltip>
        </div>
        <div style="flex: 3; margin-left: 5px; margin-top: 12px">
          <el-tooltip content="formatstr" placement="bottom" effect="light">
            <el-input
              v-model="data.formatstr"
              style="width: 100%"
              placeholder="formatstr"
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
          plain
          style="margin-top: 12px; width: 100%"
        >
          Apply
        </el-button>
      </div>
    </div>
    <div
      style="
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        position: sticky;
      "
    >
      <el-table
        ref="tableRef"
        :data="tableData"
        :height="formHeight"
        border
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

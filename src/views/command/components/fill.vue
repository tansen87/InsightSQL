<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { Refresh, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";

const isLoading = ref(false);
const isPath = ref(false);
const columns = ref("");
const originalColumns = ref([]);
const tableColumn = ref([]);
const tableData = ref([]);
const tableRef = ref(null);
const data = reactive({
  path: "",
  fileFormats: ["*"],
  value: "0",
  skipRows: "0"
});
const { formHeight } = useDynamicFormHeight(234);

async function selectFile() {
  isLoading.value = false;
  isPath.value = false;

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
    const headers: string[] = await invoke("get_fill_headers", {
      path: data.path,
      skipRows: data.skipRows
    });
    if (JSON.stringify(headers).startsWith("get header error:")) {
      throw JSON.stringify(headers).toString();
    }
    originalColumns.value = headers;

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

// invoke fill
async function fillData() {
  if (data.path === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }
  if (columns.value.length === 0) {
    ElNotification({
      title: "Column not found",
      message: "未选择columns",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  const cols = Object.values(columns.value).join("|");

  isLoading.value = true;

  try {
    const result: string = await invoke("fill", {
      path: data.path,
      columns: cols,
      values: data.value,
      skipRows: data.skipRows
    });

    if (JSON.stringify(result).startsWith("fill failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Fill done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
  } catch (err) {
    ElNotification({
      title: "Fill failed",
      message: err.match(/fill failed: (.*)/)[1].toString(),
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
    <div class="custom-container1">
      <div clas="custom-container2">
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
        <span v-else>Fill empty fields in selected columns of a CSV</span>
      </el-text>
    </div>

    <el-select
      v-model="columns"
      multiple
      filterable
      style="margin-top: 12px; width: 100%"
      placeholder="please choose columns"
    >
      <el-option
        v-for="item in originalColumns"
        :key="item.value"
        :label="item.label"
        :value="item.value"
      />
    </el-select>
    <div class="custom-container1">
      <div clas="custom-container2" style="margin-top: 12px">
        <el-tooltip
          content="The value of fill"
          placement="bottom"
          effect="light"
        >
          <el-input v-model="data.value" style="width: 120px" clearable />
        </el-tooltip>
      </div>
      <el-button
        style="margin-top: 12px"
        @click="fillData()"
        :loading="isLoading"
        :icon="Refresh"
        plain
      >
        Fill
      </el-button>
    </div>

    <div class="custom-container1">
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

<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { FolderOpened, Refresh } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";

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
  fileFormats: ["*"],
  regexPattern: "",
  replacement: "",
  skipRows: "0"
});
const { formHeight } = useDynamicFormHeight(234);

async function selectFile() {
  isPath.value = false;
  selectColumn.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];
  data.regexPattern = "";
  data.replacement = "";

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
    const arrayData = Array.isArray(jsonData) ? jsonData : [jsonData];
    tableHeader.value = Object.keys(arrayData[0]).map(header => ({
      label: header,
      value: header
    }));
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

// invoke replace
async function replaceData() {
  if (data.path === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }
  if (selectColumn.value.length === 0) {
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
    const result: string = await invoke("replace", {
      path: data.path,
      selectColumn: selectColumn.value,
      regexPattern: data.regexPattern,
      replacement: data.replacement,
      skipRows: data.skipRows
    });

    if (JSON.stringify(result).startsWith("replace failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Replace done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
  } catch (err) {
    ElNotification({
      title: "Replace failed",
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
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened" plain>
          Open File
        </el-button>

        <el-tooltip content="skip rows" placement="top" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 78px"
            placeholder="skip rows"
          />
        </el-tooltip>
      </div>

      <el-text>
        <span v-if="isPath">{{ data.path }}</span>
        <span v-else>Replace occurrences of a pattern across a CSV file</span>
      </el-text>
    </div>

    <div class="custom-container1">
      <div class="custom-container1" style="margin-top: 12px">
        <el-select
          v-model="selectColumn"
          filterable
          style="width: 200px"
          placeholder="Replace by column"
        >
          <el-option
            v-for="item in tableHeader"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>

        <el-input
          style="margin-left: 10px; width: 200px"
          placeholder="regex pattern"
          v-model="data.regexPattern"
          clearable
        />
      </div>

      <el-button
        @click="replaceData()"
        :loading="isLoading"
        :icon="Refresh"
        plain
        style="margin-top: 10px"
      >
        Replace
      </el-button>
    </div>

    <div style="margin-top: 12px">
      <el-input
        v-model="data.replacement"
        autosize
        clearable
        placeholder="Replacement string - Replace with any string"
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

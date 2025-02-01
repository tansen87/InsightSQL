<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { Search, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";

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
  fileFormats: ["*"],
  mode: "equal",
  condition: "",
  skipRows: "0"
});
const { formHeight } = useDynamicFormHeight(233);

async function selectFile() {
  isPath.value = false;
  columns.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

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
    const result: string[] = await invoke("query", {
      path: data.path,
      sqlQuery: "select * from _t_1 limit 10",
      write: false,
      writeFormat: "csv",
      lowMemory: false,
      skipRows: data.skipRows
    });

    const q = Array.isArray(result[0]) ? result[0][0] : null;
    if (q.startsWith("Query failed")) {
      throw q;
    }

    const jsonData = JSON.parse(result[0]);
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

// invoke search
async function searchData() {
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
      message: "未选择column",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  isLoading.value = true;

  try {
    const result: string[] = await invoke("search", {
      path: data.path,
      selectColumn: columns.value,
      mode: data.mode,
      condition: data.condition,
      skipRows: data.skipRows
    });

    if (JSON.stringify(result).startsWith("search failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Search done, match rows: 
        ${result[0]}
         lines, elapsed time: 
        ${result[1]} s`,
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
  } catch (err) {
    ElNotification({
      title: "Search failed",
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
            style="margin-left: 10px; width: 50px"
            placeholder="skip rows"
          />
        </el-tooltip>
      </div>

      <el-text>
        <span v-if="isPath">{{ data.path }}</span>
        <span v-else>Select fields matching rows</span>
      </el-text>
    </div>

    <div class="custom-container1">
      <div class="custom-container2" style="margin-top: 12px">
        <el-tooltip content="Search mode" placement="bottom" effect="light">
          <el-select v-model="data.mode" style="width: 112px">
            <el-option label="equal" value="equal" />
            <el-option label="contains" value="contains" />
            <el-option label="startswith" value="startswith" />
            <el-option label="regex" value="regex" />
          </el-select>
        </el-tooltip>

        <el-select
          v-model="columns"
          filterable
          style="margin-left: 10px; width: 200px"
          placeholder="Search by column"
        >
          <el-option
            v-for="item in tableHeader"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
      </div>

      <el-button
        @click="searchData()"
        :loading="isLoading"
        :icon="Search"
        plain
        style="margin-top: 12px"
      >
        Search
      </el-button>
    </div>

    <div style="margin-top: 12px">
      <el-input
        v-model="data.condition"
        autosize
        type="textarea"
        placeholder="Search rows with text...Example: tom|jack|world"
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

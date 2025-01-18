<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { IceCreamRound, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";

const isLoading = ref(false);
const columns = ref([]);
const isPath = ref(false);
const tableData = ref([]);
const tableRef = ref(null);
const data = reactive({
  filePath: "",
  fileFormats: ["*"]
});
const { formHeight } = useDynamicFormHeight(220);

async function selectFile() {
  isLoading.value = false;
  isPath.value = false;
  columns.value = [];
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
    data.filePath = selected.toString();
  } else if (selected === null) {
    return;
  } else {
    data.filePath = selected;
  }
  isPath.value = true;

  const df: string = await invoke("query", {
    path: data.filePath,
    sqlQuery: "select * from _t_1 limit 5",
    write: false,
    writeFormat: "csv",
    lowMemory: false,
    skipRows: "0"
  });

  const jsonData = JSON.parse(df);
  const isJsonArray = Array.isArray(jsonData);
  const arrayData = isJsonArray ? jsonData : [jsonData];
  columns.value = Object.keys(arrayData[0]).map(key => ({
    name: key,
    label: key,
    prop: key
  }));
  tableData.value = arrayData;
}

// invoke enumer function
async function enumerate() {
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  isLoading.value = true;

  try {
    const result: string = await invoke("enumer", {
      filePath: data.filePath
    });

    if (JSON.stringify(result).startsWith("enumerate failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Enumerate done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 5000
    });
  } catch (err) {
    ElNotification({
      title: "Invoke Enumerate Error",
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
        <el-button
          type="primary"
          @click="selectFile()"
          :icon="FolderOpened"
          plain
        >
          Open File
        </el-button>
        <el-button
          type="success"
          @click="enumerate()"
          :loading="isLoading"
          :icon="IceCreamRound"
          plain
          style="margin-left: 16px"
        >
          Enumerate
        </el-button>
      </div>
      <el-text type="primary" size="large">
        <span v-if="isPath">{{ data.filePath }}</span>
        <span v-else>Add an index for a CSV</span>
      </el-text>
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
        style="margin-top: 15px; width: 100%"
      >
        <el-table-column
          v-for="column in columns"
          :prop="column.prop"
          :label="column.label"
          :key="column.prop"
        />
      </el-table>
    </div>
  </div>
</template>

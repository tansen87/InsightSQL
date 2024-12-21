<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { Refresh, FolderOpened } from "@element-plus/icons-vue";

const isLoading = ref(false);
const isPath = ref(false);
const selectColumn = ref("");
const originalColumns = ref([]);
const data = reactive({
  filePath: "",
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"],
  regexPattern: "",
  replacement: ""
});
const tableColumn = ref([]);
const tableData = ref([]);
const tableRef = ref(null);
const windowHeight = ref(window.innerHeight);
const formHeight = computed(() => {
  const height = 278;
  return windowHeight.value - height;
});
const updateWindowHeight = () => {
  windowHeight.value = window.innerHeight;
};
onMounted(() => {
  window.addEventListener("resize", updateWindowHeight);
});
onBeforeUnmount(() => {
  window.removeEventListener("resize", updateWindowHeight);
});

async function selectFile() {
  isLoading.value = false;
  isPath.value = false;
  originalColumns.value = [];
  selectColumn.value = "";

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

  try {
    const header: any = await invoke("get_replace_headers", {
      filePath: data.filePath
    });
    originalColumns.value = header;

    const result: string = await invoke("query", {
      path: data.filePath,
      sqlQuery: "select * from _t_1 limit 10",
      write: false,
      writeFormat: "csv",
      lowMemory: false,
      skipRows: "0"
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

// invoke replace
async function replaceData() {
  if (data.filePath === "") {
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

  if (data.filePath !== "") {
    isLoading.value = true;

    try {
      const result: string = await invoke("replace", {
        filePath: data.filePath,
        selectColumn: selectColumn.value,
        regexPattern: data.regexPattern,
        replacement: data.replacement
      });

      if (JSON.stringify(result).startsWith("Replace failed:")) {
        throw JSON.stringify(result).toString();
      }

      isLoading.value = false;
      ElNotification({
        message: "Replace done, " + "elapsed time: " + result + " s.",
        position: "bottom-right",
        type: "success",
        duration: 10000
      });
    } catch (err) {
      ElNotification({
        title: "Invoke Replace Error",
        message: err.toString(),
        position: "bottom-right",
        type: "error",
        duration: 10000
      });
    }

    isLoading.value = false;
  }
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
      </div>

      <el-text type="primary" size="large">
        <span v-if="isPath">{{ data.filePath }}</span>
        <span v-else>Replace occurrences of a pattern across a CSV file</span>
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
      <div style="margin-top: 15px; display: flex; align-items: flex-start">
        <el-select
          v-model="selectColumn"
          filterable
          style="width: 200px"
          placeholder="Replace by column"
        >
          <el-option
            v-for="item in originalColumns"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
        <el-input
          style="margin-left: 15px; width: 200px"
          placeholder="regex pattern"
          v-model="data.regexPattern"
          clearable
        />
      </div>
      <el-button
        type="success"
        @click="replaceData()"
        :loading="isLoading"
        :icon="Refresh"
        plain
        style="margin-top: 10px"
      >
        Replace
      </el-button>
    </div>
    <div style="margin-top: 15px">
      <el-input
        v-model="data.replacement"
        autosize
        clearable
        placeholder="Replacement string - Replace with any string"
      />
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
          v-for="column in tableColumn"
          :prop="column.prop"
          :label="column.label"
          :key="column.prop"
        />
      </el-table>
    </div>
  </div>
</template>

<style>
.page-container {
  margin-bottom: 20px;
  padding: 20px;
  border-radius: 10px;
  background-color: #fff;
}
</style>

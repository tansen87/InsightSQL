<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { Search, FolderOpened } from "@element-plus/icons-vue";

const isLoading = ref(false);
const isPath = ref(false);
const runtime = ref(0.0);
const columns = ref("");
const originalColumns = ref([]);
const data = reactive({
  filePath: "",
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"],
  mode: "equal",
  condition: ""
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

listen("runtime", (event: any) => {
  runtime.value = event.payload;
});

async function selectFile() {
  isLoading.value = false;
  isPath.value = false;
  originalColumns.value = [];
  columns.value = "";

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

  const header: any = await invoke("get_search_headers", {
    path: data.filePath
  });
  originalColumns.value = header;

  try {
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

// search data
async function searchData() {
  if (data.filePath === "") {
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

  if (data.filePath !== "") {
    isLoading.value = true;

    try {
      const matchRows: string = await invoke("search", {
        path: data.filePath,
        selectColumn: columns.value,
        mode: data.mode,
        condition: data.condition
      });

      if (JSON.stringify(matchRows).startsWith("search failed")) {
        throw JSON.stringify(matchRows).toString();
      }

      isLoading.value = false;
      ElNotification({
        message:
          "Search done, match rows: " +
          matchRows +
          " lines, elapsed time: " +
          runtime.value,
        position: "bottom-right",
        type: "success",
        duration: 10000
      });
    } catch (err) {
      ElNotification({
        title: "Invoke Search Error",
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
        <span v-else>Select fields matching rows</span>
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
          style="margin-left: 12px; width: 200px"
          placeholder="Search by column"
        >
          <el-option
            v-for="item in originalColumns"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
      </div>
      <el-button
        type="success"
        @click="searchData()"
        :loading="isLoading"
        :icon="Search"
        plain
        style="margin-top: 10px"
      >
        Search
      </el-button>
    </div>
    <div style="margin-top: 15px">
      <el-input
        v-model="data.condition"
        autosize
        type="textarea"
        placeholder="Search rows with text...Example: tom|jack|world"
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

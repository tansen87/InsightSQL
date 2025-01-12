<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { Refresh, FolderOpened } from "@element-plus/icons-vue";

const isLoading = ref(false);
const isPath1 = ref(false);
const isPath2 = ref(false);
const sel1 = ref("");
const originalColumns1 = ref([]);
const sel2 = ref("");
const originalColumns2 = ref([]);
const data = reactive({
  path1: "",
  path2: "",
  joinType: "left",
  nulls: false,
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"]
});
const tableColumn1 = ref([]);
const tableData1 = ref([]);
const tableColumn2 = ref([]);
const tableData2 = ref([]);
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

async function selectFile(fileIndex) {
  const isPath = fileIndex === 1 ? isPath1 : isPath2;
  const originalColumns: any =
    fileIndex === 1 ? originalColumns1 : originalColumns2;
  const selectColumn = fileIndex === 1 ? sel1 : sel2;
  const tableColumn = fileIndex === 1 ? tableColumn1 : tableColumn2;
  const tableData = fileIndex === 1 ? tableData1 : tableData2;
  const path = fileIndex === 1 ? "path1" : "path2";

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

  if (selected === null) return;

  data[path] = Array.isArray(selected) ? selected.toString() : selected;
  isPath.value = true;

  try {
    const header = await invoke("get_join_headers", { filePath: data[path] });
    originalColumns.value = header;

    const result: string = await invoke("query", {
      path: data[path],
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
      throw new Error(result[0]);
    }

    const jsonData = JSON.parse(result);
    const arrayData = Array.isArray(jsonData) ? jsonData : [jsonData];
    tableColumn.value = Object.keys(arrayData[0]).map(key => ({
      name: key,
      label: key,
      prop: key
    }));
    tableData.value = arrayData;
  } catch (err) {
    ElNotification({
      title: "Open file error",
      message: err.message,
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
}

// invoke join
async function joinData() {
  if (data.path1 === "" || data.path2 === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }
  if (sel1.value.length === 0 || sel2.value.length === 0) {
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
    const result: string = await invoke("join", {
      path1: data.path1,
      path2: data.path2,
      sel1: sel1.value,
      sel2: sel2.value,
      joinType: data.joinType,
      nulls: data.nulls
    });

    if (JSON.stringify(result).startsWith("join failed:")) {
      throw JSON.stringify(result).toString();
    }

    isLoading.value = false;
    ElNotification({
      message: "Join done, " + "elapsed time: " + result + " s.",
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
  } catch (err) {
    ElNotification({
      title: "Invoke Join Error",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
  isLoading.value = false;
}

const viewFileName1 = computed(() => {
  const paths = data.path1.split("|");
  return paths.map(path => {
    const pathParts = path.split(/[/\\]/);
    const fileName = pathParts[pathParts.length - 1];
    return fileName;
  });
});
const viewFileName2 = computed(() => {
  const paths = data.path2.split("|");
  return paths.map(path => {
    const pathParts = path.split(/[/\\]/);
    const fileName = pathParts[pathParts.length - 1];
    return fileName;
  });
});
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
          @click="selectFile(1)"
          :icon="FolderOpened"
          plain
        >
          File 1
        </el-button>

        <el-button
          type="primary"
          @click="selectFile(2)"
          :icon="FolderOpened"
          plain
        >
          File 2
        </el-button>
      </div>

      <el-text type="primary" size="large">
        <span>Joins two sets of CSV data on the specified columns</span>
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
        <el-tooltip content="column of file1" placement="top" effect="light">
          <el-select
            v-model="sel1"
            filterable
            style="width: 200px; margin-right: 10px"
            placeholder="column of file1"
          >
            <el-option
              v-for="item in originalColumns1"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>
        </el-tooltip>
        <el-tooltip content="column of file2" placement="top" effect="light">
          <el-select
            v-model="sel2"
            filterable
            style="width: 200px; margin-right: 10px"
            placeholder="column of file2"
          >
            <el-option
              v-for="item in originalColumns2"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>
        </el-tooltip>
        <el-tooltip
          content="When set true, joins will work on empty fields"
          placement="top"
          effect="light"
        >
          <el-select v-model="data.nulls" style="width: 100px">
            <el-option label="true" :value="true" />
            <el-option label="false" :value="false" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="join type" placement="top" effect="light">
          <el-select
            v-model="data.joinType"
            style="width: 100px; margin-left: 10px"
          >
            <el-option label="left" value="left" />
            <el-option label="right" value="right" />
            <el-option label="full" value="full" />
            <el-option label="cross" value="cross" />
            <el-option label="inner" value="inner" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button
        type="success"
        @click="joinData()"
        :loading="isLoading"
        :icon="Refresh"
        plain
        style="margin-top: 15px"
      >
        Join
      </el-button>
    </div>
    <div style="display: flex; justify-content: space-between">
      <!-- 第一个文件的文本和表格 -->
      <div style="display: flex; flex-direction: column; width: 49%">
        <div style="margin-bottom: 10px">
          <el-text type="primary" size="large">
            <span v-if="isPath1">{{ viewFileName1[0] }}</span>
          </el-text>
        </div>
        <el-table
          ref="tableRef1"
          :data="tableData1"
          :height="formHeight"
          border
          style="width: 100%"
        >
          <el-table-column
            v-for="column in tableColumn1"
            :prop="column.prop"
            :label="column.label"
            :key="column.prop"
          />
        </el-table>
      </div>

      <!-- 第二个文件的文本和表格 -->
      <div style="display: flex; flex-direction: column; width: 49%">
        <div style="margin-bottom: 10px">
          <el-text type="primary" size="large">
            <span v-if="isPath2">{{ viewFileName2[0] }}</span>
          </el-text>
        </div>
        <el-table
          ref="tableRef2"
          :data="tableData2"
          :height="formHeight"
          border
          style="width: 100%"
        >
          <el-table-column
            v-for="column in tableColumn2"
            :prop="column.prop"
            :label="column.label"
            :key="column.prop"
          />
        </el-table>
      </div>
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

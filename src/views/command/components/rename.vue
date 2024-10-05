<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { Watermelon, FolderOpened } from "@element-plus/icons-vue";

const tableData: any = ref([]);
const writeRows = ref(0);
const runtime = ref(0.0);
const isLoading = ref(false);
const isPath = ref(false);
const search = ref("");
const tableRef = ref(null);
const windowHeight = ref(window.innerHeight);
const filterTableData = computed(() =>
  tableData.value.filter(
    (data: any) =>
      !search.value ||
      data.col1.toLowerCase().includes(search.value.toLowerCase())
  )
);
const data = reactive({
  filePath: "",
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"],
  sep: ","
});

const formHeight = computed(() => {
  const height = 205;
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
listen("get_err", (event: any) => {
  const getRenameHeadersError = event.payload;
  ElNotification({
    title: "Get Rename Headers Error",
    message: getRenameHeadersError,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});
listen("rename_err", (event: any) => {
  const renameError = event.payload;
  ElNotification({
    title: "Rename Error",
    message: renameError,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});
listen("count_rows", (event: any) => {
  const count: any = event.payload;
  writeRows.value = count;
});

// open file
async function selectFile() {
  tableData.value = [];
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
    data.filePath = selected.toString();
  } else if (selected === null) {
    return;
  } else {
    data.filePath = selected;
  }

  isPath.value = true;

  const headers: any = await invoke("get_rename_headers", {
    path: data.filePath,
    sep: data.sep
  });

  for (let i = 0; i < headers.length; i++) {
    const colData = {
      col1: headers[i],
      col2: headers[i % headers.length]
    };
    tableData.value.push(colData);
  }
}

// rename csv headers
async function renameData() {
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  const headersStringArray = tableData.value.map((row: any) => row.col2);
  const headersString = headersStringArray.join("|");
  isLoading.value = true;

  await invoke("rename", {
    path: data.filePath,
    sep: data.sep,
    headers: headersString
  });

  isLoading.value = false;
  ElNotification({
    message:
      "Rename done, write rows: " +
      writeRows.value +
      " lines, elapsed time: " +
      runtime.value,
    position: "bottom-right",
    type: "success",
    duration: 10000
  });
}

async function headerEdit(row: any) {
  return row;
}
</script>

<template>
  <el-form class="page-container" :style="formHeight">
    <el-form>
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
          <el-select v-model="data.sep" style="margin-left: 16px; width: 100px">
            <el-option label="," value="," />
            <el-option label="|" value="|" />
            <el-option label="\t" value="\t" />
            <el-option label=";" value=";" />
          </el-select>
          <el-button
            type="success"
            @click="renameData()"
            :loading="isLoading"
            :icon="Watermelon"
            plain
            style="margin-left: 16px"
          >
            Rename
          </el-button>
        </div>

        <el-text type="primary" size="large">
          <el-icon> <Watermelon /> </el-icon>
          <span v-if="isPath">{{ data.filePath }}</span>
          <span v-else>Rename the columns of a CSV</span>
        </el-text>
      </div>
      <el-table
        ref="tableRef"
        :data="filterTableData"
        :height="formHeight"
        style="width: 100%"
      >
        <el-table-column prop="col1" label="headers" style="width: 50%" />
        <el-table-column prop="col2" label="rename headers" width="300">
          <template #default="{ row }">
            <el-input
              v-model="row.col2"
              placeholder="new header"
              class="custom-header-input"
              @blur="headerEdit(row)"
            />
          </template>
        </el-table-column>
        <el-table-column>
          <template #header>
            <el-input
              v-model="search"
              size="small"
              placeholder="Type to search headers"
            />
          </template>
        </el-table-column>
      </el-table>
    </el-form>
  </el-form>
</template>

<style>
.page-container {
  margin-bottom: 20px;
  padding: 20px;
  border-radius: 10px;
  background-color: #fff;
}
</style>

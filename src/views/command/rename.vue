<script setup lang="ts">
import { ref, reactive, computed } from "vue";
import { open } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { ElMessage } from "element-plus";
import {
  SuccessFilled,
  Loading,
  Watermelon,
  FolderOpened
} from "@element-plus/icons-vue";

const tableData: any = ref([]);
const writeRows = ref(0);
const isFinish = ref(false);
const isLoading = ref(false);
const isWrite = ref(false);
const isPath = ref(false);
const search = ref("");
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

listen("get_err", (event: any) => {
  const error: any = event.payload;
  const getErrMsg: any = "get error: " + error;
  ElMessage.error(getErrMsg);
  isLoading.value = false;
});
listen("rename_err", (event: any) => {
  const error: any = event.payload;
  const renameErrMsg: any = "rename error: " + error;
  ElMessage.error(renameErrMsg);
  isLoading.value = false;
});
listen("count_rows", (event: any) => {
  const count: any = event.payload;
  writeRows.value = count;
});

async function selectFile() {
  tableData.value = [];
  isLoading.value = false;
  isFinish.value = false;
  isWrite.value = false;
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
  if (data.filePath == "") {
    ElMessage.warning("未选择csv文件");
    return;
  }

  ElMessage.info("Running...");

  const headersStringArray = tableData.value.map((row: any) => row.col2);
  const headersString = headersStringArray.join(",");
  isLoading.value = true;
  isFinish.value = false;
  await invoke("rename", {
    path: data.filePath,
    sep: data.sep,
    headers: headersString
  });
  isLoading.value = false;
  isFinish.value = true;
  isWrite.value = true;
  ElMessage.success("rename done.");
}

async function headerEdit(row: any) {
  return row;
}
</script>

<template>
  <div class="page-container">
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
            :icon="Watermelon"
            plain
            style="margin-left: 16px"
          >
            Rename
          </el-button>
        </div>
        <el-form-item>
          <el-icon v-if="isLoading" color="#FF8C00" class="is-loading">
            <Loading />
          </el-icon>
          <el-icon v-if="isFinish" color="#32CD32"> <SuccessFilled /> </el-icon>
          <el-text v-if="isWrite" class="mx-1">{{ writeRows }}</el-text>
        </el-form-item>

        <el-text type="primary" size="large">
          <el-icon> <Watermelon /> </el-icon>
          <span v-if="isPath">{{ data.filePath }}</span>
          <span v-else>Rename the columns of a CSV</span>
        </el-text>
      </div>
      <el-table :data="filterTableData" height="760" style="width: 100%">
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

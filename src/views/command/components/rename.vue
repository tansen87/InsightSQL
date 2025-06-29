<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Refresh } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { message } from "@/utils/message";
import { viewOpenFile } from "@/utils/view";
import { listen } from "@tauri-apps/api/event";

const mode = ref("nil");
const tableData = ref([]);
const [search, path] = [ref(""), ref("")];
const [currentRows, totalRows] = [ref(0), ref(0)];
const [isLoading, isPath] = [ref(false), ref(false)];
const { dynamicHeight } = useDynamicHeight(122);
const filterTableData = computed(() =>
  tableData.value.filter(
    (data: any) =>
      !search.value ||
      data.col1.toLowerCase().includes(search.value.toLowerCase())
  )
);

listen("update-rows", (event: any) => {
  currentRows.value = event.payload;
});
listen("total-rows", (event: any) => {
  totalRows.value = event.payload;
});

async function selectFile() {
  tableData.value = [];
  isPath.value = false;
  search.value = "";
  totalRows.value = 0;

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

  try {
    const headers: string[] = await invoke("from_headers", {
      path: path.value
    });
    for (let i = 0; i < headers.length; i++) {
      const colData = {
        col1: headers[i],
        col2: headers[i % headers.length]
      };
      tableData.value.push(colData);
    }
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke rename
async function renameData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const headersStringArray = tableData.value.map((row: any) => row.col2);
    const headersString = headersStringArray.join(",");
    const rtime: string = await invoke("rename", {
      path: path.value,
      headers: headersString,
      mode: mode.value
    });
    message(`Rename done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}

async function headerEdit(row: any) {
  return row;
}
</script>

<template>
  <el-form class="page-container" :style="dynamicHeight">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
        <el-tooltip content="if nil, do not add progress bar" effect="light">
          <el-select v-model="mode" style="margin-left: 10px; width: 70px">
            <el-option label="idx" value="idx" />
            <el-option label="std" value="std" />
            <el-option label="nil" value="nil" />
          </el-select>
        </el-tooltip>
        <el-button
          @click="renameData()"
          :loading="isLoading"
          :icon="Refresh"
          style="margin-left: 10px"
        >
          Rename
        </el-button>
      </div>
      <el-text>
        <span v-if="isPath">
          <el-tooltip :content="path" effect="light">
            <span>{{ shortFileName(path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>Rename the columns of a CSV</span>
      </el-text>
    </div>
    <el-table
      :data="filterTableData"
      :height="dynamicHeight"
      style="width: 100%"
      empty-text=""
    >
      <el-table-column prop="col1" label="headers" style="width: 50%" />
      <el-table-column prop="col2" label="new headers" width="300">
        <template #default="{ row }">
          <el-input
            v-model="row.col2"
            placeholder="new header"
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
            style="flex: 1; margin-right: 8px"
          />
          <el-progress
            :text-inside="true"
            :stroke-width="15"
            v-if="totalRows !== 0 && isFinite(currentRows / totalRows)"
            :percentage="Math.round((currentRows / totalRows) * 100)"
            style="width: 100px"
          />
        </template>
      </el-table-column>
    </el-table>
  </el-form>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, SwitchFilled } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { listen } from "@tauri-apps/api/event";

const [mode, pinyinStyle] = [ref("nil"), ref("upper")];
const [currentRows, totalRows] = [ref(0), ref(0)];
const [columns, path] = [ref(""), ref("")];
const [isLoading, isPath] = [ref(false), ref(false)];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const { dynamicHeight } = useDynamicHeight(192);

listen("update-rows", (event: any) => {
  currentRows.value = event.payload;
});
listen("total-rows", (event: any) => {
  totalRows.value = event.payload;
});

async function selectFile() {
  isPath.value = false;
  columns.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];
  totalRows.value = 0;

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

  try {
    tableHeader.value = await mapHeaders(path.value, "0");
    const { columnView, dataView } = await toJson(path.value);
    tableColumn.value = columnView;
    tableData.value = dataView;
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke pinyin
async function chineseToPinyin() {
  if (path.value === "") {
    message("File not selected", { type: "warning" });
    return;
  }
  if (columns.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const cols = Object.values(columns.value).join("|");
    const rtime: string = await invoke("pinyin", {
      path: path.value,
      columns: cols,
      mode: mode.value,
      pinyinStyle: pinyinStyle.value
    });
    message(`Convert done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <div class="page-container">
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
        <el-tooltip content="upper or lower" effect="light">
          <el-select
            v-model="pinyinStyle"
            style="margin-left: 10px; width: 80px"
          >
            <el-option label="upper" value="upper" />
            <el-option label="lower" value="lower" />
          </el-select>
        </el-tooltip>
        <el-button
          @click="chineseToPinyin()"
          :loading="isLoading"
          :icon="SwitchFilled"
          style="margin-left: 10px"
        >
          Convert
        </el-button>
      </div>
      <el-text>
        <span v-if="isPath">
          <el-tooltip :content="path" effect="light">
            <span>{{ shortFileName(path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>Convert Chinese to Pinyin in CSV</span>
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
        v-for="item in tableHeader"
        :key="item.value"
        :label="item.label"
        :value="item.value"
      />
    </el-select>
    <el-table
      :data="tableData"
      :height="dynamicHeight"
      border
      empty-text=""
      style="margin-top: 12px; width: 100%"
      show-overflow-tooltip
    >
      <el-table-column
        v-for="column in tableColumn"
        :prop="column.prop"
        :label="column.label"
        :key="column.prop"
      />
    </el-table>
    <el-progress
      v-if="totalRows !== 0 && isFinite(currentRows / totalRows)"
      :percentage="Math.round((currentRows / totalRows) * 100)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { FolderOpened, Refresh } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";
import { viewOpenFile, viewSqlp } from "@/utils/view";

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
  skipRows: "0",
  n: "4",
  sliceSep: "-",
  mode: "left"
});
const { formHeight } = useDynamicFormHeight(190);

async function selectFile() {
  isPath.value = false;
  selectColumn.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

  data.path = await viewOpenFile(false, "csv", ["*"]);
  if (data.path === null) {
    return;
  }
  isPath.value = true;

  try {
    const { headerView, columnView, dataView } = await viewSqlp(
      data.path,
      data.skipRows
    );
    tableHeader.value = headerView;
    tableColumn.value = columnView;
    tableData.value = dataView;
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

// invoke slice
async function sliceData() {
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
    const result: string = await invoke("slice", {
      path: data.path,
      skipRows: data.skipRows,
      selectColumn: selectColumn.value,
      n: data.n,
      sliceSep: data.sliceSep,
      mode: data.mode
    });

    if (JSON.stringify(result).startsWith("slice failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Slice done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
  } catch (err) {
    ElNotification({
      title: "Slice failed",
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
          />
        </el-tooltip>
      </div>

      <el-text>
        <span v-if="isPath">{{ data.path }}</span>
        <span v-else>Slicing of CSV column</span>
      </el-text>
    </div>

    <div class="custom-container1">
      <div class="custom-container1" style="margin-top: 12px">
        <el-select
          v-model="selectColumn"
          filterable
          style="width: 200px"
          placeholder="Slice by column"
        >
          <el-option
            v-for="item in tableHeader"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>

        <el-tooltip content="Numer of slice" placement="top" effect="light">
          <el-input v-model="data.n" style="margin-left: 10px; width: 50px" />
        </el-tooltip>
        <el-tooltip content="Slice separator" placement="top" effect="light">
          <el-input
            v-model="data.sliceSep"
            style="margin-left: 10px; width: 50px"
          />
        </el-tooltip>
        <el-tooltip content="Slice mode" placement="top" effect="light">
          <el-select v-model="data.mode" style="margin-left: 10px; width: 84px">
            <el-option label="Left" value="left" />
            <el-option label="Right" value="right" />
            <el-option label="Nth" value="nth" />
            <el-option label="Nmax" value="nmax" />
          </el-select>
        </el-tooltip>
      </div>

      <el-button
        @click="sliceData()"
        :loading="isLoading"
        :icon="Refresh"
        plain
        style="margin-top: 10px"
      >
        Slice
      </el-button>
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

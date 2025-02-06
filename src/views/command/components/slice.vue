<script setup lang="ts">
import { ref, reactive, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Refresh, Link } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";
import { viewOpenFile, viewSqlp } from "@/utils/view";
import { sliceContent, useMarkdown } from "@/utils/markdown";
import { message } from "@/utils/message";

const [
  isLoading,
  isPath,
  selectColumn,
  tableHeader,
  tableColumn,
  tableData,
  infoDialog
] = [ref(false), ref(false), ref(""), ref([]), ref([]), ref([]), ref(false)];
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
    message(err.toString(), { type: "error", duration: 10000 });
  }
}

// invoke slice
async function sliceData() {
  if (data.path === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (selectColumn.value.length === 0) {
    message("Column not selected", { type: "warning" });
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

    message(`Slice done, elapsed time: ${result} s`, { duration: 5000 });
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
  isLoading.value = false;
}

const { compiledMarkdown, manualHighlight } = useMarkdown(sliceContent);
const handleDialogOpened = async () => {
  await nextTick();
  manualHighlight();
};
</script>

<template>
  <div class="page-container">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>

        <el-tooltip content="skip rows" placement="top" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 78px"
          />
        </el-tooltip>
      </div>

      <el-link @click="infoDialog = true" :icon="Link">
        <span v-if="isPath">{{ data.path }}</span>
        <span v-else>
          How to use
          <span style="color: skyblue; font-weight: bold">slice</span>
        </span>
      </el-link>
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

    <el-dialog
      v-model="infoDialog"
      title="Slice - Slicing of csv column"
      width="800"
      @opened="handleDialogOpened"
    >
      <el-scrollbar :height="formHeight * 0.8">
        <div v-html="compiledMarkdown" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

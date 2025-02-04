<script setup lang="ts">
import { ref, reactive, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { IceCreamRound, FolderOpened, Link } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";
import { viewOpenFile, viewSqlp } from "@/utils/view";
import { splitContent, useMarkdown } from "@/utils/markdown";

const [isLoading, isPath, tableColumn, tableData, infoDialog] = [
  ref(false),
  ref(false),
  ref([]),
  ref([]),
  ref(false)
];
const data = reactive({
  path: "",
  size: 1000000,
  skipRows: "0"
});
const { formHeight } = useDynamicFormHeight(188);

async function selectFile() {
  isPath.value = false;
  tableColumn.value = [];
  tableData.value = [];

  data.path = await viewOpenFile(false, "csv", ["*"]);
  if (data.path === null) {
    return;
  }
  isPath.value = true;

  try {
    const { columnView, dataView } = await viewSqlp(data.path, data.skipRows);
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

// invoke split
async function splitData() {
  if (data.path === "") {
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
    const result: string = await invoke("split", {
      path: data.path,
      size: data.size,
      skipRows: data.skipRows
    });

    if (JSON.stringify(result).startsWith("split failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Split done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 5000
    });
  } catch (err) {
    ElNotification({
      title: "Split failed",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
  isLoading.value = false;
}

const { compiledMarkdown, manualHighlight } = useMarkdown(splitContent);
const handleDialogOpened = async () => {
  await nextTick();
  manualHighlight();
};
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
            style="margin-left: 10px; width: 50px"
            placeholder="skip rows"
          />
        </el-tooltip>
      </div>

      <el-link @click="infoDialog = true" :icon="Link">
        <span v-if="isPath">{{ data.path }}</span>
        <span v-else>
          How to use
          <span style="color: skyblue; font-weight: bold">split</span>
        </span>
      </el-link>
    </div>

    <div class="custom-container1">
      <div class="custom-container2" style="margin-top: 10px">
        <el-tooltip content="Split rows" placement="bottom" effect="light">
          <el-input-number
            v-model="data.size"
            controls-position="right"
            style="width: 172px"
          />
        </el-tooltip>
      </div>

      <el-button
        style="margin-top: 10px"
        @click="splitData()"
        :loading="isLoading"
        :icon="IceCreamRound"
        plain
      >
        Split
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
      title="Split - Split one CSV file into many CSV files"
      width="800"
      @opened="handleDialogOpened"
    >
      <el-scrollbar :height="formHeight * 0.8">
        <div v-html="compiledMarkdown" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

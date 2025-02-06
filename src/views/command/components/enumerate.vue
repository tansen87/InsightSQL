<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { IceCreamRound, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";
import { viewOpenFile, viewSqlp } from "@/utils/view";
import { message } from "@/utils/message";

const [isLoading, isPath, tableColumn, tableData] = [
  ref(false),
  ref(false),
  ref([]),
  ref([])
];
const data = reactive({
  path: "",
  skipRows: "0"
});
const { formHeight } = useDynamicFormHeight(149);

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
    message(err.toString(), { type: "error", duration: 10000 });
  }
}

// invoke enumer
async function enumerate() {
  if (data.path === "") {
    message("File not selected", { type: "warning" });
    return;
  }

  isLoading.value = true;

  try {
    const result: string = await invoke("enumer", {
      path: data.path,
      skipRows: data.skipRows
    });

    message(`Enumerate done, elapsed time: ${result} s`, { duration: 5000 });
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
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

        <el-tooltip content="skip rows" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 50px"
          />
        </el-tooltip>

        <el-button
          @click="enumerate()"
          :loading="isLoading"
          :icon="IceCreamRound"
          style="margin-left: 10px"
        >
          Enumerate
        </el-button>
      </div>

      <el-text>
        <span v-if="isPath">{{ data.path }}</span>
        <span v-else>Add an index for a CSV</span>
      </el-text>
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

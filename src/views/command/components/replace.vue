<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Refresh } from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { mapHeaders, viewOpenFile, viewSqlp } from "@/utils/view";
import { message } from "@/utils/message";

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
  regexPattern: "",
  replacement: "",
  skipRows: "0"
});
const { dynamicHeight } = useDynamicHeight(234);

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

  try {
    tableHeader.value = await mapHeaders(data.path, data.skipRows);
    const { columnView, dataView } = await viewSqlp(data.path, data.skipRows);
    tableColumn.value = columnView;
    tableData.value = dataView;
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
}

// invoke replace
async function replaceData() {
  if (data.path === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (selectColumn.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const result: string = await invoke("replace", {
      path: data.path,
      selectColumn: selectColumn.value,
      regexPattern: data.regexPattern,
      replacement: data.replacement,
      skipRows: data.skipRows
    });
    message(`Replace done, elapsed time: ${result} s`);
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
        <el-tooltip content="skip rows" placement="top" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 78px"
          />
        </el-tooltip>
      </div>

      <el-text>
        <span v-if="isPath">
          <el-tooltip :content="data.path" placement="top" effect="light">
            <span>{{ shortFileName(data.path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>Replace occurrences of a pattern across a CSV file</span>
      </el-text>
    </div>

    <div class="custom-container1">
      <div class="custom-container1" style="margin-top: 12px">
        <el-select
          v-model="selectColumn"
          filterable
          style="width: 200px"
          placeholder="Replace by column"
        >
          <el-option
            v-for="item in tableHeader"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>

        <el-input
          style="margin-left: 10px; width: 200px"
          placeholder="regex pattern"
          v-model="data.regexPattern"
          clearable
        />
      </div>

      <el-button
        @click="replaceData()"
        :loading="isLoading"
        :icon="Refresh"
        style="margin-top: 10px"
      >
        Replace
      </el-button>
    </div>

    <div style="margin-top: 12px">
      <el-input
        v-model="data.replacement"
        autosize
        clearable
        placeholder="Replacement string - Replace with any string"
      />
    </div>

    <el-table
      :data="tableData"
      :height="dynamicHeight"
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

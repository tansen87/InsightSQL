<script setup lang="ts">
import { ref, reactive, computed } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { FolderOpened, Connection } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";

const [
  isLoading,
  isPath1,
  isPath2,
  sel1,
  sel2,
  tableHeader1,
  tableHeader2,
  tableColumn1,
  tableColumn2,
  tableData1,
  tableData2
] = [
  ref(false),
  ref(false),
  ref(false),
  ref(""),
  ref(""),
  ref([]),
  ref([]),
  ref([]),
  ref([]),
  ref([]),
  ref([])
];
const data = reactive({
  path1: "",
  path2: "",
  joinType: "left",
  nulls: false,
  fileFormats: ["*"]
});

const { formHeight } = useDynamicFormHeight(215);

async function selectFile(fileIndex) {
  const isPath = fileIndex === 1 ? isPath1 : isPath2;
  const selectColumn = fileIndex === 1 ? sel1 : sel2;
  const tableHeader: any = fileIndex === 1 ? tableHeader1 : tableHeader2;
  const tableColumn = fileIndex === 1 ? tableColumn1 : tableColumn2;
  const tableData = fileIndex === 1 ? tableData1 : tableData2;
  const path = fileIndex === 1 ? "path1" : "path2";

  isPath.value = false;
  selectColumn.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

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
    tableHeader.value = Object.keys(arrayData[0]).map(header => ({
      label: header,
      value: header
    }));
    tableColumn.value = Object.keys(arrayData[0]).map(key => ({
      name: key,
      label: key,
      prop: key
    }));
    tableData.value = arrayData;
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

    ElNotification({
      message: `Join done, elapsed time: ${result} s.`,
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
  } catch (err) {
    ElNotification({
      title: "Join failed",
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
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile(1)" :icon="FolderOpened" plain>
          File 1
        </el-button>

        <el-button @click="selectFile(2)" :icon="FolderOpened" plain>
          File 2
        </el-button>
      </div>

      <el-text>
        <span>Joins two sets of CSV data on the specified columns</span>
      </el-text>
    </div>

    <div class="custom-container1">
      <div class="custom-container2" style="margin-top: 12px">
        <el-tooltip content="column of file1" placement="top" effect="light">
          <el-select
            v-model="sel1"
            filterable
            style="width: 200px; margin-right: 10px"
            placeholder="column of file1"
          >
            <el-option
              v-for="item in tableHeader1"
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
              v-for="item in tableHeader2"
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
        @click="joinData()"
        :loading="isLoading"
        :icon="Connection"
        plain
        style="margin-top: 12px"
      >
        Join
      </el-button>
    </div>

    <div style="display: flex; justify-content: space-between">
      <div style="display: flex; flex-direction: column; width: 49%">
        <div style="margin-bottom: 10px">
          <el-text>
            <span v-if="isPath1">{{ viewFileName1[0] }}</span>
          </el-text>
        </div>
        <el-table
          :data="tableData1"
          :height="formHeight"
          border
          empty-text=""
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

      <div style="display: flex; flex-direction: column; width: 49%">
        <div style="margin-bottom: 10px">
          <el-text>
            <span v-if="isPath2">{{ viewFileName2[0] }}</span>
          </el-text>
        </div>
        <el-table
          :data="tableData2"
          :height="formHeight"
          border
          empty-text=""
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

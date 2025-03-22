<script setup lang="ts">
import { ref, reactive, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Connection } from "@element-plus/icons-vue";
import { useDynamicHeight } from "@/utils/utils";
import { mapHeaders, viewOpenFile, viewSqlp } from "@/utils/view";
import { message } from "@/utils/message";

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
  tableData2,
  joinType,
  nulls
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
  ref([]),
  ref("left"),
  ref(false)
];
const data = reactive({
  path1: "",
  path2: ""
});

const { dynamicHeight } = useDynamicHeight(200);

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

  data[path] = await viewOpenFile(false, "csv", ["*"]);
  if (data[path] === null) {
    return;
  }

  try {
    tableHeader.value = await mapHeaders(data[path], "0");
    const { columnView, dataView } = await viewSqlp(data[path], "0");
    tableColumn.value = columnView;
    tableData.value = dataView;
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
}

// invoke join
async function joinData() {
  if (data.path1 === "" || data.path2 === "") {
    message("File not selected", { type: "warning" });
    return;
  }
  if (sel1.value.length === 0 || sel2.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("join", {
      path1: data.path1,
      path2: data.path2,
      sel1: sel1.value,
      sel2: sel2.value,
      joinType: joinType.value,
      nulls: nulls.value
    });
    message(`Join done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
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
        <el-button @click="selectFile(1)" :icon="FolderOpened">
          File 1
        </el-button>
        <el-button @click="selectFile(2)" :icon="FolderOpened">
          File 2
        </el-button>
      </div>
      <el-text>
        <span>Joins two sets of CSV data on the specified columns</span>
      </el-text>
    </div>

    <div class="custom-container1">
      <div class="custom-container2" style="margin-top: 12px">
        <el-tooltip content="column of file1" effect="light">
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
        <el-tooltip content="column of file2" effect="light">
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
          effect="light"
        >
          <el-select v-model="nulls" style="width: 100px">
            <el-option label="true" :value="true" />
            <el-option label="false" :value="false" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="join type" effect="light">
          <el-select v-model="joinType" style="width: 100px; margin-left: 10px">
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
          :height="dynamicHeight"
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
          :height="dynamicHeight"
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

<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { ElMessage, ElMessageBox } from "element-plus";
import { Select, Loading, View, Download } from "@element-plus/icons-vue";

const selectedFiles = ref([]);
const columns = ref([]);
const tableData = ref([]);
const isLoading = ref(false);
const isFinish = ref(false);
const isRuntime = ref(false);
const runtime = ref(0.0);
const data = reactive({
  filePath: "",
  fileFormats: ["csv", "txt", "tsv", "spext", "dat", "parquet"],
  sqlsrc: "select * from `filename`",
  sep: ",",
  write: false,
  writeFormat: "csv"
});

listen("run_time", (event: any) => {
  const time: any = event.payload;
  runtime.value = time;
});
listen("expired", (event: any) => {
  const expired: any = event.payload;
  ElMessageBox.alert(expired, "Tips", {
    confirmButtonText: "OK"
  });
});
listen("query_err", (event: any) => {
  const error: any = event.payload;
  const queryErrmsg: any = "query_err: " + error;
  ElMessage.error(queryErrmsg);
});
listen("size_msg", (event: any) => {
  const error: any = event.payload;
  const sizeMsg: any = "file size error: " + error;
  ElMessage.error(sizeMsg);
});
listen("exec_err", (event: any) => {
  const error: any = "exec_err: " + event.payload;
  ElMessage.error(error);
});
listen("get_err", (event: any) => {
  const error: any = event.payload;
  const getErrmsg: any = "get_err: " + error;
  ElMessage.error(getErrmsg);
});
listen("show", (event: any) => {
  const df: any = event.payload;
  const jsonData = JSON.parse(df);

  // 检查 jsonData 是否是数组，如果不是，则将其转换为数组
  const isJsonArray = Array.isArray(jsonData);
  const data = isJsonArray ? jsonData : [jsonData];
  columns.value = Object.keys(data[0]).map(key => ({
    name: key,
    label: key,
    prop: key
  }));
  tableData.value = data;
});

// query data
async function queryData() {
  columns.value = [];
  tableData.value = [];
  if (data.filePath == "") {
    ElMessage.warning("未选择csv文件");
    return;
  }
  if (data.sqlsrc == "") {
    ElMessage.warning("sql script is empty");
    return;
  }
  if (data.filePath != "" && data.sqlsrc != "") {
    isLoading.value = true;
    isFinish.value = false;
    isRuntime.value = false;
    try {
      await invoke("query", {
        path: data.filePath,
        sqlsrc: data.sqlsrc,
        sep: data.sep,
        write: data.write,
        writeFormat: data.writeFormat
      });
    } catch (err) {
      ElMessage.error(err);
    }
    isLoading.value = false;
    isFinish.value = true;
    isRuntime.value = true;
  }
}

async function selectFile() {
  columns.value = [];
  tableData.value = [];
  selectedFiles.value = [];
  isLoading.value = false;
  isFinish.value = false;
  isRuntime.value = false;
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "csv",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.filePath = selected.toString();
    const nonEmptyRows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = nonEmptyRows.map((row: any) => {
      return { filename: row };
    });
  } else if (selected === null) {
    return;
  } else {
    data.filePath = selected;
  }
  /*
  const results: any = await invoke("get", {
    path: data.filePath,
    sep: data.sep
  });
  const jsonData = JSON.parse(results);
  columns.value = Object.keys(jsonData[0]).map(key => ({
    name: key,
    label: key,
    prop: key
  }));
  tableData.value = jsonData;
  */
}

function textareaChange(event: any) {
  const textarea = event.target;
  textarea.style.height = "auto";
  textarea.style.height = textarea.scrollHeight + "px";
}
</script>

<template>
  <el-form>
    <div style="display: flex; align-items: flex-start">
      <el-button type="primary" @click="selectFile()">Open File</el-button>
      <el-form-item style="margin-left: 10px; width: 200px">
        <el-select v-model="data.sep">
          <el-option label="," value="," />
          <el-option label="|" value="|" />
          <el-option label="\t" value="\t" />
          <el-option label=";" value=";" />
        </el-select>
      </el-form-item>
    </div>
    <el-form-item>
      <textarea
        v-model="data.sqlsrc"
        rows="1"
        class="txt"
        @input="textareaChange"
        placeholder="select * from `filename`"
      />
    </el-form-item>
    <el-form-item>
      <el-button type="success" @click="queryData()">Execute</el-button>
      <el-switch
        v-model="data.write"
        :active-action-icon="Download"
        :inactive-action-icon="View"
        style="margin-left: 20px"
      />
      <el-select
        v-model="data.writeFormat"
        style="margin-left: 20px; width: 80px"
      >
        <el-option label="csv" value="csv" />
        <el-option label="xlsx" value="xlsx" />
      </el-select>
      <div class="icon-group">
        <el-icon v-if="isLoading" color="#FF4500" class="is-loading">
          <Loading />
        </el-icon>
        <el-icon v-if="isFinish" color="#32CD32">
          <Select />
        </el-icon>
        <el-text
          v-if="isRuntime"
          :style="{ color: '#32CD32', fontSize: '20px' }"
          >{{ runtime }}</el-text
        >
      </div>
    </el-form-item>
  </el-form>
  <el-table :data="selectedFiles" height="120" style="width: 100%">
    <el-table-column prop="filename" label="file" />
  </el-table>
  <el-table :data="tableData" height="520" style="width: 100%">
    <el-table-column
      v-for="column in columns"
      :prop="column.prop"
      :label="column.label"
      :key="column.prop"
    />
  </el-table>
</template>

<style scoped>
.icon-group {
  display: flex;
  justify-content: flex-end; /* 将图标对齐到右侧 */
  margin-left: 50px; /* 增加左侧的间距 */
  align-items: center; /* 确保图标和按钮垂直对齐 */
}
.el-icon {
  font-size: 30px; /* 根据需要调整图标大小 */
}
.txt {
  border: 1px solid #cccccc;
  outline: none;
  font-size: 16px;
  font-family: "Cascadia Code", monospace;
  display: block;
  width: 100%;
  resize: none;
  line-height: 1.5;
  background: transparent;
  color: inherit;
  overflow: hidden;
}
.txt.autosize {
  min-height: 40px; /* 设置一个最小高度 */
  height: auto !important; /* 允许高度自动调整 */
}
</style>

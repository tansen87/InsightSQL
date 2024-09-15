<script setup lang="ts">
import { ref, reactive, onMounted, watchEffect } from "vue";
import { open } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { ElMessage, ElMessageBox, ElNotification } from "element-plus";
import { Select, Loading, View, Download } from "@element-plus/icons-vue";
import { FolderOpened, Search } from "@element-plus/icons-vue";
import Prism from "prismjs";
import "prismjs/components/prism-sql";
import "prismjs/themes/prism.css";

// const selectedFiles = ref([]);
const columns = ref([]);
const tableData = ref([]);
const isLoading = ref(false);
const isFinish = ref(false);
const isRuntime = ref(false);
const runtime = ref(0.0);
const data = reactive({
  filePath: "",
  fileFormats: [
    "csv",
    "txt",
    "tsv",
    "spext",
    "dat",
    "parquet",
    "xls",
    "xlsx",
    "xlsm",
    "xlsb",
    "ods"
  ],
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
listen("exec_err", (event: any) => {
  const error: any = "" + event.payload;
  ElMessage({
    message: error,
    type: "error",
    plain: true
  });
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
    ElMessage({
      message: "未选择文件",
      type: "warning",
      plain: true
    });
    return;
  }
  if (sqlsrc.value == "") {
    ElMessage({
      message: "sql script is empty",
      type: "warning",
      plain: true
    });
    return;
  }
  if (data.filePath != "" && sqlsrc.value != "") {
    isLoading.value = true;
    isFinish.value = false;
    isRuntime.value = false;
    try {
      await invoke("query", {
        path: data.filePath,
        sqlsrc: sqlsrc.value,
        sep: data.sep,
        write: data.write,
        writeFormat: data.writeFormat
      });
    } catch (err) {
      ElNotification({
        title: "invoke query",
        message: err,
        position: "bottom-right",
        type: "error"
      });
    }
    isLoading.value = false;
    isFinish.value = true;
    isRuntime.value = true;
  }
}

async function selectFile() {
  columns.value = [];
  tableData.value = [];
  // selectedFiles.value = [];
  isLoading.value = false;
  isFinish.value = false;
  isRuntime.value = false;
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.filePath = selected.join("|").toString();
    // const nonEmptyRows = selected.filter((row: any) => row.trim() !== "");
    // selectedFiles.value = nonEmptyRows.map((row: any) => {
    //   return { filename: row };
    // });
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

const sqlsrc = ref("select * from _t_1");
const highlightedCode = ref(null);

const textareaChange = event => {
  sqlsrc.value = event.target.value;
  adjustTextareaHeight(event.target);
};

const adjustTextareaHeight = textarea => {
  textarea.style.height = "auto";
  textarea.style.height = `${textarea.scrollHeight}px`;
};

const updateHighlightedCode = () => {
  if (highlightedCode.value) {
    highlightedCode.value.innerHTML = Prism.highlight(
      sqlsrc.value,
      Prism.languages.sql,
      "sql"
    );
  }
};

watchEffect(() => {
  updateHighlightedCode();
});

onMounted(() => {
  updateHighlightedCode();
});

const indexMethod = (index: number) => {
  return (index + 1) * 1;
};
</script>

<template>
  <el-form>
    <div
      style="
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
      "
    >
      <div style="display: flex; align-items: flex-start">
        <el-button
          type="primary"
          @click="selectFile()"
          :icon="FolderOpened"
          plain
        >
          Open File
        </el-button>
        <el-form-item style="margin-left: 10px; width: 140px">
          <el-select v-model="data.sep">
            <el-option label="," value="," />
            <el-option label="|" value="|" />
            <el-option label="\t" value="\t" />
            <el-option label=";" value=";" />
          </el-select>
        </el-form-item>
      </div>
      <el-text tag="ins">{{ data.filePath }}</el-text>
    </div>
    <el-form-item class="editor-container">
      <textarea
        :value="sqlsrc"
        @input="textareaChange"
        rows="1"
        class="txt"
        placeholder="select * from _t_1"
      />
      <div ref="highlightedCode" class="highlighted-code" />
    </el-form-item>
    <el-form-item>
      <el-button type="success" @click="queryData()" :icon="Search" plain>
        Execute
      </el-button>
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
  <el-table :data="tableData" height="700" border style="width: 100%">
    <el-table-column type="index" :index="indexMethod" />
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
  font-size: 30px;
}
.txt {
  border: 1px solid #f0dddd;
  outline: none;
  font-size: 18px;
  display: block;
  width: 100%;
  resize: none;
  line-height: 1.5;
  background: transparent;
  color: inherit;
  overflow: hidden;
  font-family: "Consolas", monospace;
}
.highlighted-code {
  border: 1px solid #f0dddd;
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  overflow: hidden;
  pointer-events: none; /* 确保点击事件不会影响到 <div> */
  white-space: pre-wrap; /* 保留换行符和空格 */
  word-wrap: break-word; /* 单词换行 */
  background-color: transparent; /* 背景透明 */
  color: inherit; /* 文字颜色继承 */
  font-family: inherit; /* 字体继承 */
  font-size: 18px;
  padding: 0; /* 消除默认填充 */
  margin: 0; /* 消除默认边距 */
  line-height: 1.5;
  font-family: "Consolas", monospace;
}
</style>

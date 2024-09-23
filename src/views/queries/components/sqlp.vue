<script setup lang="ts">
import {
  ref,
  reactive,
  onMounted,
  watchEffect,
  watch,
  computed,
  onBeforeUnmount
} from "vue";
import { open } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { View, Download } from "@element-plus/icons-vue";
import { FolderOpened, Search } from "@element-plus/icons-vue";
import Prism from "prismjs";
import "prismjs/components/prism-sql";
import "prismjs/themes/prism.css";

const columns = ref([]);
const tableData = ref([]);
const isLoading = ref(false);
const isPath = ref(false);
const runtime = ref(0.0);
const tableRef = ref(null);
const windowHeight = ref(window.innerHeight);
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
  writeFormat: "csv",
  lowMemory: false
});

const formHeight = computed(() => {
  const height = 345;
  return windowHeight.value - height;
});

const updateWindowHeight = () => {
  windowHeight.value = window.innerHeight;
};

onMounted(() => {
  window.addEventListener("resize", updateWindowHeight);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", updateWindowHeight);
});

listen("runtime", (event: any) => {
  runtime.value = event.payload;
});
// listen("expired", (event: any) => {
//   const expired: any = event.payload;
//   ElMessageBox.alert(expired, "Tips", {
//     confirmButtonText: "OK"
//   });
// });
listen("exec_err", (event: any) => {
  ElNotification({
    title: "Execute Error",
    message: event.payload,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
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
    ElNotification({
      title: "File not found",
      message: "未选择CSV, Excel or Parquet文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }
  if (sqlsrc.value == "") {
    ElNotification({
      title: "Warning",
      message: "SQL script is empty",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  if (data.filePath != "" && sqlsrc.value != "") {
    isLoading.value = true;
    try {
      await invoke("query", {
        path: data.filePath,
        sqlsrc: sqlsrc.value,
        sep: data.sep,
        write: data.write,
        writeFormat: data.writeFormat,
        lowMemory: data.lowMemory
      });
    } catch (err) {
      ElNotification({
        title: "invoke query error",
        message: err,
        position: "bottom-right",
        type: "error",
        duration: 10000
      });
    }
    ElNotification({
      message: "Query done, elapsed time: " + runtime.value,
      position: "bottom-right",
      type: "success",
      duration: 5000
    });
    isLoading.value = false;
  }
}

async function selectFile() {
  columns.value = [];
  tableData.value = [];
  isLoading.value = false;
  isPath.value = false;

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
  } else if (selected === null) {
    return;
  } else {
    data.filePath = selected;
  }

  isPath.value = true;
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

watch(
  () => data.lowMemory,
  newVal => {
    if (newVal) {
      data.write = true;
      data.writeFormat = "csv";
    }
  }
);

watch(
  () => data.lowMemory,
  newVal => {
    if (!newVal) {
      data.write = false;
      data.writeFormat = "csv";
    }
  }
);

watch(
  () => data.write,
  newVal => {
    if (!newVal) {
      data.lowMemory = false;
    }
  }
);
</script>

<template>
  <el-form :style="formHeight">
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
        <el-form-item style="margin-left: 10px; width: 80px">
          <el-select v-model="data.sep">
            <el-option label="," value="," />
            <el-option label="|" value="|" />
            <el-option label="\t" value="\t" />
            <el-option label=";" value=";" />
          </el-select>
        </el-form-item>
        <el-form-item style="margin-left: 10px; width: 100px">
          <el-select v-model="data.lowMemory">
            <el-option label="Memory" :value="false" />
            <el-option label="Stream" :value="true" />
          </el-select>
        </el-form-item>
      </div>
      <el-text type="primary" size="large" tag="ins">
        <span v-if="isPath">{{ data.filePath }}</span>
        <span v-else>View Excel, CSV and Parquet using SQL</span>
      </el-text>
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
      <el-button
        type="success"
        @click="queryData()"
        :loading="isLoading"
        :icon="Search"
        plain
      >
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
    </el-form-item>
  </el-form>
  <el-table
    ref="tableRef"
    :data="tableData"
    :height="formHeight"
    border
    style="width: 100%"
  >
    <el-table-column type="index" label="id" :index="indexMethod" />
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
  justify-content: flex-end;
  margin-left: 50px;
  align-items: center;
}
.el-icon {
  font-size: 25px;
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
  pointer-events: none;
  white-space: pre-wrap;
  word-wrap: break-word;
  background-color: transparent;
  color: inherit;
  font-family: inherit;
  font-size: 18px;
  padding: 0;
  margin: 0;
  line-height: 1.5;
  font-family: "Consolas", monospace;
}
</style>

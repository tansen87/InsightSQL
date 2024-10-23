<script setup lang="ts">
import {
  ref,
  reactive,
  onMounted,
  watch,
  computed,
  onBeforeUnmount
} from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { View, Download } from "@element-plus/icons-vue";
import { FolderOpened, Search } from "@element-plus/icons-vue";
import { VAceEditor } from "vue3-ace-editor";
import "./ace-config";

const columns = ref([]);
const tableData = ref([]);
const isLoading = ref(false);
const runtime = ref(0.0);
const counter = ref(0);
const tableRef = ref(null);
const tables = ref([]);
const sqlsrc = ref("select * from _t_1");
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
  write: false,
  writeFormat: "csv",
  lowMemory: false
});

const initializeEditor = editor => {
  editor.completers.push({
    getCompletions: (editor, session, pos, prefix, callback) => {
      callback(
        null,
        tables.value.map(table => ({
          caption: table.name,
          value: table.name,
          meta: "table"
        }))
      );
    }
  });
  tables.value.forEach(item => {
    editor.completers.push({
      getCompletions: (editor, session, pos, prefix, callback) => {
        callback(
          null,
          item.children.map(col => ({
            caption: col.label,
            value: col.label,
            meta: "column"
          }))
        );
      }
    });
  });
};

const formHeight = computed(() => {
  const height = 305;
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

  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择CSV, Excel or Parquet文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }
  if (sqlsrc.value === "") {
    ElNotification({
      title: "Warning",
      message: "SQL script is empty",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  if (data.filePath !== "" && sqlsrc.value !== "") {
    isLoading.value = true;
    try {
      await invoke("query", {
        path: data.filePath,
        sqlsrc: sqlsrc.value,
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

  await invoke("query", {
    path: data.filePath,
    sqlsrc: "select * from _t_1 limit 5",
    write: false,
    writeFormat: "csv",
    lowMemory: false
  });
  /*
  const results: any = await invoke("get", {
    path: data.filePath,
    sep: data.sep
  });
  */
}

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
  <el-form class="page-container">
    <el-form :style="formHeight">
      <div
        style="
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
        "
      >
        <div style="display: flex; align-items: flex-start">
          <el-tooltip
            content="Open local Excel, CSV or Parquet file"
            placement="top"
            effect="light"
          >
            <el-button
              type="primary"
              @click="selectFile()"
              :icon="FolderOpened"
              plain
            >
              Open File
            </el-button>
          </el-tooltip>
          <el-form-item style="margin-left: 10px; width: 100px">
            <el-tooltip
              content="Memory or stream query"
              placement="top"
              effect="light"
            >
              <el-select v-model="data.lowMemory">
                <el-option label="Memory" :value="false" />
                <el-option label="Stream" :value="true" />
              </el-select>
            </el-tooltip>
          </el-form-item>
        </div>
        <el-form-item>
          <el-tooltip
            content="Export data or not"
            placement="top"
            effect="light"
          >
            <el-switch
              v-model="data.write"
              :active-action-icon="Download"
              :inactive-action-icon="View"
            />
          </el-tooltip>
          <el-tooltip content="Export datatype" placement="top" effect="light">
            <el-select
              v-model="data.writeFormat"
              style="margin-left: 10px; width: 80px"
            >
              <el-option label="csv" value="csv" />
              <el-option label="xlsx" value="xlsx" />
            </el-select>
          </el-tooltip>
          <el-tooltip content="Execute query" placement="top" effect="light">
            <el-button
              type="success"
              @click="queryData()"
              :loading="isLoading"
              :icon="Search"
              style="margin-left: 10px"
              plain
            >
              Execute
            </el-button>
          </el-tooltip>
        </el-form-item>
      </div>
      <el-form-item>
        <VAceEditor
          v-model:value="sqlsrc"
          ref="editor"
          lang="sql"
          :options="{
            useWorker: true,
            enableBasicAutocompletion: true,
            enableSnippets: true,
            enableLiveAutocompletion: true,
            customScrollbar: true,
            fontSize: '1.1rem'
          }"
          :key="counter"
          @init="initializeEditor"
          theme="chrome"
          style="flex: 1 1 0%; min-height: 8rem"
        />
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
  </el-form>
</template>

<style scoped>
.page-container {
  margin-bottom: 10px;
  padding: 15px;
  border-radius: 10px;
  background-color: #fff;
}
.icon-group {
  display: flex;
  justify-content: flex-end;
  margin-left: 50px;
  align-items: center;
}
.el-icon {
  font-size: 25px;
}
</style>

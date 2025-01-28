<script setup lang="ts">
import { ref, reactive, watch, computed } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { FolderOpened, Search, View, Download } from "@element-plus/icons-vue";
import { VAceEditor } from "vue3-ace-editor";
import { useDark } from "@pureadmin/utils";
import "./ace-config";
import { useDynamicFormHeight } from "@/utils/utils";

const columns = ref([]);
const treeHeaders = ref([]);
const tableData = ref([]);
const isLoading = ref(false);
const viewTable = ref(false);
const runtime = ref(0.0);
const counter = ref(0);
const tables = ref([]);
const isDataLoaded = ref(false);
const headersByFile = reactive({});
const sqlQuery = ref("select\n*\nfrom _t_1\nlimit 100");
const data = reactive({
  path: "",
  fileFormats: ["*"],
  write: false,
  writeFormat: "csv",
  lowMemory: false,
  skipRows: "0"
});
const { formHeight } = useDynamicFormHeight(102);
const { isDark } = useDark();
const theme = computed(() => (isDark.value ? "monokai" : "chrome"));
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

listen("runtime", (event: any) => {
  runtime.value = event.payload;
});

const queryViewData = async () => {
  const queryResult = await queryData();
  if (queryResult) {
    viewTable.value = true;
  }
};

// invoke query
async function queryData() {
  columns.value = [];
  tableData.value = [];

  if (data.path === "") {
    ElNotification({
      title: "File not found",
      message: "未选择CSV, Excel or Parquet文件",
      position: "bottom-right",
      type: "warning"
    });
    return false;
  }
  if (sqlQuery.value === "") {
    ElNotification({
      title: "Warning",
      message: "SQL script is empty",
      position: "bottom-right",
      type: "warning"
    });
    return false;
  }

  isLoading.value = true;

  try {
    const df: string = await invoke("query", {
      path: data.path,
      sqlQuery: sqlQuery.value,
      write: data.write,
      writeFormat: data.writeFormat,
      lowMemory: data.lowMemory,
      skipRows: data.skipRows
    });

    if (
      (typeof df[0] === "string" && df[0].startsWith("execute_query")) ||
      df[0].startsWith("prepare_query")
    ) {
      throw df[0].toString();
    }

    const jsonData = JSON.parse(df);
    const isJsonArray = Array.isArray(jsonData);
    const arrayData = isJsonArray ? jsonData : [jsonData];
    columns.value = Object.keys(arrayData[0]).map(key => ({
      name: key,
      label: key,
      prop: key
    }));
    tableData.value = arrayData;

    ElNotification({
      message: `Query done, elapsed time: ${runtime.value} s`,
      position: "top-left",
      type: "success",
      duration: 5000
    });

    isLoading.value = false;
    return true;
  } catch (err) {
    ElNotification({
      title: "Invoke query error",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }

  isLoading.value = false;
  return false;
}

const selectViewFile = async () => {
  const selectedFile = await selectFile();
  if (selectedFile) {
    viewTable.value = true;
  }
};

async function selectFile() {
  columns.value = [];
  treeHeaders.value = [];
  tableData.value = [];
  data.path = "";
  viewTable.value = false;

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
    data.path = selected.join("|").toString();
  } else if (selected === null) {
    return;
  } else {
    data.path = selected;
  }

  // 使用 Promise.all 并行处理每个文件
  await Promise.all(
    data.path.split("|").map(async (path, index) => {
      const basename = viewFileName.value[index];
      try {
        const result: any = await invoke("query", {
          path: path,
          sqlQuery: `select * from "${basename}" limit 10`,
          write: false,
          writeFormat: "csv",
          lowMemory: false,
          skipRows: data.skipRows
        });

        if (
          (typeof result[0] === "string" &&
            result[0].startsWith("execute_query")) ||
          result[0].startsWith("prepare_query")
        ) {
          throw result[0].toString();
        }

        const jsonData = JSON.parse(result);
        const isJsonArray = Array.isArray(jsonData);
        const arrayData = isJsonArray ? jsonData : [jsonData];

        columns.value = Object.keys(arrayData[0]).map(key => ({
          name: key,
          label: key,
          prop: key
        }));
        tableData.value = arrayData;

        headersByFile[basename] = Object.keys(arrayData[0]);
        treeHeaders.value = {
          ...treeHeaders.value,
          [basename]: headersByFile[basename]
        };
      } catch (err) {
        ElNotification({
          title: "Open file error",
          message: err.toString(),
          position: "bottom-right",
          type: "error",
          duration: 10000
        });
      }
    })
  );

  isDataLoaded.value = true; // 所有文件处理完成后设置加载完成标志

  return false;
}

// 处理文件路径，提取文件名
const viewFileName = computed(() => {
  const paths = data.path.split("|");
  return paths.map(path => {
    const pathParts = path.split(/[/\\]/); // 使用正则表达式匹配 / 或 \
    let fileName = pathParts[pathParts.length - 1]; // 获取文件名

    // 去除文件后缀
    const dotIndex = fileName.lastIndexOf(".");
    if (dotIndex !== -1 && dotIndex < fileName.length - 1) {
      // 确保 . 不是文件名的最后一个字符
      fileName = fileName.substring(0, dotIndex);
    }

    return fileName; // 返回不带后缀的文件名
  });
});

// 更新计算属性以检查加载状态
const fileTreeData = computed(() => {
  if (!isDataLoaded.value) return []; // 如果数据未加载完成，则返回空数组

  return viewFileName.value.map((fileName, index) => {
    const basename = fileName;
    const headers = headersByFile[basename] || [];
    const children = headers.map(header => ({
      label: header,
      key: `${basename}-${header}`
    }));

    return {
      label: fileName, // 文件名作为节点标签
      children: children,
      key: index
    };
  });
});

// 树形组件的配置
const defaultProps = {
  children: "children",
  label: "label"
};

const handleNodeClick = async data => {
  try {
    const textToCopy = JSON.stringify(data.label);
    if (navigator.clipboard) {
      await navigator.clipboard.writeText(textToCopy);
    }
  } catch (err) {
    console.error("Failed to copy to clipboard: ", err);
  }
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
    <el-form :style="{ height: formHeight + 'px' }">
      <div class="custom-container1">
        <div class="custom-container2">
          <el-button @click="selectViewFile()" :icon="FolderOpened" plain>
            Open File
          </el-button>
          <el-tooltip content="skip rows" placement="top" effect="light">
            <el-input
              v-model="data.skipRows"
              style="margin-left: 10px; width: 80px"
              placeholder="skip rows"
            />
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

        <el-button @click="viewTable = true" :icon="View" plain>
          View
        </el-button>
        <el-form-item>
          <el-tooltip
            content="Export data or not"
            placement="top"
            effect="light"
          >
            <el-switch
              v-model="data.write"
              inline-prompt
              style="
                --el-switch-on-color: #43cd80;
                --el-switch-off-color: #b0c4de;
              "
              active-text="Y"
              inactive-text="N"
              :active-action-icon="Download"
              :inactive-action-icon="View"
            />
          </el-tooltip>
          <el-tooltip content="Export type" placement="top" effect="light">
            <el-select
              v-model="data.writeFormat"
              style="margin-left: 10px; width: 95px"
            >
              <el-option label="csv" value="csv" />
              <el-option label="xlsx" value="xlsx" />
              <el-option label="parquet" value="parquet" />
            </el-select>
          </el-tooltip>
          <el-button
            @click="queryViewData"
            :loading="isLoading"
            :icon="Search"
            style="margin-left: 10px"
            plain
          >
            Execute
          </el-button>
        </el-form-item>
      </div>

      <div style="display: flex; height: calc(100% - 60px)">
        <div
          style="
            flex: 7.5 0 0%;
            padding: 10px;
            box-sizing: border-box;
            height: 100%;
          "
        >
          <el-form-item style="width: 100%; height: 100%">
            <VAceEditor
              v-model:value="sqlQuery"
              ref="editor"
              lang="sql"
              :options="{
                useWorker: true,
                enableBasicAutocompletion: true,
                enableSnippets: true,
                enableLiveAutocompletion: true,
                customScrollbar: true,
                showPrintMargin: false,
                fontSize: '1.0rem'
              }"
              :key="counter"
              @init="initializeEditor"
              :theme="theme"
              style="flex: 1 1 0%; height: 100%"
            />
          </el-form-item>
        </div>
        <div
          style="
            flex: 2.5 0 0%;
            padding: 10px;
            box-sizing: border-box;
            height: 100%;
          "
        >
          <el-scrollbar style="height: 100%">
            <el-tree
              :data="fileTreeData"
              :props="defaultProps"
              @node-click="handleNodeClick"
              style="height: 100%; overflow-y: auto"
            />
          </el-scrollbar>
        </div>
      </div>
    </el-form>

    <el-drawer
      v-model="viewTable"
      :with-header="false"
      :direction="'btt'"
      size="75%"
    >
      <el-scrollbar :height="formHeight * 0.8">
        <el-table
          :data="tableData"
          border
          empty-text=""
          style="width: 100%"
          :height="formHeight * 0.8"
        >
          <el-table-column
            v-for="column in columns"
            :prop="column.prop"
            :label="column.label"
            :key="column.prop"
          />
        </el-table>
      </el-scrollbar>
    </el-drawer>
  </el-form>
</template>

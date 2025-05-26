<script setup lang="ts">
import { ref, reactive, computed, markRaw, shallowRef } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Search, View, Download } from "@element-plus/icons-vue";
import { VAceEditor } from "vue3-ace-editor";
import { useDark } from "@pureadmin/utils";
import "./ace-config";
import { useDynamicHeight } from "@/utils/utils";
import { message } from "@/utils/message";

const currentPage = ref(1);
const pageSize = ref(10);
const total = ref(0);
const tableColumn = shallowRef<any[]>([]);
const treeHeaders = ref([]);
const tableData = shallowRef<any[]>([]);
const isLoading = ref(false);
const viewTable = ref(false);
const counter = ref(0);
const tables = ref([]);
const isDataLoaded = ref(false);
const headersByFile = reactive({});
const sqlQuery = ref("select\n*\nfrom _t_1\nlimit 100");
const data = reactive({
  path: "",
  write: false,
  writeFormat: "xlsx",
  skipRows: "0",
  schemaLength: "0"
});
const { dynamicHeight } = useDynamicHeight(84);
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

const queryViewData = async () => {
  const queryResult = await queryData();
  if (queryResult) {
    viewTable.value = true;
  }
};

const pagedTableData = computed(() => {
  return tableData.value.slice(
    (currentPage.value - 1) * pageSize.value,
    currentPage.value * pageSize.value
  );
});
const handleSizeChange = (newSize: number) => {
  pageSize.value = newSize;
  currentPage.value = 1;
};
const handleCurrentChange = (newPage: number) => {
  currentPage.value = newPage;
};

// invoke query
async function queryData() {
  tableColumn.value = [];
  tableData.value = [];
  currentPage.value = 1;
  total.value = 0;

  if (data.path === "") {
    message("File not selected", { type: "warning" });
    return false;
  }
  if (sqlQuery.value === "") {
    message("SQL script is empty", { type: "warning" });
    return false;
  }

  try {
    isLoading.value = true;

    const result: string[] = await invoke("query", {
      path: data.path,
      sqlQuery: sqlQuery.value,
      write: data.write,
      writeFormat: data.writeFormat,
      skipRows: data.skipRows,
      schemaLength: data.schemaLength
    });

    const q = Array.isArray(result[0]) ? result[0][0] : null;
    if (q.startsWith("Query failed")) {
      throw q;
    }

    const jsonData = JSON.parse(result[0]);
    const arrayData = Array.isArray(jsonData) ? jsonData : [jsonData];
    tableColumn.value = Object.keys(arrayData[0]).map(key => ({
      name: key,
      label: key,
      prop: key
    }));
    tableData.value = markRaw(arrayData);
    total.value = arrayData.length;

    message(`Query done, elapsed time: ${result[1]} s`, { type: "success" });
    isLoading.value = false;
    return true;
  } catch (err) {
    isLoading.value = false;
    message(err.toString(), { type: "error", duration: 10000 });
  }

  return false;
}

const selectViewFile = async () => {
  const selectedFile = await selectFile();
  if (selectedFile) {
    viewTable.value = true;
  }
};

async function selectFile() {
  tableColumn.value = [];
  treeHeaders.value = [];
  tableData.value = [];
  data.path = "";
  viewTable.value = false;
  currentPage.value = 1;
  total.value = 0;

  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "",
        extensions: ["*"]
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

  // 使用Promise.all并行处理每个文件
  await Promise.all(
    data.path.split("|").map(async (path, index) => {
      const basename = viewFileName.value[index];
      try {
        const result: string[] = await invoke("query", {
          path: path,
          sqlQuery: `select * from "${basename}" limit 10`,
          write: false,
          writeFormat: "csv",
          skipRows: data.skipRows,
          schemaLength: "0"
        });

        const q = Array.isArray(result[0]) ? result[0][0] : null;
        if (q.startsWith("Query failed")) {
          throw q;
        }

        const jsonData = JSON.parse(result[0]);
        const arrayData = Array.isArray(jsonData) ? jsonData : [jsonData];
        tableColumn.value = Object.keys(arrayData[0]).map(key => ({
          name: key,
          label: key,
          prop: key
        }));
        tableData.value = markRaw(arrayData);

        headersByFile[basename] = Object.keys(arrayData[0]);
        treeHeaders.value = {
          ...treeHeaders.value,
          [basename]: headersByFile[basename]
        };
      } catch (err) {
        message(err.toString(), { type: "error", duration: 10000 });
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
    const pathParts = path.split(/[/\\]/);
    let fileName = pathParts[pathParts.length - 1];

    // 去除文件后缀
    const dotIndex = fileName.lastIndexOf(".");
    if (dotIndex !== -1 && dotIndex < fileName.length - 1) {
      // 确保.不是文件名的最后一个字符
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
      label: fileName,
      children: children,
      key: index
    };
  });
});

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
    message(err.toString(), { type: "error", duration: 10000 });
  }
};
</script>

<template>
  <el-form class="page-container" :style="{ height: dynamicHeight + 'px' }">
    <div
      style="
        display: flex;
        flex-direction: column;
        height: calc(100% - 35px);
        overflow: hidden;
      "
    >
      <div style="flex: 0 0 50%; display: flex; overflow: hidden">
        <div
          style="
            flex: 3;
            box-sizing: border-box;
            height: 100%;
            overflow: hidden;
          "
        >
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
            style="height: 100%"
          />
        </div>
        <div
          style="
            flex: 1;
            box-sizing: border-box;
            height: 100%;
            overflow: hidden;
          "
        >
          <el-scrollbar style="height: 100%; overflow: auto">
            <el-tree
              :data="fileTreeData"
              :props="defaultProps"
              @node-click="handleNodeClick"
              empty-text=""
              style="max-height: 100%; overflow-y: auto"
            />
          </el-scrollbar>
        </div>
      </div>

      <div
        style="
          flex: 0 0 50%;
          box-sizing: border-box;
          overflow: hidden;
          display: flex;
          flex-direction: column;
          height: 100%;
        "
      >
        <el-table
          :data="pagedTableData"
          border
          empty-text=""
          style="width: 100%"
          show-overflow-tooltip
          :height="dynamicHeight * 0.45"
        >
          >
          <el-table-column
            v-for="column in tableColumn"
            :prop="column.prop"
            :label="column.label"
            :key="column.prop"
            width="150px"
          />
        </el-table>
      </div>
    </div>
    <div class="custom-container1" style="margin-bottom: -10px">
      <div class="custom-container2">
        <el-tooltip content="open file" effect="light">
          <el-button @click="selectViewFile()" :icon="FolderOpened" circle />
        </el-tooltip>
        <el-tooltip content="skip rows" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 30px"
          />
        </el-tooltip>
      </div>
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :pager-count="5"
        :total="total"
        layout="pager"
        hide-on-single-page
        :simplified="true"
        @size-change="handleSizeChange"
        @current-change="handleCurrentChange"
      />
      <el-form-item>
        <el-tooltip content="Export data or not" effect="light">
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
        <el-tooltip content="Export type" effect="light">
          <el-select
            v-model="data.writeFormat"
            style="margin-left: 10px; width: 70px"
          >
            <el-option label="csv" value="csv" />
            <el-option label="xlsx" value="xlsx" />
            <el-option label="parquet" value="parquet" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="execute" effect="light">
          <el-button
            @click="queryViewData"
            :loading="isLoading"
            :icon="Search"
            style="margin-left: 10px"
            circle
          />
        </el-tooltip>
      </el-form-item>
    </div>
  </el-form>
</template>

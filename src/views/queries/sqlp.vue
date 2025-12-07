<script setup lang="ts">
import { ref, reactive, computed, markRaw, shallowRef } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import {
  FolderOpened,
  ArrowRight,
  Download,
  View,
  Hide,
  Smoking,
  NoSmoking
} from "@element-plus/icons-vue";
import { VAceEditor } from "vue3-ace-editor";
import { useDark } from "@pureadmin/utils";
import "./ace-config";
import { useDynamicHeight } from "@/utils/utils";
import { message } from "@/utils/message";

const currentPage = ref(1);
const pageSize = ref(50);
const counter = ref(0);
const total = ref(0);
const tableColumn = shallowRef<any[]>([]);
const tables = ref([]);
const treeHeaders = ref([]);
const tableData = shallowRef<any[]>([]);
const isLoading = ref(false);
const viewTable = ref(false);
const isDataLoaded = ref(false);
const headersByFile = reactive<Record<string, string[]>>({});
const limitContent = ref("limit 500");
const varcharContent = ref("dtype: string");
const sqlQuery = ref("select\n*\nfrom _t_1\nlimit 100");
const data = reactive({
  path: "",
  write: false,
  writeFormat: "xlsx",
  skipRows: "0",
  limit: true,
  varchar: true
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

/**
 * 执行查询,invoke query(用于预览或导出)
 * @param write - 是否导出
 * @returns boolean - 是否成功
 */
async function executeQuery(write: boolean): Promise<boolean> {
  if (data.path === "" || sqlQuery.value === "") {
    message(data.path === "" ? "File not selected" : "SQL script is empty", {
      type: "warning"
    });
    return false;
  }

  // 仅预览时重置分页
  if (!write) {
    tableColumn.value = [];
    tableData.value = [];
    currentPage.value = 1;
    total.value = 0;
  }

  try {
    isLoading.value = true;
    const result = await invoke("query", {
      path: data.path,
      sqlQuery: sqlQuery.value,
      write,
      writeFormat: data.writeFormat,
      skipRows: data.skipRows,
      varchar: data.varchar,
      limit: data.limit
    });

    const q = result[0];
    if (q === "{}") throw new Error("Unsupported file type");

    const jsonData = JSON.parse(q);
    const arrayData = Array.isArray(jsonData) ? jsonData : [jsonData];

    // 仅在预览时更新表格
    if (!write) {
      tableColumn.value = Object.keys(arrayData[0]).map(key => ({
        name: key,
        label: key,
        prop: key
      }));
      tableData.value = markRaw(arrayData);
      total.value = arrayData.length;
    }

    if (write) {
      message(`Export done, elapsed time: ${result[1]} s`, { type: "success" });
    }

    return true;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 5000 });
    return false;
  } finally {
    isLoading.value = false;
  }
}

const queryData = () => executeQuery(false);
const exportData = () => executeQuery(true);
const selectViewFile = async () => {
  const fileSelect = await selectFile();
  if (fileSelect) {
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
        name: "All",
        extensions: ["*"]
      },
      {
        name: "csv",
        extensions: ["csv", "tsv", "psv", "txt", "dat"]
      },
      {
        name: "excel",
        extensions: ["xls", "xlsx", "xlsb", "xlsm", "ods"]
      },
      {
        name: "json",
        extensions: ["json"]
      },
      {
        name: "jsonl",
        extensions: ["jsonl", "ndjson"]
      },
      {
        name: "parquet",
        extensions: ["parquet"]
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
          varchar: true,
          limit: true
        });
        const q = result.length > 0 ? result[0] : null;
        if (q === "{}") {
          throw new Error(`${basename}: unsupported file type`);
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
        message(err.toString(), { type: "error", duration: 5000 });
      }
    })
  );

  isDataLoaded.value = true; // 所有文件处理完成后设置加载完成标志

  return false;
}

// 提取文件名
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

  // 只显示 headersByFile 中存在的文件
  return viewFileName.value
    .map((fileName, index) => {
      const basename = fileName;
      const headers = headersByFile[basename];
      if (!headers) return null;
      const children = headers.map(header => ({
        label: header,
        key: `${basename}-${header}`
      }));

      return {
        label: fileName,
        children: children,
        key: index
      };
    })
    .filter(Boolean); // 过滤掉null
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
    message(err.toString(), { type: "error", duration: 5000 });
  }
};

function limitRows() {
  data.limit = !data.limit;
  if (data.limit === true) {
    limitContent.value = "limit 500";
  } else {
    limitContent.value = "not limit";
  }
}

function allVarchar() {
  data.varchar = !data.varchar;
  if (data.varchar === true) {
    varcharContent.value = "dtype: string";
  } else {
    varcharContent.value = "dtype: auto";
  }
}
</script>

<template>
  <el-form class="page-container" :style="{ height: dynamicHeight + 'px' }">
    <div style="height: calc(100% - 0px)">
      <el-splitter>
        <el-splitter-panel size="150">
          <div style="display: flex; flex-direction: column; height: 100%">
            <el-tooltip content="Add data" effect="light">
              <el-button
                @click="selectViewFile()"
                :icon="FolderOpened"
                circle
                text
              />
            </el-tooltip>
            <el-scrollbar style="flex: 1">
              <el-tree
                :data="fileTreeData"
                :props="defaultProps"
                @node-click="handleNodeClick"
                empty-text=""
              />
            </el-scrollbar>
          </div>
        </el-splitter-panel>

        <el-splitter-panel>
          <el-splitter layout="vertical">
            <el-splitter-panel :collapsible="true">
              <div style="display: flex; flex-direction: column; height: 100%">
                <div style="display: flex; align-items: center">
                  <el-tooltip content="Run" effect="light">
                    <el-button
                      @click="queryViewData"
                      :loading="isLoading"
                      :icon="ArrowRight"
                      circle
                      text
                    />
                  </el-tooltip>
                  <el-tooltip :content="limitContent" effect="light">
                    <el-button @click="limitRows" circle text>
                      <el-icon>
                        <Hide v-if="data.limit" />
                        <View v-else />
                      </el-icon>
                    </el-button>
                  </el-tooltip>
                  <el-tooltip :content="varcharContent" effect="light">
                    <el-button @click="allVarchar" circle text>
                      <el-icon>
                        <NoSmoking v-if="data.varchar" />
                        <Smoking v-else />
                      </el-icon>
                    </el-button>
                  </el-tooltip>
                </div>

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
            </el-splitter-panel>

            <el-splitter-panel
              min="35"
              style="display: flex; flex-direction: column"
            >
              <div
                style="
                  flex: 1;
                  display: flex;
                  flex-direction: column;
                  overflow: hidden;
                "
              >
                <el-table :data="pagedTableData" height="100%">
                  <el-table-column
                    v-for="column in tableColumn"
                    :prop="column.prop"
                    :label="column.label"
                    :key="column.prop"
                    width="150px"
                  />
                </el-table>
                <div
                  style="
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    flex-shrink: 0;
                  "
                >
                  <el-pagination
                    v-model:current-page="currentPage"
                    v-model:page-size="pageSize"
                    :total="total"
                    layout="total, prev, next, jumper"
                    size="small"
                    :simplified="true"
                    @size-change="handleSizeChange"
                    @current-change="handleCurrentChange"
                  />
                  <div style="display: flex; align-items: center">
                    <el-tooltip content="Export type" effect="light">
                      <el-select
                        v-model="data.writeFormat"
                        size="small"
                        style="width: 90px"
                      >
                        <el-option label="csv" value="csv" />
                        <el-option label="xlsx" value="xlsx" />
                        <el-option label="parquet" value="parquet" />
                        <el-option label="json" value="json" />
                        <el-option label="jsonl" value="jsonl" />
                      </el-select>
                    </el-tooltip>
                    <el-tooltip content="Export" effect="light">
                      <el-button
                        @click="exportData"
                        :loading="isLoading"
                        :icon="Download"
                        circle
                        text
                      />
                    </el-tooltip>
                  </div>
                </div>
              </div>
            </el-splitter-panel>
          </el-splitter>
        </el-splitter-panel>
      </el-splitter>
    </div>
  </el-form>
</template>

<style>
.ace_gutter {
  background: transparent !important;
}
</style>

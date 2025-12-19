<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Event } from "@tauri-apps/api/event";
import { ElIcon } from "element-plus";
import {
  CloseBold,
  Select,
  FolderOpened,
  Loading,
  ArrowRight
} from "@element-plus/icons-vue";
import { useDark } from "@pureadmin/utils";
import {
  useDynamicHeight,
  filterFileStatus,
  ListenEvent,
  updateEvent
} from "@/utils/utils";
import { closeAllMessage, message } from "@/utils/message";
import { trimOpenFile } from "@/utils/view";

const [
  activeTab,
  chunksize,
  csvMode,
  progress,
  wtrSep,
  skipRows,
  quote,
  quoteStyle,
  encoding
] = [
  ref("excel"),
  ref("700000"),
  ref("csv"),
  ref("nil"),
  ref("|"),
  ref("0"),
  ref('"'),
  ref("necessary"),
  ref("gbk")
];
const [backendInfo, path] = [ref(""), ref("")];
const [sheetOptions, fileSheet] = [ref([]), ref([])];
const [allSheets, isLoading, backendCompleted, writeSheetname, ignoreErr] = [
  ref(true),
  ref(false),
  ref(false),
  ref(false),
  ref(false)
];
const modeOptions = [
  { label: "FormtCsv", value: "fmt" },
  { label: "EncodingCsv", value: "encoding" },
  { label: "Excel2Csv", value: "excel" },
  { label: "Csv2Xlsx", value: "csv" },
  { label: "Access2Csv", value: "access" },
  { label: "Dbf2Csv", value: "dbf" },
  { label: "Json2Csv", value: "json" },
  { label: "NdJson2Csv", value: "jsonl" }
];
const sheetsOptions = [
  { label: "All", value: true },
  { label: "One", value: false }
];
const writeOptions = [
  { label: "True", value: true },
  { label: "False", value: false }
];
const sepOptions = [
  { label: "|", value: "|" },
  { label: "\\t", value: "\t" },
  { label: ",", value: "," },
  { label: ";", value: ";" }
];
const quoteOptions = [
  { label: "'", value: "'" },
  { label: '"', value: '"' }
];
const pgsOptions = [
  { label: "Nil", value: "nil" },
  { label: "Idx", value: "idx" }
];
const csvModeOptions = [
  { label: "Csv", value: "csv" },
  { label: "Polars", value: "polars" }
];
const iErrOptions = [
  { label: "True", value: true },
  { label: "False", value: false }
];
const sheetsData = ref({});
const fileSelect = ref<ListenEvent[]>([]);
const { dynamicHeight } = useDynamicHeight(122);
const { isDark } = useDark();

listen("update-msg", (event: Event<string>) => {
  const [filename, rows] = event.payload.split("|");
  updateEvent(fileSelect, filename, file => {
    file.currentRows = rows;
  });
});
listen("total-msg", (event: Event<string>) => {
  const [filename, rows] = event.payload.split("|");
  updateEvent(fileSelect, filename, file => {
    file.totalRows = rows;
  });
});
listen("info", (event: Event<string>) => {
  const filename = event.payload;
  updateEvent(fileSelect, filename, file => {
    file.status = "loading";
  });
});
listen("err", (event: Event<string>) => {
  const [filename, message] = event.payload.split("|");
  updateEvent(fileSelect, filename, file => {
    file.status = "error";
    file.message = message;
  });
});
listen("success", (event: Event<string>) => {
  const filename = event.payload;
  updateEvent(fileSelect, filename, file => {
    file.status = "success";
  });
});

const getSheetsForFile = fileName => {
  return sheetsData.value[fileName] || [];
};

watch(
  () => fileSelect.value.map(file => file.selectSheet),
  (newVal, oldVal) => {
    newVal.forEach((selectSheet, index) => {
      if (selectSheet !== oldVal?.[index]) {
        const fileSheetRecord = {
          filename: fileSelect.value[index].filename,
          sheetname: selectSheet
        };
        const existingIndex = fileSheet.value.findIndex(
          record => record.filename === fileSheetRecord.filename
        );
        if (existingIndex > -1) {
          fileSheet.value.splice(existingIndex, 1);
        }
        fileSheet.value.push(fileSheetRecord);
      }
    });
  },
  { deep: true }
);

function updateFileSheet(file: ListenEvent) {
  if (!file.selectSheet) return;
  const existingIndex = fileSheet.value.findIndex(
    record => record.filename === file.filename
  );
  if (existingIndex > -1) {
    fileSheet.value[existingIndex].sheetname = file.selectSheet;
  } else {
    fileSheet.value.push({
      filename: file.filename,
      sheetname: file.selectSheet
    });
  }
}

async function selectFile() {
  fileSelect.value = [];
  sheetsData.value = [];
  sheetOptions.value = [];
  fileSheet.value = [];
  backendCompleted.value = false;
  backendInfo.value = "";
  try {
    const trimFile = await trimOpenFile(true, "Files", ["*"], {
      includeStatus: true
    });
    path.value = trimFile.filePath;
    fileSelect.value = trimFile.fileInfo;

    if (activeTab.value === "excel") {
      message("get excel sheets...", {
        type: "info",
        duration: 0,
        icon: Loading
      });
      const mapSheets: string[] = await invoke("map_excel_sheets", {
        path: path.value
      });
      sheetsData.value = mapSheets[0];
      for (const fileName in sheetsData.value) {
        sheetsData.value[fileName].forEach(sheet => {
          sheetOptions.value.push({
            label: `${fileName} - ${sheet}`,
            value: sheet
          });
        });
      }
      fileSelect.value.forEach(file => {
        if (!file.selectSheet && getSheetsForFile(file.filename).length > 0) {
          file.selectSheet = getSheetsForFile(file.filename)[0];
        }
        file.sheets = getSheetsForFile(file.filename);
      });
      closeAllMessage();
      backendInfo.value = "get excel sheets done";
      backendCompleted.value = true;
    }
  } catch (err) {
    closeAllMessage();
    message(err.toString(), { type: "error" });
  }
}

// invoke convert
async function convert() {
  if (path.value === "") {
    message("File not selected", { type: "warning" });
    return;
  }
  try {
    isLoading.value = true;
    let rtime: string;
    if (activeTab.value === "excel") {
      const mapFileSheet = fileSheet.value.map(item => ({
        filename: item.filename,
        sheetname: item.sheetname
      }));
      rtime = await invoke("excel2csv", {
        path: path.value,
        skipRows: skipRows.value,
        mapFileSheet: mapFileSheet,
        allSheets: allSheets.value,
        writeSheetname: writeSheetname.value
      });
    } else if (activeTab.value === "fmt") {
      rtime = await invoke("csv2csv", {
        path: path.value,
        wtrSep: wtrSep.value,
        quote: quote.value,
        quoteStyle: quoteStyle.value,
        progress: progress.value
      });
      console.log(progress.value);
    } else if (activeTab.value === "encoding") {
      rtime = await invoke("encoding2utf8", {
        path: path.value,
        encoding: encoding.value
      });
      console.log(progress.value);
    } else if (activeTab.value === "access") {
      rtime = await invoke("access2csv", {
        path: path.value,
        wtrSep: wtrSep.value
      });
    } else if (activeTab.value === "dbf") {
      rtime = await invoke("dbf2csv", {
        path: path.value,
        wtrSep: wtrSep.value
      });
    } else if (activeTab.value === "csv") {
      rtime = await invoke("csv2xlsx", {
        path: path.value,
        csvMode: csvMode.value,
        chunksize: chunksize.value
      });
    } else if (activeTab.value === "json") {
      rtime = await invoke("json2csv", {
        path: path.value,
        wtrSep: wtrSep.value
      });
    } else if (activeTab.value === "jsonl") {
      rtime = await invoke("jsonl2csv", {
        path: path.value,
        wtrSep: wtrSep.value,
        ignoreErr: ignoreErr.value
      });
    }
    message(`${activeTab.value} done, elapsed time: ${rtime} s`, {
      type: "success"
    });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container">
    <el-splitter>
      <el-splitter-panel size="240" :resizable="false">
        <div class="splitter-container">
          <el-tooltip content="Add data" effect="light" placement="right">
            <el-button @click="selectFile()" :icon="FolderOpened" circle text />
          </el-tooltip>

          <!-- mode choice -->
          <div class="mode-toggle-v" style="margin-bottom: 8px">
            <span
              v-for="item in modeOptions"
              :key="item.value"
              class="mode-item"
              :class="{
                active: activeTab === item.value,
                'active-dark': isDark && activeTab === item.value
              }"
              @click="activeTab = item.value"
            >
              {{ item.label }}
            </span>
          </div>

          <!-- excel to csv -->
          <el-tooltip
            v-if="activeTab === 'excel'"
            content="Convert all sheets or not"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle">
              <span
                v-for="item in sheetsOptions"
                :key="String(item.value)"
                class="mode-item"
                :class="{
                  active: allSheets === item.value,
                  'active-dark': isDark && allSheets === item.value
                }"
                @click="allSheets = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <el-tooltip
            v-if="activeTab === 'excel'"
            content="Write sheet name or not"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle" style="margin-top: 8px">
              <span
                v-for="item in writeOptions"
                :key="String(item.value)"
                class="mode-item"
                :class="{
                  active: writeSheetname === item.value,
                  'active-dark': isDark && writeSheetname === item.value
                }"
                @click="writeSheetname = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <el-tooltip content="Skip rows" effect="light" placement="right">
            <el-input
              v-model="skipRows"
              v-if="activeTab === 'excel'"
              style="width: 220px; margin-top: 8px; margin-left: 8px"
            />
          </el-tooltip>

          <!-- format csv -->
          <el-tooltip
            v-if="!new Set(['excel', 'csv', 'encoding']).has(activeTab)"
            content="Write delimiter"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle">
              <span
                v-for="item in sepOptions"
                :key="item.value"
                class="mode-item"
                :class="{
                  active: wtrSep === item.value,
                  'active-dark': isDark && wtrSep === item.value
                }"
                @click="wtrSep = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <el-tooltip
            v-if="activeTab === 'fmt'"
            content="Quote character"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle" style="margin-top: 8px">
              <span
                v-for="item in quoteOptions"
                :key="item.value"
                class="mode-item"
                :class="{
                  active: quote === item.value,
                  'active-dark': isDark && quote === item.value
                }"
                @click="quote = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <el-tooltip
            v-if="activeTab === 'fmt'"
            content="if Nil, no progress bar"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle" style="margin-top: 8px">
              <span
                v-for="item in pgsOptions"
                :key="item.value"
                class="mode-item"
                :class="{
                  active: progress === item.value,
                  'active-dark': isDark && progress === item.value
                }"
                @click="progress = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <el-tooltip content="Quote style" effect="light" placement="right">
            <el-select
              v-model="quoteStyle"
              style="width: 220px; margin-left: 8px; margin-top: 8px"
              v-if="activeTab === 'fmt'"
            >
              <el-option label="Necessary" value="necessary" />
              <el-option label="Always" value="always" />
              <el-option label="NonNumeric" value="non_numeric" />
              <el-option label="Never" value="never" />
            </el-select>
          </el-tooltip>

          <!-- csv to xlsx -->
          <div class="mode-toggle" v-if="activeTab === 'csv'">
            <span
              v-for="item in csvModeOptions"
              :key="item.value"
              class="mode-item"
              :class="{
                active: csvMode === item.value,
                'active-dark': isDark && csvMode === item.value
              }"
              @click="csvMode = item.value"
            >
              {{ item.label }}
            </span>
          </div>

          <el-tooltip
            content="Split every N rows into a sheet"
            effect="light"
            placement="right"
          >
            <el-input
              v-model="chunksize"
              v-if="activeTab === 'csv' && csvMode === 'csv'"
              style="margin-left: 8px; margin-top: 8px; width: 220px"
            />
          </el-tooltip>

          <!-- jsonl to csv -->
          <el-tooltip
            v-if="activeTab === 'jsonl'"
            content="Ignore errors"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle" style="margin-top: 8px">
              <span
                v-for="item in iErrOptions"
                :key="String(item.value)"
                class="mode-item"
                :class="{
                  active: ignoreErr === item.value,
                  'active-dark': isDark && ignoreErr === item.value
                }"
                @click="ignoreErr = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <!-- encoding -->
          <el-tooltip content="Read Encoding" effect="light" placement="right">
            <el-select
              v-model="encoding"
              style="margin-left: 8px; width: 220px"
              v-if="activeTab === 'encoding'"
            >
              <el-option label="GBK" value="gbk" />
              <el-option label="UTF-8" value="utf_8" />
              <el-option label="UTF-16LE" value="utf_16le" />
              <el-option label="UTF-16BE" value="utf_16be" />
            </el-select>
          </el-tooltip>

          <text
            v-if="backendCompleted && activeTab === 'excel'"
            style="margin-left: 8px; margin-top: auto"
          >
            {{ backendInfo }}
          </text>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-tooltip content="Run" effect="light" placement="right">
          <el-button
            @click="convert()"
            :loading="isLoading"
            :icon="ArrowRight"
            circle
            text
          />
        </el-tooltip>

        <el-table
          :data="fileSelect"
          :height="dynamicHeight"
          style="width: 100%"
          show-overflow-tooltip
          empty-text=""
        >
          <el-table-column type="index" width="35" />
          <el-table-column
            prop="filename"
            label="File"
            :class="{ 'custom-width': true }"
            style="flex: 0 0 30%"
          />
          <el-table-column
            prop="status"
            label="Status"
            :filters="[
              { text: 'x', value: 'error' },
              { text: '√', value: 'success' }
            ]"
            :filter-method="filterFileStatus"
            :class="{ 'custom-width': true }"
            style="flex: 0 0 10%"
          >
            <template #default="scope">
              <ElIcon v-if="scope.row.status === 'loading'" class="is-loading">
                <Loading />
              </ElIcon>
              <ElIcon
                v-else-if="scope.row.status === 'success'"
                color="#00CD66"
              >
                <Select />
              </ElIcon>
              <ElIcon v-else-if="scope.row.status === 'error'" color="#FF0000">
                <CloseBold />
              </ElIcon>
              <span
                v-if="
                  scope.row.message &&
                  scope.row.status !== 'loading' &&
                  activeTab === 'excel'
                "
              >
                {{ scope.row.message || scope.row.status }}
              </span>
            </template>
          </el-table-column>
          <el-table-column
            prop="message"
            label="Message"
            :class="{ 'custom-width': true }"
            style="flex: 0 0 60%"
          >
            <template #default="scope">
              <template v-if="activeTab === 'excel'">
                <el-select
                  v-model="scope.row.selectSheet"
                  placeholder="Select a sheet"
                  @change="updateFileSheet(scope.row)"
                  style="width: 100%"
                >
                  <el-option
                    v-for="sheet in scope.row.sheets"
                    :key="sheet"
                    :label="sheet"
                    :value="sheet"
                  />
                </el-select>
              </template>
              <template v-else>
                <span v-if="scope.row.status === 'error'">
                  {{ scope.row.message }}
                </span>
                <el-progress
                  v-else-if="
                    activeTab === 'fmt' &&
                    scope.row.totalRows > 0 &&
                    isFinite(scope.row.currentRows / scope.row.totalRows)
                  "
                  :percentage="
                    Math.round(
                      (scope.row.currentRows / scope.row.totalRows) * 100
                    )
                  "
                />
              </template>
            </template>
          </el-table-column>
        </el-table>
      </el-splitter-panel>
    </el-splitter>
  </el-form>
</template>

<style scoped>
.mode-toggle {
  width: 220px;
}
.mode-toggle-v {
  width: 220px;
  height: 128px;
}
</style>

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
  SwitchButton
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
import { useQuoting } from "@/store/modules/options";

const [
  activeTab,
  chunksize,
  csvMode,
  progress,
  wtrSep,
  skipRows,
  quote,
  quoteStyle
] = [
  ref("excel"),
  ref("700000"),
  ref("csv"),
  ref("nil"),
  ref("|"),
  ref("0"),
  ref('"'),
  ref("necessary")
];
const [backendInfo, path] = [ref(""), ref("")];
const [sheetOptions, fileSheet] = [ref([]), ref([])];
const [allSheets, isLoading, backendCompleted, writeSheetname, ignoreErr, bom] =
  [ref(true), ref(false), ref(false), ref(false), ref(false), ref(false)];
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
const fmtOptions = [
  { label: "Necessary", value: "necessary" },
  { label: "Always", value: "always" },
  { label: "NonNumeric", value: "non_numeric" },
  { label: "Never", value: "never" }
];
const bomOptions = [
  { label: "True", value: true },
  { label: "False", value: false }
];
const sheetsData = ref({});
const fileSelect = ref<ListenEvent[]>([]);
const { dynamicHeight } = useDynamicHeight(74);
const { isDark } = useDark();
const quotingStore = useQuoting();

interface ExcelSheetMap {
  [filename: string]: string[];
}
const mapSheets = ref<ExcelSheetMap | null>(null);

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

const filterBySheetCount = (filterValue: string, row: any): boolean => {
  const sheetCount = row.sheets?.length ?? 0;

  switch (filterValue) {
    case "zero":
      return sheetCount === 0;
    case "one":
      return sheetCount === 1;
    case "many":
      return sheetCount > 1;
    default:
      return true;
  }
};

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
      mapSheets.value = await invoke<ExcelSheetMap>("map_excel_sheets", {
        path: path.value
      });
      sheetsData.value = mapSheets.value[0];
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
        quoting: quotingStore.quoting,
        progress: progress.value
      });
    } else if (activeTab.value === "encoding") {
      rtime = await invoke("encoding2utf8", {
        path: path.value,
        bom: bom.value,
        quoting: quotingStore.quoting
      });
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
        chunksize: chunksize.value,
        quoting: quotingStore.quoting
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
          <el-button @click="selectFile()" :icon="FolderOpened" text round>
            Open File(s)
          </el-button>

          <!-- mode choice -->
          <div class="mode-toggle-v mb-2 w-[220px] h-[128px]">
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
            <div class="mode-toggle w-[220px]">
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
            <div class="mode-toggle mt-2 w-[220px]">
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

          <el-tooltip
            v-if="activeTab === 'excel'"
            content="Skip rows"
            effect="light"
            placement="right"
          >
            <el-input
              v-model="skipRows"
              class="mt-2 ml-2"
              style="width: 220px"
            />
          </el-tooltip>

          <!-- format csv -->
          <el-tooltip
            v-if="!new Set(['excel', 'csv', 'encoding']).has(activeTab)"
            content="Write delimiter"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle w-[220px]">
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
            <div class="mode-toggle mt-2 w-[220px]">
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
            <div class="mode-toggle mt-2 w-[220px]">
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

          <el-tooltip
            v-if="activeTab === 'fmt'"
            content="Quote style"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle-v mt-2 w-[220px] h-[64px]">
              <span
                v-for="item in fmtOptions"
                :key="item.value"
                class="mode-item"
                :class="{
                  active: quoteStyle === item.value,
                  'active-dark': isDark && quoteStyle === item.value
                }"
                @click="quoteStyle = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <!-- csv to xlsx -->
          <div class="mode-toggle w-[220px]" v-if="activeTab === 'csv'">
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
            v-if="activeTab === 'csv' && csvMode === 'csv'"
            content="Split every N rows into a sheet"
            effect="light"
            placement="right"
          >
            <el-input
              v-model="chunksize"
              class="ml-2 mt-2"
              style="width: 220px"
            />
          </el-tooltip>

          <!-- jsonl to csv -->
          <el-tooltip
            v-if="activeTab === 'jsonl'"
            content="Ignore errors"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle mt-2 w-[220px]">
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
          <el-tooltip
            v-if="activeTab === 'encoding'"
            content="BOM"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle-v mt-2 w-[220px] h-[32px]">
              <span
                v-for="item in bomOptions"
                :key="String(item.value)"
                class="mode-item"
                :class="{
                  active: bom === item.value,
                  'active-dark': isDark && bom === item.value
                }"
                @click="bom = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <text
            v-if="backendCompleted && activeTab === 'excel'"
            class="ml-2 mt-auto"
          >
            {{ backendInfo }}
          </text>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-button
          @click="convert()"
          :loading="isLoading"
          :icon="SwitchButton"
          text
          round
          >Run
        </el-button>

        <el-table
          :data="fileSelect"
          :height="dynamicHeight"
          show-overflow-tooltip
          tooltip-effect="light"
        >
          <el-table-column type="index" width="35" />
          <el-table-column prop="filename" label="File" />
          <el-table-column
            prop="status"
            label="Status"
            :filters="[
              { text: 'x', value: 'error' },
              { text: '√', value: 'success' }
            ]"
            :filter-method="filterFileStatus"
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
            :filters="[
              { text: 'No worksheet', value: 'zero' },
              { text: '1 worksheet', value: 'one' },
              { text: 'Multiple worksheets', value: 'many' }
            ]"
            :filter-method="filterBySheetCount"
          >
            <template #default="scope">
              <template v-if="activeTab === 'excel'">
                <el-select
                  v-model="scope.row.selectSheet"
                  placeholder="Select a sheet"
                  @change="updateFileSheet(scope.row)"
                  :disabled="!scope.row.sheets || scope.row.sheets.length === 0"
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

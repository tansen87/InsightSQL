<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Event } from "@tauri-apps/api/event";
import { ElIcon } from "element-plus";
import {
  CloseBold,
  Select,
  FolderOpened,
  SwitchFilled,
  Loading
} from "@element-plus/icons-vue";
import {
  useDynamicHeight,
  filterFileStatus,
  ListenEvent,
  updateEvent
} from "@/utils/utils";
import { closeAllMessage, message } from "@/utils/message";
import { trimOpenFile } from "@/utils/view";

const [activeTab, chunksize, csvMode, progress, wtrSep, skipRows] = [
  ref("excel"),
  ref("700000"),
  ref("csv"),
  ref("nil"),
  ref("|"),
  ref("0")
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
const sheetsData = ref({});
const fileSelect = ref<ListenEvent[]>([]);
const toTab = computed(() => activeTab.value);
const { dynamicHeight } = useDynamicHeight(176);

listen("update-rows", (event: Event<string>) => {
  const [filename, rows] = event.payload.split("|");
  updateEvent(fileSelect, filename, file => {
    file.currentRows = rows;
  });
});
listen("total-rows", (event: Event<string>) => {
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

    if (toTab.value === "excel") {
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
    if (toTab.value === "excel") {
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
    } else if (toTab.value === "fmt") {
      rtime = await invoke("csv2csv", {
        path: path.value,
        wtrSep: wtrSep.value,
        progress: progress.value
      });
    } else if (toTab.value === "access") {
      rtime = await invoke("access2csv", {
        path: path.value,
        wtrSep: wtrSep.value
      });
    } else if (toTab.value === "dbf") {
      rtime = await invoke("dbf2csv", {
        path: path.value,
        wtrSep: wtrSep.value
      });
    } else if (toTab.value === "csv") {
      rtime = await invoke("csv2xlsx", {
        path: path.value,
        csvMode: csvMode.value,
        chunksize: chunksize.value
      });
    } else if (toTab.value === "json") {
      rtime = await invoke("json2csv", {
        path: path.value,
        wtrSep: wtrSep.value
      });
    } else if (toTab.value === "jsonl") {
      rtime = await invoke("jsonl2csv", {
        path: path.value,
        wtrSep: wtrSep.value,
        ignoreErr: ignoreErr.value
      });
    }
    message(`${toTab.value} done, elapsed time: ${rtime} s`, {
      type: "success"
    });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <div class="page-container">
    <el-tabs v-model="activeTab">
      <el-tab-pane name="excel" label="Excel2Csv" />
      <el-tab-pane name="fmt" label="FmtCsv" />
      <el-tab-pane name="access" label="Access2Csv" />
      <el-tab-pane name="dbf" label="Dbf2Csv" />
      <el-tab-pane name="csv" label="Csv2Xlsx" />
      <el-tab-pane name="json" label="Json2Csv" />
      <el-tab-pane name="jsonl" label="Jsonl2Csv" />
    </el-tabs>
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
        <el-tooltip content="Convert all sheets or not" effect="light">
          <el-select
            v-model="allSheets"
            v-if="activeTab === 'excel'"
            style="width: 75px; margin-right: 8px; margin-left: 8px"
          >
            <el-option label="All" :value="true" />
            <el-option label="One" :value="false" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="Write sheet name or not" effect="light">
          <el-select
            v-model="writeSheetname"
            v-if="activeTab === 'excel'"
            style="width: 55px; margin-right: 8px"
          >
            <el-option label="Y" :value="true" />
            <el-option label="N" :value="false" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="Skip rows" effect="light">
          <el-input
            v-model="skipRows"
            v-if="activeTab === 'excel'"
            style="width: 50px"
          />
        </el-tooltip>
        <span
          v-if="backendCompleted && activeTab === 'excel'"
          style="margin-left: 8px"
          >{{ backendInfo }}</span
        >
        <el-tooltip content="Write delimiter" effect="light">
          <el-select
            v-model="wtrSep"
            style="width: 52px; margin-left: 8px"
            v-if="!new Set(['excel', 'csv']).has(activeTab)"
          >
            <el-option label="|" value="|" />
            <el-option label="\t" value="\t" />
            <el-option label="," value="," />
            <el-option label=";" value=";" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="If nil, no progress" effect="light">
          <el-select
            v-model="progress"
            style="margin-left: 8px; width: 70px"
            v-if="activeTab === 'fmt'"
          >
            <el-option label="idx" value="idx" />
            <el-option label="nil" value="nil" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="Polars or Csv engine" effect="light">
          <el-select
            v-model="csvMode"
            v-if="activeTab === 'csv'"
            style="margin-left: 8px; width: 85px"
          >
            <el-option label="Csv" value="csv" />
            <el-option label="Polars" value="polars" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="Split every N rows into a sheet" effect="light">
          <el-input
            v-model="chunksize"
            v-if="activeTab === 'csv' && csvMode === 'csv'"
            style="margin-left: 8px; width: 80px"
          />
        </el-tooltip>
        <el-tooltip content="Ignore errors" effect="light">
          <el-select
            v-model="ignoreErr"
            style="margin-left: 8px; width: 75px"
            v-if="activeTab === 'jsonl'"
          >
            <el-option label="true" :value="true" />
            <el-option label="false" :value="false" />
          </el-select>
        </el-tooltip>
      </div>
      <el-button @click="convert()" :loading="isLoading" :icon="SwitchFilled">
        Convert
      </el-button>
    </div>
    <el-table
      :data="fileSelect"
      :height="dynamicHeight"
      style="width: 100%"
      show-overflow-tooltip
      empty-text=""
    >
      <el-table-column type="index" width="50" />
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
          <ElIcon v-else-if="scope.row.status === 'success'" color="#00CD66">
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
                Math.round((scope.row.currentRows / scope.row.totalRows) * 100)
              "
            />
          </template>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

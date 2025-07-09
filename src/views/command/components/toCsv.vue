<script setup lang="ts">
import { ref, reactive, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElIcon } from "element-plus";
import {
  CloseBold,
  Select,
  FolderOpened,
  SwitchFilled,
  Loading
} from "@element-plus/icons-vue";
import { useDynamicHeight, filterFileStatus } from "@/utils/utils";
import { closeAllMessage, message } from "@/utils/message";
import { trimOpenFile } from "@/utils/view";

const btnShow = ref("CONVERT ALL");
const typeTo = ref("excel");
const [backendInfo, path] = [ref(""), ref("")];
const [selectedFiles, sheetOptions, fileSheet] = [ref([]), ref([]), ref([])];
const [allSheets, isLoading, backendCompleted, writeSheetname] = [
  ref(true),
  ref(false),
  ref(false),
  ref(false)
];
const sheetsData = ref({});
const data = reactive({
  skipRows: "0",
  sep: "|",
  mode: "nil"
});
const { dynamicHeight } = useDynamicHeight(172);
watch(
  () => allSheets.value,
  val => {
    if (val === true) {
      btnShow.value = "CONVERT ALL";
    } else if (val === false) {
      btnShow.value = "CONVERT ONE";
    }
  }
);

listen("update-rows", (event: any) => {
  const [backFilename, rows] = event.payload.split("|");
  selectedFiles.value.forEach(file => {
    if (file.filename === backFilename) {
      file.currentRows = rows;
    }
  });
});
listen("total-rows", (event: any) => {
  const [backFilename, rows] = event.payload.split("|");
  selectedFiles.value.forEach(file => {
    if (file.filename === backFilename) {
      file.totalRows = rows;
    }
  });
});
listen("start-to", event => {
  const startConvert: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert) {
      file.status = "loading";
    }
  });
});
listen("to-err", event => {
  const rowCountErr: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === rowCountErr.split("|")[0]) {
      file.status = "error";
      file.errMessage = rowCountErr.split("|")[1];
    }
  });
});
listen("to-msg", (event: any) => {
  const e2cMsg: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === e2cMsg) {
      file.status = "success";
    }
  });
});

const getSheetsForFile = fileName => {
  return sheetsData.value[fileName] || [];
};

watch(
  () => selectedFiles.value.map(file => file.selectedSheet),
  (newVal, oldVal) => {
    newVal.forEach((selectedSheet, index) => {
      if (selectedSheet !== oldVal[index]) {
        const fileSheetRecord = {
          filename: selectedFiles.value[index].filename,
          sheetname: selectedSheet
        };

        fileSheet.value.push(fileSheetRecord);
      }
    });
  },
  { deep: true }
);

function updateFileSheet(file) {
  const existingRecordIndex = fileSheet.value.findIndex(
    record => record.filename === file.filename
  );
  if (existingRecordIndex > -1) {
    fileSheet.value[existingRecordIndex].sheetname = file.selectedSheet;
  } else {
    fileSheet.value.push({
      filename: file.filename,
      sheetname: file.selectedSheet
    });
  }
}

async function selectFile() {
  selectedFiles.value = [];
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
    selectedFiles.value = trimFile.fileInfo;

    if (typeTo.value === "excel") {
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
      selectedFiles.value.forEach(file => {
        if (!file.selectedSheet && getSheetsForFile(file.filename).length > 0) {
          file.selectedSheet = getSheetsForFile(file.filename)[0];
        }
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

// invoke excel2csv
async function toCsv() {
  if (path.value === "") {
    message("Excel file not selected", { type: "warning" });
    return;
  }
  try {
    isLoading.value = true;
    if (typeTo.value === "excel") {
      const mapFileSheet = fileSheet.value.map(item => ({
        filename: item.filename,
        sheetname: item.sheetname
      }));
      const rtime: string = await invoke("excel2csv", {
        path: path.value,
        skipRows: data.skipRows,
        mapFileSheet: mapFileSheet,
        allSheets: allSheets.value,
        writeSheetname: writeSheetname.value
      });
      message(`Done, elapsed time: ${rtime} s`, { type: "success" });
    } else if (typeTo.value === "csv") {
      const rtime: string = await invoke("csv2csv", {
        path: path.value,
        sep: data.sep,
        mode: data.mode
      });
      message(`Done, elapsed time: ${rtime} s`, { type: "success" });
    } else if (typeTo.value === "access") {
      const rtime: string = await invoke("access2csv", {
        path: path.value,
        sep: data.sep
      });
      message(`Done, elapsed time: ${rtime} s`, { type: "success" });
    }
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container" :style="dynamicHeight">
    <div class="custom-container1">
      <el-button @click="selectFile()" :icon="FolderOpened">
        Open File
      </el-button>
      <el-form-item>
        <el-select v-model="typeTo" style="width: 95px; margin-right: 10px">
          <el-option label="Excel" value="excel" />
          <el-option label="CSV" value="csv" />
          <el-option label="Access" value="access" />
        </el-select>
        <span> TO CSV </span>
      </el-form-item>
    </div>

    <div class="custom-container1">
      <div class="custom-container2" v-if="typeTo === 'excel'">
        <el-tooltip content="Convert all sheets or not" effect="light">
          <el-select
            v-model="allSheets"
            style="width: 75px; margin-right: 10px"
          >
            <el-option label="All" :value="true" />
            <el-option label="One" :value="false" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="write sheetname or not" effect="light">
          <el-select
            v-model="writeSheetname"
            style="width: 55px; margin-right: 10px"
          >
            <el-option label="Y" :value="true" />
            <el-option label="N" :value="false" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="skip rows" effect="light">
          <el-input v-model="data.skipRows" style="width: 50px" />
        </el-tooltip>
      </div>
      <div class="custom-container2" v-if="typeTo !== 'excel'">
        <el-tooltip content="write delimiter" effect="light">
          <el-select v-model="data.sep" style="width: 50px">
            <el-option label="|" value="|" />
            <el-option label="\t" value="\t" />
            <el-option label="," value="," />
            <el-option label=";" value=";" />
          </el-select>
        </el-tooltip>
      </div>
      <div class="custom-container2">
        <el-tooltip content="if nil, do not add progress bar" effect="light">
          <el-select
            v-if="typeTo === 'csv'"
            v-model="data.mode"
            style="margin-left: 10px; width: 70px"
          >
            <el-option label="idx" value="idx" />
            <el-option label="std" value="std" />
            <el-option label="nil" value="nil" />
          </el-select>
        </el-tooltip>
        <span v-if="backendCompleted && typeTo === 'excel'">
          {{ backendInfo }}
        </span>
      </div>
      <el-button
        @click="toCsv()"
        :loading="isLoading"
        :icon="SwitchFilled"
        style="width: 157px"
      >
        {{ btnShow }}
      </el-button>
    </div>

    <el-table
      :data="selectedFiles"
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
              scope.row.errMessage &&
              scope.row.status !== 'loading' &&
              typeTo === 'excel'
            "
          >
            {{ scope.row.errMessage || scope.row.status }}
          </span>
        </template>
      </el-table-column>
      <el-table-column
        v-if="typeTo === 'excel'"
        prop="message"
        label="Message"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 60%"
      >
        <template #default="scope">
          <el-select
            v-model="scope.row.selectedSheet"
            placeholder="Select a sheet"
            @change="updateFileSheet(scope.row)"
          >
            <el-option
              v-for="(sheet, index) in getSheetsForFile(scope.row.filename)"
              :key="index"
              :label="sheet"
              :value="sheet"
            />
          </el-select>
        </template>
      </el-table-column>
      <el-table-column
        v-if="typeTo !== 'excel'"
        prop="errMessage"
        label="Message"
        :class="{ 'custom-width': true }"
        style="flex: 0 0 60%"
      >
        <template #default="scope">
          <span v-if="scope.row.status === 'error'">
            {{ scope.row.errMessage }}
          </span>
          <el-progress
            v-if="
              scope.row.totalRows !== 0 &&
              isFinite(scope.row.currentRows / scope.row.totalRows) &&
              typeTo === 'csv'
            "
            :percentage="
              Math.round((scope.row.currentRows / scope.row.totalRows) * 100)
            "
          />
        </template>
      </el-table-column>
    </el-table>
  </el-form>
</template>

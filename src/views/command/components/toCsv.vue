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
  Loading,
  Open,
  TurnOff
} from "@element-plus/icons-vue";
import { useDynamicHeight, filterFileStatus } from "@/utils/utils";
import { closeAllMessage, message } from "@/utils/message";
import { trimOpenFile } from "@/utils/view";

const btnShow = ref("CONVERT ONE");
const backendInfo = ref("");
const [selectedFiles, sheetOptions, fileSheet] = [ref([]), ref([]), ref([])];
const [isLoading, backendCompleted] = [ref(false), ref(false)];
const sheetsData = ref({});
const data = reactive({
  path: "",
  fileFormats: ["xlsx", "xls", "xlsb", "xlsm", "xlam", "xla", "ods"],
  skipRows: "0",
  allSheets: false,
  writeSheetname: false
});
const { dynamicHeight } = useDynamicHeight(172);
watch(
  () => data.allSheets,
  val => {
    if (val === true) {
      btnShow.value = "CONVERT ALL";
    } else if (val === false) {
      btnShow.value = "CONVERT ONE";
    }
  }
);

listen("start_convert", event => {
  const startConvert: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert) {
      file.status = "loading";
    }
  });
});
listen("switch_excel_err", event => {
  const excelRowCountErr: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === excelRowCountErr.split("|")[0]) {
      file.status = "error";
      file.errorMessage = excelRowCountErr.split("|")[1];
    }
  });
});
listen("e2c_msg", (event: any) => {
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
    const trimFile = await trimOpenFile(true, "Excel", ["*"], {
      includeStatus: true
    });
    data.path = trimFile.filePath;
    selectedFiles.value = trimFile.fileInfo;
    message("get excel sheets...", {
      type: "info",
      duration: 0,
      icon: Loading
    });
    const mapSheets: string[] = await invoke("map_excel_sheets", {
      path: data.path
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
  } catch (err) {
    closeAllMessage();
    message(err.toString(), { type: "error" });
  }
}

// invoke excel2csv
async function excelToCsv() {
  if (data.path === "") {
    message("Excel file not selected", { type: "warning" });
    return;
  }
  try {
    isLoading.value = true;
    const mapFileSheet = fileSheet.value.map(item => ({
      filename: item.filename,
      sheetname: item.sheetname
    }));
    const rtime: string = await invoke("excel2csv", {
      path: data.path,
      skipRows: data.skipRows,
      mapFileSheet: mapFileSheet,
      allSheets: data.allSheets,
      writeSheetname: data.writeSheetname
    });
    message(`Convert done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container" :style="dynamicHeight">
    <div class="custom-container1">
      <el-form-item>
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
      </el-form-item>
      <span v-if="backendCompleted"> {{ backendInfo }} </span>
      <span v-else> Batch convert files to csv </span>
    </div>

    <div class="custom-container1">
      <div class="custom-container2">
        <el-tooltip content="Convert all sheets or not" effect="light">
          <el-switch
            v-model="data.allSheets"
            inline-prompt
            style="
              --el-switch-on-color: #43cd80;
              --el-switch-off-color: #b0c4de;
              width: 90px;
            "
            active-text="All sheet"
            inactive-text="One sheet"
            :active-action-icon="Open"
            :inactive-action-icon="TurnOff"
          />
        </el-tooltip>
        <el-tooltip content="write sheetname or not" effect="light">
          <el-switch
            v-model="data.writeSheetname"
            inline-prompt
            style="
              --el-switch-on-color: #43cd80;
              --el-switch-off-color: #b0c4de;
              margin-right: 10px;
            "
            active-text="Y"
            inactive-text="N"
            :active-action-icon="Open"
            :inactive-action-icon="TurnOff"
          />
        </el-tooltip>
        <el-tooltip content="skip rows" effect="light">
          <el-input v-model="data.skipRows" style="width: 50px" />
        </el-tooltip>
      </div>
      <el-button
        @click="excelToCsv()"
        :loading="isLoading"
        :icon="SwitchFilled"
        style="width: 135px"
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
          <span v-if="scope.row.errorMessage && scope.row.status !== 'loading'">
            {{ scope.row.errorMessage || scope.row.status }}
          </span>
        </template>
      </el-table-column>
      <el-table-column
        prop="sheets"
        label="Sheets"
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
    </el-table>
  </el-form>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Event } from "@tauri-apps/api/event";
import { Files, FolderOpened, SwitchButton } from "@element-plus/icons-vue";
import { useDynamicHeight } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { mdFill, useMarkdown } from "@/utils/markdown";
import {
  useFlexible,
  useProgress,
  useQuoting,
  useSkiprows
} from "@/store/modules/options";

const [currentRows, totalRows] = [ref(0), ref(0)];
const [isLoading, dialog] = [ref(false), ref(false)];
const columns = ref<string[]>([]);
const path = ref("");
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];

// 每列对应的日期格式(key: 列名, value: format 字符串)
const inputFormats = ref<Record<string, string>>({});
const outputFormats = ref<Record<string, string>>({});

// 日期格式选项
// 输入格式（含 Auto detect）
const dateFormats = [
  { label: "Auto detect", value: "" },
  // 无分隔符
  { label: "YYYYMMDD", value: "%Y%m%d" },
  { label: "YYYYMMDDHHMMSS", value: "%Y%m%d%H%M%S" },
  { label: "YYYYMMDDHHMM", value: "%Y%m%d%H%M" },
  { label: "DDMMYYYY", value: "%d%m%Y" },
  { label: "DDMMYYYYHHMMSS", value: "%d%m%Y%H%M%S" },
  { label: "MMDDYYYY", value: "%m%d%Y" },
  { label: "MMDDYYYYHHMMSS", value: "%m%d%Y%H%M%S" },

  // YMD (年-月-日)
  { label: "YYYY-MM-DD", value: "%Y-%m-%d" },
  { label: "YYYY/MM/DD", value: "%Y/%m/%d" },
  { label: "YYYY-MM-DD HH:mm:ss", value: "%Y-%m-%d %H:%M:%S" },
  { label: "YYYY/MM/DD HH:mm:ss", value: "%Y/%m/%d %H:%M:%S" },

  // YDM (年-日-月)
  { label: "YYYY-DD-MM", value: "%Y-%d-%m" },
  { label: "YYYY/DD/MM", value: "%Y/%d/%m" },
  { label: "YYYY-DD-MM HH:mm:ss", value: "%Y-%d-%m %H:%M:%S" },
  { label: "YYYY/DD/MM HH:mm:ss", value: "%Y/%d/%m %H:%M:%S" },

  // MDY (月-日-年)
  { label: "MM-DD-YYYY", value: "%m-%d-%Y" },
  { label: "MM/DD/YYYY", value: "%m/%d/%Y" },
  { label: "MM-DD-YYYY HH:mm:ss", value: "%m-%d-%Y %H:%M:%S" },
  { label: "MM/DD/YYYY HH:mm:ss", value: "%m/%d/%Y %H:%M:%S" },

  // MYD (月-年-日)
  { label: "MM-YYYY-DD", value: "%m-%Y-%d" },
  { label: "MM/YYYY/DD", value: "%m/%Y/%d" },
  { label: "MM-YYYY-DD HH:mm:ss", value: "%m-%Y-%d %H:%M:%S" },
  { label: "MM/YYYY/DD HH:mm:ss", value: "%m/%Y/%d %H:%M:%S" },

  // DMY (日-月-年)
  { label: "DD-MM-YYYY", value: "%d-%m-%Y" },
  { label: "DD/MM/YYYY", value: "%d/%m/%Y" },
  { label: "DD-MM-YYYY HH:mm:ss", value: "%d-%m-%Y %H:%M:%S" },
  { label: "DD/MM/YYYY HH:mm:ss", value: "%d/%m/%Y %H:%M:%S" },

  // DYM (日-年-月)
  { label: "DD-YYYY-MM", value: "%d-%Y-%m" },
  { label: "DD/YYYY/MM", value: "%d/%Y/%m" },
  { label: "DD-YYYY-MM HH:mm:ss", value: "%d-%Y-%m %H:%M:%S" },
  { label: "DD/YYYY/MM HH:mm:ss", value: "%d/%Y/%m %H:%M:%S" },

  // 其他带时间格式
  { label: "YYYY-MM-DD HH:mm:ss.SSS", value: "%Y-%m-%d %H:%M:%S%.f" },
  { label: "YYYY-MM-DDTHH:mm:ss", value: "%Y-%m-%dT%H:%M:%S" },
  { label: "YYYY-MM-DDTHH:mm:ss.SSS", value: "%Y-%m-%dT%H:%M:%S%.f" },
  { label: "YYYY-MM-DD HH:mm", value: "%Y-%m-%d %H:%M" },
  { label: "YYYY/MM/DD HH:mm", value: "%Y/%m/%d %H:%M" },

  // 中文格式
  { label: "YYYY年MM月DD日", value: "%Y年%m月%d日" },
  { label: "YYYY年MM月DD日 HH时MM分SS秒", value: "%Y年%m月%d日 %H时%M分%S秒" },
  { label: "YYYY年MM月DD日 HH:mm:ss", value: "%Y年%m月%d日 %H:%M:%S" },
  { label: "DD日MM月YYYY年", value: "%d日%m月%Y年" },
  { label: "DD日MM月YYYY年 HH时MM分SS秒", value: "%d日%m月%Y年 %H时%M分%S秒" },
  { label: "DD日MM月YYYY年 HH:mm:ss", value: "%d日%m月%Y年 %H:%M:%S" },
  { label: "MM月DD日YYYY年", value: "%m月%d日%Y年" },
  { label: "MM月DD日YYYY年 HH时MM分SS秒", value: "%m月%d日%Y年 %H时%M分%S秒" },
  { label: "MM月DD日YYYY年 HH:mm:ss", value: "%m月%d日%Y年 %H:%M:%S" },

  // 时间在前
  { label: "HH:mm:ss YYYY-MM-DD", value: "%H:%M:%S %Y-%m-%d" },
  { label: "HH:mm YYYY-MM-DD", value: "%H:%M %Y-%m-%d" }
];

// 输出格式
const outputDateFormats = dateFormats.filter(fmt => fmt.value !== "");

const { dynamicHeight } = useDynamicHeight(106);
const { mdShow } = useMarkdown(mdFill);
const quoting = useQuoting();
const skiprows = useSkiprows();
const progress = useProgress();
const flexible = useFlexible();

listen("update-rows", (event: Event<number>) => {
  currentRows.value = event.payload;
});
listen("total-rows", (event: Event<number>) => {
  totalRows.value = event.payload;
});

watch(
  columns,
  newCols => {
    const newSet = new Set(newCols);

    for (const col of Object.keys(inputFormats.value)) {
      if (!newSet.has(col)) {
        delete inputFormats.value[col];
        delete outputFormats.value[col];
      }
    }

    for (const col of newCols) {
      if (!(col in inputFormats.value)) {
        inputFormats.value[col] = "";
      }
      if (!(col in outputFormats.value)) {
        outputFormats.value[col] = "%Y-%m-%d";
      }
    }
  },
  { immediate: true }
);

async function selectFile() {
  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (!path.value) {
    path.value = "";
    columns.value = [];
    return;
  }

  totalRows.value = 0;
  try {
    tableHeader.value = await mapHeaders(path.value, skiprows.skiprows);
    const { columnView, dataView } = await toJson(
      path.value,
      skiprows.skiprows
    );
    tableColumn.value = columnView;
    tableData.value = dataView;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

async function convertDates() {
  if (!path.value) {
    message("File not selected", { type: "warning" });
    return;
  }
  if (columns.value.length === 0) {
    message("No column selected", { type: "warning" });
    return;
  }

  const columnConfigs: Record<
    string,
    { inputFormat?: string; outputFormat: string }
  > = {};

  for (const col of columns.value) {
    const input = inputFormats.value[col]?.trim() || "";
    const output = outputFormats.value[col]?.trim() || "%Y-%m-%d";

    columnConfigs[col] = {
      inputFormat: input === "" ? undefined : input,
      outputFormat: output
    };
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("datefmt", {
      path: path.value,
      columnConfigs,
      flexible: flexible.flexible,
      quoting: quoting.quoting,
      skiprows: skiprows.skiprows,
      progress: progress.progress
    });
    message(`Date conversion completed, time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(`${err}`, { type: "error" });
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <el-form class="page-container">
    <el-splitter>
      <el-splitter-panel size="300" :resizable="false">
        <div class="splitter-container">
          <el-button @click="selectFile()" :icon="FolderOpened" text round>
            Open File
          </el-button>

          <el-select
            v-model="columns"
            multiple
            filterable
            placeholder="Select date columns"
            class="mt-2 ml-2"
            style="width: 280px"
          >
            <el-option
              v-for="item in tableHeader"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>

          <div v-for="col in columns" :key="col" class="mt-2 ml-2 mr-2">
            <el-tooltip
              :content="`Column: ${col}`"
              placement="right"
              effect="light"
            >
              <div class="flex gap-3 w-full">
                <div class="flex flex-col items-start">
                  <span class="text-[10px] text-blue-500 font-medium mb-1">
                    IN
                  </span>
                  <el-select
                    v-model="inputFormats[col]"
                    filterable
                    placeholder="Auto"
                    style="width: 135px"
                  >
                    <el-option
                      v-for="fmt in dateFormats"
                      :key="fmt.value"
                      :label="fmt.label"
                      :value="fmt.value"
                    />
                  </el-select>
                </div>

                <div class="flex flex-col items-start">
                  <span class="text-[10px] text-green-600 font-medium mb-1">
                    OUT
                  </span>
                  <el-select
                    v-model="outputFormats[col]"
                    filterable
                    style="width: 135px"
                  >
                    <el-option
                      v-for="fmt in outputDateFormats"
                      :key="fmt.value"
                      :label="fmt.label"
                      :value="fmt.value"
                    />
                  </el-select>
                </div>
              </div>
            </el-tooltip>
          </div>

          <div class="flex flex-col mt-auto">
            <el-progress
              v-if="totalRows > 0"
              :percentage="Math.round((currentRows / totalRows) * 100)"
              class="mb-2 ml-2"
            />
            <el-link @click="dialog = true">
              <span class="link-text">Date Format</span>
            </el-link>
          </div>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-button
          @click="convertDates()"
          :loading="isLoading"
          :icon="SwitchButton"
          text
          round
        >
          Convert Dates
        </el-button>

        <el-table
          :data="tableData"
          :height="dynamicHeight"
          show-overflow-tooltip
          tooltip-effect="light"
          class="mt-2"
        >
          <el-table-column
            v-for="column in tableColumn"
            :prop="column.prop"
            :label="column.label"
            :key="column.prop"
          />
        </el-table>

        <el-text class="mt-2">
          <el-icon><Files /></el-icon>
          {{ path }}
        </el-text>
      </el-splitter-panel>
    </el-splitter>

    <el-dialog v-model="dialog" title="Date Format Converter" width="70%">
      <el-scrollbar :height="dynamicHeight * 0.7">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

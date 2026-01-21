<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Event } from "@tauri-apps/api/event";
import { FolderOpened, Files, SwitchButton } from "@element-plus/icons-vue";
import { useDark } from "@pureadmin/utils";
import { useDynamicHeight } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { mdPinyin, useMarkdown } from "@/utils/markdown";

const [mode, pinyinStyle] = [ref("idx"), ref("upper")];
const modeOptions = [
  { label: "Nil", value: "nil" },
  { label: "Idx", value: "idx" }
];
const pyOptions = [
  { label: "Upper", value: "upper" },
  { label: "Lower", value: "lower" }
];
const [currentRows, totalRows] = [ref(0), ref(0)];
const [columns, path] = [ref(""), ref("")];
const [isLoading, dialog] = [ref(false), ref(false)];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const { dynamicHeight } = useDynamicHeight(98);
const { mdShow } = useMarkdown(mdPinyin);
const { isDark } = useDark();

listen("update-rows", (event: Event<number>) => {
  currentRows.value = event.payload;
});
listen("total-rows", (event: Event<number>) => {
  totalRows.value = event.payload;
});

async function selectFile() {
  columns.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];
  totalRows.value = 0;

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

  try {
    tableHeader.value = await mapHeaders(path.value, "0");
    const { columnView, dataView } = await toJson(path.value);
    tableColumn.value = columnView;
    tableData.value = dataView;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke pinyin
async function chineseToPinyin() {
  if (path.value === "") {
    message("File not selected", { type: "warning" });
    return;
  }
  if (columns.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const cols = Object.values(columns.value).join("|");
    const rtime: string = await invoke("pinyin", {
      path: path.value,
      columns: cols,
      mode: mode.value,
      pinyinStyle: pinyinStyle.value
    });
    message(`Convert done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container">
    <el-splitter>
      <el-splitter-panel size="180" :resizable="false">
        <div class="splitter-container">
          <el-button @click="selectFile()" :icon="FolderOpened" text round>
            Open File
          </el-button>

          <el-tooltip
            content="if Nil, no progress bar"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle w-40">
              <span
                v-for="item in modeOptions"
                :key="item.value"
                class="mode-item"
                :class="{
                  active: mode === item.value,
                  'active-dark': isDark && mode === item.value
                }"
                @click="mode = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <el-tooltip content="pinyin style" effect="light" placement="right">
            <div class="mode-toggle mt-2 w-40">
              <span
                v-for="item in pyOptions"
                :key="item.value"
                class="mode-item"
                :class="{
                  active: pinyinStyle === item.value,
                  'active-dark': isDark && pinyinStyle === item.value
                }"
                @click="pinyinStyle = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <el-select
            v-model="columns"
            multiple
            filterable
            placeholder="Select columns"
            class="mt-2 ml-2"
            style="width: 160px"
          >
            <el-option
              v-for="item in tableHeader"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>

          <div class="flex flex-col mt-auto">
            <el-progress
              v-if="totalRows !== 0 && isFinite(currentRows / totalRows)"
              :percentage="Math.round((currentRows / totalRows) * 100)"
              class="mb-2 ml-2"
            />
            <el-link @click="dialog = true">
              <span class="link-text">Pinyin</span>
            </el-link>
          </div>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-button
          @click="chineseToPinyin()"
          :loading="isLoading"
          :icon="SwitchButton"
          text
          round
          >Run
        </el-button>

        <el-table
          :data="tableData"
          :height="dynamicHeight"
          show-overflow-tooltip
          tooltip-effect="light"
        >
          <el-table-column
            v-for="column in tableColumn"
            :prop="column.prop"
            :label="column.label"
            :key="column.prop"
          />
        </el-table>

        <el-text>
          <el-icon class="ml-2">
            <Files />
          </el-icon>
          {{ path }}
        </el-text>
      </el-splitter-panel>
    </el-splitter>

    <el-dialog
      v-model="dialog"
      title="Pinyin - Convert Chinese to Pinyin in CSV"
      width="70%"
    >
      <el-scrollbar :height="dynamicHeight * 0.7">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

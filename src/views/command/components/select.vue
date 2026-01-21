<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Event } from "@tauri-apps/api/event";
import { CheckboxValueType } from "element-plus";
import { FolderOpened, Files, SwitchButton } from "@element-plus/icons-vue";
import { useDark } from "@pureadmin/utils";
import { message } from "@/utils/message";
import { viewOpenFile, mapHeaders, toJson } from "@/utils/view";
import { useDynamicHeight } from "@/utils/utils";
import { mdSelect, useMarkdown } from "@/utils/markdown";

const path = ref("");
const [currentRows, totalRows] = [ref(0), ref(0)];
const [selMode, pgsMode] = [ref("include"), ref("idx")];
const selModeOptions = [
  { label: "Include", value: "include" },
  { label: "Exclude", value: "exclude" }
];
const pgsModeOptions = [
  { label: "Nil", value: "nil" },
  { label: "Idx", value: "idx" }
];
const [originalColumns, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const [isLoading, dialog, checkAll, indeterminate] = [
  ref(false),
  ref(false),
  ref(false),
  ref(false)
];
const { dynamicHeight } = useDynamicHeight(98);
const { mdShow } = useMarkdown(mdSelect);
const { isDark } = useDark();
const selColumns = ref<CheckboxValueType[]>([]);

watch(selColumns, val => {
  if (val.length === 0) {
    checkAll.value = false;
    indeterminate.value = false;
  } else if (val.length === originalColumns.value.length) {
    checkAll.value = true;
    indeterminate.value = false;
  } else {
    indeterminate.value = true;
  }
});

const handleCheckAll = (val: CheckboxValueType) => {
  indeterminate.value = false;
  if (val) {
    selColumns.value = originalColumns.value.map(_ => _.value);
  } else {
    selColumns.value = [];
  }
};

listen("update-rows", (event: Event<number>) => {
  currentRows.value = event.payload;
});
listen("total-rows", (event: Event<number>) => {
  totalRows.value = event.payload;
});

async function selectFile() {
  originalColumns.value = [];
  path.value = "";
  selColumns.value = [];
  tableColumn.value = [];
  tableData.value = [];
  totalRows.value = 0;

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

  try {
    originalColumns.value = await mapHeaders(path.value, "0");
    const { columnView, dataView } = await toJson(path.value);
    tableColumn.value = columnView;
    tableData.value = dataView;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}

// invoke select
async function selectColumns() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (selColumns.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const selCols = Object.values(selColumns.value).join("|");
    const rtime: string = await invoke("select", {
      path: path.value,
      selCols: selCols,
      selMode: selMode.value,
      pgsMode: pgsMode.value
    });
    message(`Select done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container">
    <el-splitter>
      <el-splitter-panel size="220" :resizable="false">
        <div class="splitter-container">
          <el-button @click="selectFile()" :icon="FolderOpened" text round>
            Open File
          </el-button>

          <el-tooltip content="Select mode" effect="light" placement="right">
            <div class="mode-toggle w-[200px]">
              <span
                v-for="item in selModeOptions"
                :key="item.value"
                class="mode-item"
                :class="{
                  active: selMode === item.value,
                  'active-dark': isDark && selMode === item.value
                }"
                @click="selMode = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <el-tooltip
            content="if Nil, no progress bar"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle mt-2 w-[200px]">
              <span
                v-for="item in pgsModeOptions"
                :key="item.value"
                class="mode-item"
                :class="{
                  active: pgsMode === item.value,
                  'active-dark': isDark && pgsMode === item.value
                }"
                @click="pgsMode = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <el-select
            v-model="selColumns"
            multiple
            filterable
            placeholder="Select columns"
            class="mt-2 ml-2"
            style="width: 200px"
          >
            <template #header>
              <el-checkbox
                v-model="checkAll"
                :indeterminate="indeterminate"
                @change="handleCheckAll"
              >
                All
              </el-checkbox>
            </template>
            <el-option
              v-for="item in originalColumns"
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
              <span class="link-text">Select</span>
            </el-link>
          </div>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-button
          @click="selectColumns()"
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
      title="Select - Select, drop, re-order columns"
      width="70%"
    >
      <el-scrollbar :height="dynamicHeight * 0.7">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

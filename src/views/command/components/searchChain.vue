<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Event } from "@tauri-apps/api/event";
import {
  Files,
  FolderOpened,
  SwitchButton,
  CloseBold
} from "@element-plus/icons-vue";
import { message } from "@/utils/message";
import { useDynamicHeight } from "@/utils/utils";
import { toJson, viewOpenFile, mapHeaders } from "@/utils/view";
import { mdSearch, useMarkdown } from "@/utils/markdown";
import {
  useFlexible,
  useProgress,
  useQuoting,
  useSkiprows
} from "@/store/modules/options";

interface ColumnConfig {
  column: string;
  mode: string;
  condition: string;
}

const columnConfigs = ref<ColumnConfig[]>([]);
const logics = ref<string[]>([]); // 长度 = columnConfigs.length - 1

const path = ref("");
const [currentRows, totalRows, matchRows] = [ref(0), ref(0), ref(0)];
const [dialog, isLoading] = [ref(false), ref(false)];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];

const { dynamicHeight } = useDynamicHeight(98);
const { mdShow } = useMarkdown(mdSearch);
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

watch(columnConfigs, newConfigs => {
  const n = newConfigs.length;
  logics.value = Array(n > 0 ? n - 1 : 0).fill("and");
});

async function selectFile() {
  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) {
    path.value = "";
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

// 添加/移除列配置
function addColumn() {
  columnConfigs.value.push({
    column: "",
    mode: "equal",
    condition: ""
  });
}

function removeColumn(index: number) {
  columnConfigs.value.splice(index, 1);
}

async function searchData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (columnConfigs.value.length === 0) {
    message("Add at least one column filter", { type: "warning" });
    return;
  }

  // 校验:所有列必须选中
  for (const cfg of columnConfigs.value) {
    if (!cfg.column) {
      message("All columns must be selected", { type: "warning" });
      return;
    }
  }

  try {
    isLoading.value = true;

    const res: string[] = await invoke("search_chain", {
      path: path.value,
      configs: columnConfigs.value, // [{column, mode, condition}, ...]
      logics: logics.value,
      progress: progress.progress,
      quoting: quoting.quoting,
      flexible: flexible.flexible,
      skiprows: skiprows.skiprows
    });

    matchRows.value = Number(res[0]);
    message(`Match ${res[0]} rows, elapsed time: ${res[1]} s`, {
      type: "success"
    });
  } catch (err) {
    message(err.toString(), { type: "error" });
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <el-form class="page-container">
    <el-splitter>
      <el-splitter-panel size="320" :resizable="false">
        <div class="splitter-container">
          <el-button @click="selectFile()" :icon="FolderOpened" text round>
            Open File
          </el-button>

          <el-button @click="addColumn()" class="mt-2 ml-2 mr-2" size="small">
            + Add Filter
          </el-button>

          <div
            v-for="(cfg, index) in columnConfigs"
            :key="index"
            class="mt-2 ml-2 mr-2 p-2 border rounded"
          >
            <div class="flex justify-between items-center mb-2">
              <el-text v-if="cfg.column" type="primary" class="font-medium">
                {{ cfg.column }}
              </el-text>
              <el-text v-else class="text-gray-400">No column selected</el-text>

              <el-button
                @click="removeColumn(index)"
                size="small"
                circle
                text
                type="danger"
              >
                <el-icon><CloseBold /></el-icon>
              </el-button>
            </div>

            <div class="flex gap-2 mb-2">
              <el-select
                v-model="cfg.column"
                filterable
                placeholder="Select column"
                style="width: 150px"
              >
                <el-option
                  v-for="item in tableHeader"
                  :key="item.value"
                  :label="item.label"
                  :value="item.value"
                />
              </el-select>

              <el-select
                v-model="cfg.mode"
                filterable
                placeholder="Mode"
                style="width: 150px"
              >
                <el-option label="equal" value="equal" />
                <el-option label="not_equal" value="not_equal" />
                <el-option label="contains" value="contains" />
                <el-option label="not_contains" value="not_contains" />
                <el-option label="starts_with" value="starts_with" />
                <el-option label="not_starts_with" value="not_starts_with" />
                <el-option label="ends_with" value="ends_with" />
                <el-option label="not_ends_with" value="not_ends_with" />
                <el-option label="regex" value="regex" />
                <el-option label="is_null" value="is_null" />
                <el-option label="is_not_null" value="is_not_null" />
                <el-option label="gt(>)" value="gt" />
                <el-option label="ge(≥)" value="ge" />
                <el-option label="lt(<)" value="lt" />
                <el-option label="le(≤)" value="le" />
                <el-option label="between" value="between" />
              </el-select>
            </div>

            <el-input
              v-model="cfg.condition"
              placeholder="Condition (use | for multiple values, e.g. tom|jerry)"
              class="mb-2"
              type="textarea"
              :autosize="{ minRows: 2, maxRows: 2 }"
              resize="none"
            />

            <el-select
              v-if="index < columnConfigs.length - 1"
              v-model="logics[index]"
              placeholder="logic"
            >
              <el-option label="AND" value="and" />
              <el-option label="OR" value="or" />
            </el-select>
          </div>

          <div class="flex flex-col mt-auto">
            <el-progress
              v-if="totalRows !== 0 && isFinite(currentRows / totalRows)"
              :percentage="Math.round((currentRows / totalRows) * 100)"
              class="mb-2 ml-2"
            />
            <el-link @click="dialog = true">
              <span class="link-text">Search Chain</span>
            </el-link>
          </div>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <div class="flex justify-between items-center">
          <el-button
            @click="searchData()"
            :loading="isLoading"
            :icon="SwitchButton"
            text
            round
          >
            Run
          </el-button>
          <el-text v-if="matchRows" style="margin-right: 8px">
            match rows: {{ matchRows }}
          </el-text>
        </div>

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
          <el-icon class="ml-2"><Files /></el-icon>
          {{ path }}
        </el-text>
      </el-splitter-panel>
    </el-splitter>

    <el-dialog
      v-model="dialog"
      title="Search - Filter rows matching conditions"
      width="70%"
    >
      <el-scrollbar :height="dynamicHeight * 0.7">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Event } from "@tauri-apps/api/event";
import { Files, FolderOpened, Link, ArrowRight } from "@element-plus/icons-vue";
import { useDark } from "@pureadmin/utils";
import { message } from "@/utils/message";
import { useDynamicHeight } from "@/utils/utils";
import { toJson, viewOpenFile, mapHeaders } from "@/utils/view";
import { mdSearch, useMarkdown } from "@/utils/markdown";

const [mode, progress] = [ref("equal"), ref("nil")];
const pgsOptions = [
  { label: "Nil", value: "nil" },
  { label: "Idx", value: "idx" }
];
const [currentRows, totalRows, matchRows] = [ref(0), ref(0), ref(0)];
const [column, path, condition] = [ref(""), ref(""), ref("")];
const [dialog, isLoading, isBtnShow] = [ref(false), ref(false), ref(false)];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const { dynamicHeight } = useDynamicHeight(146);
const { mdShow } = useMarkdown(mdSearch);
const { isDark } = useDark();

listen("update-rows", (event: Event<number>) => {
  currentRows.value = event.payload;
});
listen("total-rows", (event: Event<number>) => {
  totalRows.value = event.payload;
});

async function selectFile() {
  isBtnShow.value = false;
  path.value = "";
  column.value = "";
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

// invoke search
async function searchData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (column.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const res: string[] = await invoke("search", {
      path: path.value,
      column: column.value,
      mode: mode.value,
      condition: condition.value,
      progress: progress.value
    });
    matchRows.value = Number(res[0]);
    isBtnShow.value = true;
    message(`Match ${res[0]} rows, elapsed time: ${res[1]} s`, {
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
      <el-splitter-panel size="260" :resizable="false">
        <div class="splitter-container">
          <el-tooltip content="Add data" effect="light" placement="right">
            <el-button @click="selectFile()" :icon="FolderOpened" circle text />
          </el-tooltip>

          <el-tooltip
            content="if nil, no progress bar"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle">
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

          <el-select
            v-model="column"
            filterable
            style="margin-top: 8px; margin-left: 8px; width: 240px"
            placeholder="Select column"
          >
            <el-option
              v-for="item in tableHeader"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>

          <el-tooltip content="Search mode" effect="light" placement="right">
            <el-select
              v-model="mode"
              filterable
              style="margin-top: 8px; margin-left: 8px; width: 240px"
            >
              <el-option label="Equal" value="equal" />
              <el-option label="EqualMulti" value="equal_multi" />
              <el-option label="NotEqual" value="not_equal" />
              <el-option label="Contains" value="contains" />
              <el-option label="ContainsMulti" value="contains_multi" />
              <el-option label="NotContains" value="not_contains" />
              <el-option label="StartsWith" value="starts_with" />
              <el-option label="StartsWithMulti" value="starts_with_multi" />
              <el-option label="NotStartsWtih" value="not_starts_with" />
              <el-option label="EndsWith" value="ends_with" />
              <el-option label="EndsWithMulti" value="ends_with_multi" />
              <el-option label="NotEndsWith" value="not_ends_with" />
              <el-option label="Regex" value="regex" />
              <el-option label="IsNull" value="is_null" />
              <el-option label="IsNotNull" value="is_not_null" />
              <el-option label="gt(>)" value="gt" />
              <el-option label="ge(≥)" value="ge" />
              <el-option label="lt(<)" value="lt" />
              <el-option label="le(≤)" value="le" />
              <el-option label="Between" value="between" />
            </el-select>
          </el-tooltip>

          <el-input
            v-model="condition"
            :autosize="{ minRows: 8, maxRows: 8 }"
            type="textarea"
            placeholder="Search conditions......Separate by |. (Example: tom|jack|jerry)"
            style="margin-top: 8px; margin-left: 8px; width: 240px"
          />

          <div style="margin-top: auto; display: flex; flex-direction: column">
            <el-progress
              v-if="totalRows !== 0 && isFinite(currentRows / totalRows)"
              :percentage="Math.round((currentRows / totalRows) * 100)"
              style="margin-bottom: 8px; margin-left: 8px"
            />

            <el-link @click="dialog = true" :icon="Link">
              <span>
                About
                <span style="color: skyblue; font-weight: bold">Search</span>
              </span>
            </el-link>
          </div>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <div class="header-row">
          <el-tooltip content="Run" effect="light" placement="right">
            <el-button
              @click="searchData()"
              :loading="isLoading"
              :icon="ArrowRight"
              circle
              text
            />
          </el-tooltip>

          <el-text v-if="matchRows" style="margin-right: 8px">
            match rows: {{ matchRows }}
          </el-text>
        </div>

        <el-table
          :data="tableData"
          :height="dynamicHeight"
          show-overflow-tooltip
        >
          <el-table-column
            v-for="column in tableColumn"
            :prop="column.prop"
            :label="column.label"
            :key="column.prop"
          />
        </el-table>

        <el-text>
          <el-icon style="margin-left: 8px">
            <Files />
          </el-icon>
          {{ path }}
        </el-text>
      </el-splitter-panel>
    </el-splitter>

    <el-dialog
      v-model="dialog"
      title="Search - Filter rows matching conditions"
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

<style scoped>
.mode-toggle {
  width: 240px;
}
.header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>

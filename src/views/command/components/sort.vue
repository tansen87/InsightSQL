<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Files, Link, ArrowRight } from "@element-plus/icons-vue";
import { useDark } from "@pureadmin/utils";
import { useDynamicHeight } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { mdSort, useMarkdown } from "@/utils/markdown";

const mode = ref("Sort");
const modeOptions = [
  { label: "Sort", value: "Sort" },
  { label: "ExtSort", value: "ExtSort" },
  { label: "Index", value: "Index" }
];
const numOptions = [
  { label: "True", value: true },
  { label: "False", value: false }
];
const reverseOptions = [
  { label: "True", value: true },
  { label: "False", value: false }
];
const [column, path] = [ref(""), ref("")];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const [isLoading, dialog, numeric, reverse] = [
  ref(false),
  ref(false),
  ref(false),
  ref(false)
];
const { dynamicHeight } = useDynamicHeight(146);
const { mdShow } = useMarkdown(mdSort);
const { isDark } = useDark();

async function selectFile() {
  column.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

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

// invoke sort
async function sortData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (column.value.length === 0 && mode.value !== "index") {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    let rtime: string;
    if (mode.value == "Sort") {
      rtime = await invoke("sort", {
        path: path.value,
        column: column.value,
        numeric: numeric.value,
        reverse: reverse.value
      });
    } else if (mode.value == "ExtSort") {
      rtime = await invoke("extsort", {
        path: path.value,
        column: column.value,
        reverse: reverse.value
      });
    } else if (mode.value == "Index") {
      rtime = await invoke("idx", {
        path: path.value
      });
    }
    message(`${mode.value} done, elapsed time: ${rtime} s`, {
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
      <el-splitter-panel size="200" :resizable="false">
        <div class="splitter-container">
          <el-tooltip content="Add data" effect="light" placement="right">
            <el-button @click="selectFile()" :icon="FolderOpened" circle text />
          </el-tooltip>

          <el-tooltip content="Sort mode" effect="light" placement="right">
            <div class="mode-toggle">
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

          <el-tooltip content="Numeric" effect="light" placement="right">
            <div class="mode-toggle" style="margin-top: 8px">
              <span
                v-for="item in numOptions"
                :key="String(item.value)"
                class="mode-item"
                :class="{
                  active: numeric === item.value,
                  'active-dark': isDark && numeric === item.value
                }"
                @click="numeric = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <el-tooltip
            content="Reverse (when set to False, sort from small to large)"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle" style="margin-top: 8px">
              <span
                v-for="item in reverseOptions"
                :key="String(item.value)"
                class="mode-item"
                :class="{
                  active: reverse === item.value,
                  'active-dark': isDark && reverse === item.value
                }"
                @click="reverse = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <el-select
            v-model="column"
            filterable
            style="width: 180px; margin-left: 8px; margin-top: 8px"
            placeholder="Select column"
          >
            <el-option
              v-for="item in tableHeader"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>

          <el-link @click="dialog = true" :icon="Link" style="margin-top: auto">
            <el-tooltip :content="path" effect="light">
              <span>
                About
                <span style="color: skyblue; font-weight: bold">Sort</span>
              </span>
            </el-tooltip>
          </el-link>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-tooltip content="Run" effect="light" placement="right">
          <el-button
            @click="sortData()"
            :loading="isLoading"
            :icon="ArrowRight"
            circle
            text
          />
        </el-tooltip>

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
      title="Sort - Sorts CSV data lexicographically"
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
  width: 180px;
}
</style>

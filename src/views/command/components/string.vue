<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  FolderOpened,
  Files,
  Link,
  TurnOff,
  Open,
  ArrowRight
} from "@element-plus/icons-vue";
import { useDark } from "@pureadmin/utils";
import { useDynamicHeight } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { mdStr, useMarkdown } from "@/utils/markdown";
import { message } from "@/utils/message";

const [column, path] = [ref(""), ref("")];
const [n, length, by, activeTab] = [ref("4"), ref("5"), ref("-"), ref("left")];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const [isLoading, dialog, reverse] = [ref(false), ref(false), ref(false)];
const reverseOptions = [
  { label: "True", value: true },
  { label: "False", value: false }
];
const toTab = computed(() => activeTab.value);
const { dynamicHeight } = useDynamicHeight(200);
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

// invoke str_slice, str_split, str_pad
async function StrData() {
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
    let rtime: string;
    if (["left", "right", "slice"].includes(toTab.value)) {
      rtime = await invoke("str_slice", {
        path: path.value,
        column: column.value,
        n: n.value,
        length: length.value,
        reverse: reverse.value,
        mode: toTab.value
      });
    }
    if (["split_n", "split_max"].includes(toTab.value)) {
      rtime = await invoke("str_split", {
        path: path.value,
        column: column.value,
        n: n.value,
        by: by.value,
        mode: toTab.value
      });
    }
    if (["pad_left", "pad_right", "pad_both"].includes(toTab.value)) {
      rtime = await invoke("str_pad", {
        path: path.value,
        column: column.value,
        length: length.value,
        fillChar: by.value,
        mode: toTab.value
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

const { mdShow } = useMarkdown(mdStr);
</script>

<template>
  <el-form class="page-container">
    <el-tabs v-model="activeTab">
      <el-tab-pane name="left" label="Left" />
      <el-tab-pane name="right" label="Right" />
      <el-tab-pane name="slice" label="Slice" />
      <el-tab-pane name="split_n" label="SplitN" />
      <el-tab-pane name="split_max" label="SplitMax" />
      <el-tab-pane name="pad_left" label="PadLeft" />
      <el-tab-pane name="pad_right" label="PadRight" />
      <el-tab-pane name="pad_both" label="PadBoth" />
    </el-tabs>

    <el-splitter>
      <el-splitter-panel size="180" :resizable="false">
        <div class="splitter-container">
          <el-tooltip content="Add data" effect="light" placement="right">
            <el-button @click="selectFile()" :icon="FolderOpened" circle text />
          </el-tooltip>

          <el-select
            v-model="column"
            filterable
            style="width: 160px; margin-left: 8px"
            placeholder="Select column"
          >
            <el-option
              v-for="item in tableHeader"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>

          <el-tooltip content="Reverse or not" effect="light" placement="right">
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

          <el-tooltip
            v-if="['left', 'right'].includes(toTab)"
            content="Length of the slice"
            effect="light"
            placement="right"
          >
            <el-input
              v-model="n"
              style="margin-left: 8px; margin-top: 8px; width: 160px"
            />
          </el-tooltip>

          <template v-if="toTab === 'slice'">
            <el-tooltip content="Start index" effect="light" placement="right">
              <el-input
                v-model="n"
                style="margin-left: 8px; margin-top: 8px; width: 160px"
              />
            </el-tooltip>
            <el-tooltip
              content="Length of the slice"
              effect="light"
              placement="right"
            >
              <el-input
                v-model="length"
                style="margin-left: 8px; margin-top: 8px; width: 160px"
              />
            </el-tooltip>
          </template>

          <template v-if="['split_n', 'split_max'].includes(toTab)">
            <el-tooltip
              content="nth/max number of items to return"
              effect="light"
              placement="right"
            >
              <el-input
                v-model="n"
                style="margin-left: 8px; margin-top: 8px; width: 160px"
              />
            </el-tooltip>
            <el-tooltip
              content="Substring to split by"
              effect="light"
              placement="right"
            >
              <el-input
                v-model="by"
                style="margin-left: 8px; margin-top: 8px; width: 160px"
              />
            </el-tooltip>
          </template>

          <template
            v-if="['pad_left', 'pad_right', 'pad_both'].includes(toTab)"
          >
            <el-tooltip
              content="Pad the string until it reaches this length"
              effect="light"
              placement="right"
            >
              <el-input
                v-model="length"
                style="margin-left: 8px; margin-top: 8px; width: 160px"
              />
            </el-tooltip>
            <el-tooltip
              content="The character to pad the string with"
              effect="light"
              placement="right"
            >
              <el-input
                v-model="by"
                style="margin-left: 8px; margin-top: 8px; width: 160px"
              />
            </el-tooltip>
          </template>

          <el-link @click="dialog = true" :icon="Link" style="margin-top: auto">
            <span>
              About
              <span style="color: skyblue; font-weight: bold">String</span>
            </span>
          </el-link>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-tooltip content="Run" effect="light" placement="right">
          <el-button
            @click="StrData()"
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
      title="String - String expr: slice, split, pad..."
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
  width: 160px;
}
</style>

<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  FolderOpened,
  Refresh,
  Link,
  TurnOff,
  Open
} from "@element-plus/icons-vue";
import { useDynamicHeight, shortFileName } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { mdStr, useMarkdown } from "@/utils/markdown";
import { message } from "@/utils/message";

const [column, path] = [ref(""), ref("")];
const [n, length, by, activeTab] = [ref("4"), ref("5"), ref("-"), ref("left")];
const [tableHeader, tableColumn, tableData] = [ref([]), ref([]), ref([])];
const [isLoading, dialog, reverse] = [ref(false), ref(false), ref(false)];
const toTab = computed(() => activeTab.value);
const { dynamicHeight } = useDynamicHeight(207);

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
  <div class="page-container">
    <el-tabs v-model="activeTab">
      <el-tab-pane name="left" label="Left" />
      <el-tab-pane name="right" label="Right" />
      <el-tab-pane name="split_n" label="SplitN" />
      <el-tab-pane name="split_max" label="SplitMax" />
      <el-tab-pane name="pad_left" label="PadLeft" />
      <el-tab-pane name="pad_right" label="PadRight" />
      <el-tab-pane name="pad_both" label="PadBoth" />
    </el-tabs>
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
        <el-select
          v-model="column"
          filterable
          style="width: 150px; margin-left: 8px"
          placeholder="Select column"
        >
          <el-option
            v-for="item in tableHeader"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
        <el-tooltip
          v-if="['left', 'right'].includes(toTab)"
          content="Length of the slice"
          effect="light"
        >
          <el-input v-model="n" style="margin-left: 8px; width: 50px" />
        </el-tooltip>
        <template v-if="toTab === 'slice'">
          <el-tooltip content="Start index" effect="light">
            <el-input v-model="n" style="margin-left: 8px; width: 50px" />
          </el-tooltip>
          <el-tooltip content="Length of the slice" effect="light">
            <el-input v-model="length" style="margin-left: 8px; width: 50px" />
          </el-tooltip>
        </template>
        <template v-if="['split_n', 'split_max'].includes(toTab)">
          <el-tooltip
            content="nth/max number of items to return"
            effect="light"
          >
            <el-input v-model="n" style="margin-left: 8px; width: 50px" />
          </el-tooltip>
          <el-tooltip content="Substring to split by" effect="light">
            <el-input v-model="by" style="margin-left: 8px; width: 50px" />
          </el-tooltip>
        </template>
        <template v-if="['pad_left', 'pad_right', 'pad_both'].includes(toTab)">
          <el-tooltip
            content="Pad the string until it reaches this length"
            effect="light"
          >
            <el-input v-model="length" style="margin-left: 8px; width: 50px" />
          </el-tooltip>
          <el-tooltip
            content="The character to pad the string with"
            effect="light"
          >
            <el-input v-model="by" style="margin-left: 8px; width: 50px" />
          </el-tooltip>
        </template>
        <el-tooltip content="Reverse or not" effect="light">
          <el-switch
            v-model="reverse"
            v-if="['left', 'right', 'slice'].includes(toTab)"
            inline-prompt
            style="
              --el-switch-on-color: #43cd80;
              --el-switch-off-color: #b0c4de;
              margin-left: 8px;
            "
            active-text="Y"
            inactive-text="N"
            :active-action-icon="Open"
            :inactive-action-icon="TurnOff"
          />
        </el-tooltip>
      </div>
      <el-button @click="StrData()" :loading="isLoading" :icon="Refresh">
        Str
      </el-button>
    </div>
    <el-table
      :data="tableData"
      :height="dynamicHeight"
      border
      empty-text=""
      style="margin-top: 10px; width: 100%"
      show-overflow-tooltip
    >
      <el-table-column
        v-for="column in tableColumn"
        :prop="column.prop"
        :label="column.label"
        :key="column.prop"
      />
    </el-table>
    <div class="custom-container1">
      <div class="custom-container2">
        <el-tooltip :content="path" effect="light">
          <el-text>{{ shortFileName(path) }}</el-text>
        </el-tooltip>
      </div>
      <el-link @click="dialog = true" :icon="Link">
        <span>
          About
          <span style="color: skyblue; font-weight: bold">String</span>
        </span>
      </el-link>
    </div>
    <el-dialog
      v-model="dialog"
      title="String - String expr: slice, split, pad..."
      width="800"
    >
      <el-scrollbar :height="dynamicHeight * 0.8">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </div>
</template>

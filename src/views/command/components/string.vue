<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Files, ArrowRight } from "@element-plus/icons-vue";
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
const modeOptions = [
  { label: "Left", value: "left" },
  { label: "Right", value: "right" },
  { label: "Slice", value: "slice" },
  { label: "SplitN", value: "split_n" },
  { label: "SplitMax", value: "split_max" },
  { label: "PadLeft", value: "pad_left" },
  { label: "PadRight", value: "pad_right" },
  { label: "PadBoth", value: "pad_both" }
];
const { dynamicHeight } = useDynamicHeight(98);
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
    if (["left", "right", "slice"].includes(activeTab.value)) {
      rtime = await invoke("str_slice", {
        path: path.value,
        column: column.value,
        n: n.value,
        length: length.value,
        reverse: reverse.value,
        mode: activeTab.value
      });
    }
    if (["split_n", "split_max"].includes(activeTab.value)) {
      rtime = await invoke("str_split", {
        path: path.value,
        column: column.value,
        n: n.value,
        by: by.value,
        mode: activeTab.value
      });
    }
    if (["pad_left", "pad_right", "pad_both"].includes(activeTab.value)) {
      rtime = await invoke("str_pad", {
        path: path.value,
        column: column.value,
        length: length.value,
        fillChar: by.value,
        mode: activeTab.value
      });
    }
    message(`${activeTab.value} done, elapsed time: ${rtime} s`, {
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
    <el-splitter>
      <el-splitter-panel size="180" :resizable="false">
        <div class="splitter-container">
          <el-tooltip content="Add data" effect="light" placement="right">
            <el-button @click="selectFile()" :icon="FolderOpened" circle text />
          </el-tooltip>

          <!-- mode choice -->
          <div class="mode-toggle-v mb-2 w-40 h-[128px]">
            <span
              v-for="item in modeOptions"
              :key="item.value"
              class="mode-item"
              :class="{
                active: activeTab === item.value,
                'active-dark': isDark && activeTab === item.value
              }"
              @click="activeTab = item.value"
            >
              {{ item.label }}
            </span>
          </div>

          <el-select
            v-model="column"
            filterable
            placeholder="Select column"
            class="ml-2"
            style="width: 160px"
          >
            <el-option
              v-for="item in tableHeader"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>

          <el-tooltip content="Reverse or not" effect="light" placement="right">
            <div class="mode-toggle mt-2 w-40">
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
            v-if="['left', 'right'].includes(activeTab)"
            content="Length of the slice"
            effect="light"
            placement="right"
          >
            <el-input v-model="n" class="mt-2 ml-2" style="width: 160px" />
          </el-tooltip>

          <template v-if="activeTab === 'slice'">
            <el-tooltip content="Start index" effect="light" placement="right">
              <el-input v-model="n" class="mt-2 ml-2" style="width: 160px" />
            </el-tooltip>
            <el-tooltip
              content="Length of the slice"
              effect="light"
              placement="right"
            >
              <el-input
                v-model="length"
                class="mt-2 ml-2"
                style="width: 160px"
              />
            </el-tooltip>
          </template>

          <template v-if="['split_n', 'split_max'].includes(activeTab)">
            <el-tooltip
              content="nth/max number of items to return"
              effect="light"
              placement="right"
            >
              <el-input v-model="n" class="mt-2 ml-2" style="width: 160px" />
            </el-tooltip>
            <el-tooltip
              content="Substring to split by"
              effect="light"
              placement="right"
            >
              <el-input v-model="by" class="mt-2 ml-2" style="width: 160px" />
            </el-tooltip>
          </template>

          <template
            v-if="['pad_left', 'pad_right', 'pad_both'].includes(activeTab)"
          >
            <el-tooltip
              content="Pad the string until it reaches this length"
              effect="light"
              placement="right"
            >
              <el-input
                v-model="length"
                class="mt-2 ml-2"
                style="width: 160px"
              />
            </el-tooltip>
            <el-tooltip
              content="The character to pad the string with"
              effect="light"
              placement="right"
            >
              <el-input v-model="by" class="mt-2 ml-2" style="width: 160px" />
            </el-tooltip>
          </template>

          <el-link @click="dialog = true" class="mt-auto">
            <span class="link-text">String</span>
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
      title="String - String expr: slice, split, pad..."
      width="70%"
    >
      <el-scrollbar :height="dynamicHeight * 0.7">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

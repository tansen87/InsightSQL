<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  ArrowRight,
  FolderOpened,
  Link,
  CirclePlus,
  Remove,
  Files
} from "@element-plus/icons-vue";
import { useDark } from "@pureadmin/utils";
import { useDynamicHeight } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { CheckboxValueType } from "element-plus";
import { mdApply, useMarkdown } from "@/utils/markdown";

const [
  isLoading,
  checkAll,
  indeterminate,
  newColumn,
  dialog,
  backendCompleted
] = [ref(false), ref(false), ref(false), ref(false), ref(false), ref(false)];
const [operations, tableHeader, tableColumn, tableData] = [
  ref([]),
  ref([]),
  ref([]),
  ref([])
];
const [path, comparand, replacement, formatstr, backendInfo] = [
  ref(""),
  ref(""),
  ref(""),
  ref(""),
  ref("")
];
const mode = ref("operations");
const modeOptions = [
  { label: "Operations", value: "operations" },
  { label: "CalcConv", value: "calcconv" },
  { label: "DynFmt", value: "cat" }
];
const columnContent = ref("no column");
const columns = ref<CheckboxValueType[]>([]);
const { dynamicHeight } = useDynamicHeight(146);
const { isDark } = useDark();
watch(columns, val => {
  if (val.length === 0) {
    checkAll.value = false;
    indeterminate.value = false;
  } else if (val.length === tableHeader.value.length) {
    checkAll.value = true;
    indeterminate.value = false;
  } else {
    indeterminate.value = true;
  }
});
const handleCheckAll = (val: CheckboxValueType) => {
  indeterminate.value = false;
  if (val) {
    columns.value = tableHeader.value.map(_ => _.value);
  } else {
    columns.value = [];
  }
};
const { mdShow } = useMarkdown(mdApply);

async function selectFile() {
  columns.value = [];
  operations.value = [];
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

// invoke apply
async function applyData() {
  if (path.value === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (columns.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const result: string = await invoke("apply", {
      path: path.value,
      columns: Object.values(columns.value).join("|"),
      mode: mode.value,
      operations: operations.value.join("|"),
      comparand: comparand.value,
      replacement: replacement.value,
      formatstr: formatstr.value,
      newColumn: newColumn.value
    });
    message(`Apply done, elapsed time: ${result} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}

function addNewColumn() {
  newColumn.value = !newColumn.value;
  if (newColumn.value === true) {
    columnContent.value = "add column";
  } else {
    columnContent.value = "no column";
  }
}
</script>

<template>
  <el-form class="page-container">
    <el-splitter>
      <el-splitter-panel size="260" :resizable="false">
        <div class="splitter-container">
          <div class="button-row">
            <el-tooltip content="Add data" effect="light">
              <el-button
                @click="selectFile()"
                :icon="FolderOpened"
                text
                circle
              />
            </el-tooltip>
            <el-tooltip :content="columnContent" effect="light">
              <el-button @click="addNewColumn" text circle>
                <el-icon>
                  <CirclePlus v-if="newColumn" />
                  <Remove v-else />
                </el-icon>
              </el-button>
            </el-tooltip>
          </div>

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

          <el-select
            v-model="columns"
            filterable
            multiple
            placeholder="Select column(s)"
            style="margin-top: 8px; margin-left: 8px; width: 240px"
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
              v-for="item in tableHeader"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>

          <el-select
            v-if="mode === 'operations'"
            v-model="operations"
            filterable
            multiple
            placeholder="Operations"
            style="margin-top: 8px; margin-left: 8px; width: 240px"
          >
            <el-option label="Copy" value="copy" />
            <el-option label="Len" value="len" />
            <el-option label="Lower" value="lower" />
            <el-option label="Upper" value="upper" />
            <el-option label="Trim" value="trim" />
            <el-option label="Ltrim" value="ltrim" />
            <el-option label="Rtrim" value="rtrim" />
            <el-option label="Replace" value="replace" />
            <el-option label="Round" value="round" />
            <el-option label="Squeeze" value="squeeze" />
            <el-option label="Strip" value="strip" />
            <el-option label="Reverse" value="reverse" />
            <el-option label="Abs" value="abs" />
            <el-option label="Neg" value="neg" />
          </el-select>

          <div style="margin-top: 8px; margin-left: 8px; width: 240px">
            <template
              v-if="
                ['operations'].includes(mode) && operations.includes('replace')
              "
            >
              <el-tooltip content="old" effect="light" placement="right">
                <el-input v-model="comparand" placeholder="replace - from" />
              </el-tooltip>
              <el-tooltip content="new" effect="light" placement="right">
                <el-input
                  v-model="replacement"
                  placeholder="replace - to"
                  style="margin-top: 5px"
                />
              </el-tooltip>
            </template>

            <template v-if="['cat', 'calcconv'].includes(mode)">
              <el-tooltip content="Expression" effect="light" placement="right">
                <el-input v-model="formatstr" placeholder="{col1} + {col2}" />
              </el-tooltip>
            </template>
          </div>

          <el-link @click="dialog = true" :icon="Link" style="margin-top: auto">
            <span v-if="backendCompleted"> {{ backendInfo }} </span>
            <span>
              About
              <span style="color: skyblue; font-weight: bold">Apply</span>
            </span>
          </el-link>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-tooltip content="Run" effect="light">
          <el-button
            @click="applyData()"
            :loading="isLoading"
            :icon="ArrowRight"
            text
            circle
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
      title="Apply - Apply a series of transformation functions to given CSV column/s"
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
.button-row {
  display: flex;
  align-items: center;
}
</style>

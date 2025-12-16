<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Files, Link, ArrowRight } from "@element-plus/icons-vue";
import { useDark } from "@pureadmin/utils";
import { useDynamicHeight } from "@/utils/utils";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { mdJoin, useMarkdown } from "@/utils/markdown";

const joinType = ref("left");
const [sel1, sel2] = [ref(""), ref("")];
const [dialog, isLoading, nulls] = [ref(false), ref(false), ref(false)];
const nullOptions = [
  { label: "True", value: true },
  { label: "False", value: false }
];
const [
  tableHeader1,
  tableHeader2,
  tableColumn1,
  tableColumn2,
  tableData1,
  tableData2
] = [ref([]), ref([]), ref([]), ref([]), ref([]), ref([])];
const data = reactive({ path1: "", path2: "" });
const { dynamicHeight } = useDynamicHeight(84);
const { mdShow } = useMarkdown(mdJoin);
const { isDark } = useDark();

async function selectFile(fileIndex: number) {
  const selectColumn = fileIndex === 1 ? sel1 : sel2;
  const tableHeader = fileIndex === 1 ? tableHeader1 : tableHeader2;
  const tableColumn = fileIndex === 1 ? tableColumn1 : tableColumn2;
  const tableData = fileIndex === 1 ? tableData1 : tableData2;
  const path = fileIndex === 1 ? "path1" : "path2";

  data[path] = "";
  selectColumn.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

  data[path] = await viewOpenFile(false, "csv", ["*"]);
  if (data[path] === null) return;

  try {
    tableHeader.value = await mapHeaders(data[path], "0");
    const { columnView, dataView } = await toJson(data[path]);
    tableColumn.value = columnView;
    tableData.value = dataView;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
}

// invoke join
async function joinData() {
  if (data.path1 === "" || data.path2 === "") {
    message("File not selected", { type: "warning" });
    return;
  }
  if (sel1.value.length === 0 || sel2.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("join", {
      path1: data.path1,
      path2: data.path2,
      sel1: sel1.value,
      sel2: sel2.value,
      joinType: joinType.value,
      nulls: nulls.value
    });
    message(`Join done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container" :style="{ height: dynamicHeight + 'px' }">
    <el-splitter>
      <el-splitter-panel size="180" :resizable="false">
        <div class="splitter-container">
          <div class="button-row">
            <el-tooltip content="Add data 1" effect="light">
              <el-button
                @click="selectFile(1)"
                :icon="FolderOpened"
                circle
                text
              />
            </el-tooltip>
            <el-tooltip content="Add data 2" effect="light">
              <el-button
                @click="selectFile(2)"
                :icon="FolderOpened"
                circle
                text
              />
            </el-tooltip>
          </div>

          <el-tooltip
            content="column of file1"
            effect="light"
            placement="right"
          >
            <el-select
              v-model="sel1"
              filterable
              style="width: 160px; margin-left: 8px"
              placeholder="column of file1"
            >
              <el-option
                v-for="item in tableHeader1"
                :key="item.value"
                :label="item.label"
                :value="item.value"
              />
            </el-select>
          </el-tooltip>

          <el-tooltip
            content="column of file2"
            effect="light"
            placement="right"
          >
            <el-select
              v-model="sel2"
              filterable
              style="width: 160px; margin-left: 8px; margin-top: 8px"
              placeholder="column of file2"
            >
              <el-option
                v-for="item in tableHeader2"
                :key="item.value"
                :label="item.label"
                :value="item.value"
              />
            </el-select>
          </el-tooltip>

          <el-tooltip
            content="When set True, joins will work on empty fields"
            effect="light"
            placement="right"
          >
            <div class="mode-toggle" style="margin-top: 8px">
              <span
                v-for="item in nullOptions"
                :key="item.value"
                class="mode-item"
                :class="{
                  active: nulls === item.value,
                  'active-dark': isDark && nulls === item.value
                }"
                @click="nulls = item.value"
              >
                {{ item.label }}
              </span>
            </div>
          </el-tooltip>

          <el-tooltip content="Join type" effect="light" placement="right">
            <el-select
              v-model="joinType"
              style="width: 160px; margin-left: 8px; margin-top: 8px"
            >
              <el-option label="left" value="left" />
              <el-option label="right" value="right" />
              <el-option label="full" value="full" />
              <el-option label="cross" value="cross" />
              <el-option label="inner" value="inner" />
              <el-option label="left-semi" value="left_semi" />
              <el-option label="left-anti" value="left_anti" />
              <el-option label="right-semi" value="right_semi" />
              <el-option label="right-anti" value="right_anti" />
            </el-select>
          </el-tooltip>

          <el-link @click="dialog = true" :icon="Link" style="margin-top: auto">
            <span>
              About
              <span style="color: skyblue; font-weight: bold">Join</span>
            </span>
          </el-link>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-splitter layout="vertical">
          <el-splitter-panel size="33" :resizable="false">
            <el-tooltip content="Run" effect="light" placement="right">
              <el-button
                @click="joinData()"
                :loading="isLoading"
                :icon="ArrowRight"
                circle
                text
              />
            </el-tooltip>
          </el-splitter-panel>

          <el-splitter-panel :resizable="false">
            <el-table
              :data="tableData1"
              :height="dynamicHeight / 2 - 49"
              empty-text="data 1"
              show-overflow-tooltip
            >
              <el-table-column
                v-for="column in tableColumn1"
                :prop="column.prop"
                :label="column.label"
                :key="column.prop"
              />
            </el-table>

            <el-text>
              <el-icon style="margin-left: 8px">
                <Files />
              </el-icon>
              {{ data.path1 }}
            </el-text>
          </el-splitter-panel>

          <el-splitter-panel :resizable="false">
            <el-table
              :data="tableData2"
              :height="dynamicHeight / 2 - 49"
              empty-text="data 2"
              show-overflow-tooltip
            >
              <el-table-column
                v-for="column in tableColumn2"
                :prop="column.prop"
                :label="column.label"
                :key="column.prop"
              />
            </el-table>

            <el-text>
              <el-icon style="margin-left: 8px">
                <Files />
              </el-icon>
              {{ data.path2 }}
            </el-text>
          </el-splitter-panel>
        </el-splitter>
      </el-splitter-panel>
    </el-splitter>

    <el-dialog
      v-model="dialog"
      title="Join - Joins two sets of CSV data on the specified columns"
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
.button-row {
  display: flex;
  align-items: center;
}
</style>

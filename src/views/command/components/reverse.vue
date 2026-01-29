<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpened, Files, SwitchButton } from "@element-plus/icons-vue";
import { useDynamicHeight } from "@/utils/utils";
import { viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { mdReverse, useMarkdown } from "@/utils/markdown";
import { useQuoting, useSkiprows } from "@/store/modules/options";

const path = ref("");
const [tableColumn, tableData] = [ref([]), ref([])];
const [isLoading, dialog] = [ref(false), ref(false)];
const { dynamicHeight } = useDynamicHeight(98);
const { mdShow } = useMarkdown(mdReverse);
const quotingStore = useQuoting();
const skiprowsStore = useSkiprows();

async function selectFile() {
  tableColumn.value = [];
  tableData.value = [];

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;

  try {
    const { columnView, dataView } = await toJson(
      path.value,
      skiprowsStore.skiprows
    );
    tableColumn.value = columnView;
    tableData.value = dataView;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
}

// invoke reverse
async function reverseData() {
  if (path.value === "") {
    message("File not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;
    const rtime: string = await invoke("reverse", {
      path: path.value,
      quoting: quotingStore.quoting,
      skiprows: skiprowsStore.skiprows
    });
    message(`Reverse done, elapsed time: ${rtime} s`, { type: "success" });
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

          <el-link @click="dialog = true" class="mt-auto">
            <span class="link-text">Reverse</span>
          </el-link>
        </div>
      </el-splitter-panel>

      <el-splitter-panel>
        <el-button
          @click="reverseData()"
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
          <el-icon class="ml-2"><Files /></el-icon>
          {{ path }}
        </el-text>
      </el-splitter-panel>
    </el-splitter>

    <el-dialog
      v-model="dialog"
      title="Reverse - Reverse order of rows in a CSV"
      width="70%"
    >
      <el-scrollbar :height="dynamicHeight * 0.7">
        <div v-html="mdShow" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>

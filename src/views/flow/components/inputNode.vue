<script setup lang="ts">
import { ref } from "vue";
import { Handle, Position } from "@vue-flow/core";
import { FolderOpened, CloseBold } from "@element-plus/icons-vue";
import { mapHeaders, viewOpenFile, toJson } from "@/utils/view";
import { message } from "@/utils/message";
import { shortFileName } from "@/utils/utils";
import { usePath, useHeaders } from "@/store/modules/flow";
import { useWorkflowStore } from "@/store/modules/workflow";
import { useSkiprows } from "@/store/modules/options";

const path = ref("");
const isPath = ref(false);
const [tableColumn, tableData] = [ref([]), ref([])];
const headerStore = useHeaders();
const pathStore = usePath();
const skiprowsStore = useSkiprows();

const props = defineProps<{ id: string }>();

function deleteBtn() {
  const store = useWorkflowStore();
  store.removeNodes([props.id]);
}

async function selectFile() {
  path.value = "";
  isPath.value = false;

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;
  pathStore.path = path.value;
  isPath.value = true;
  try {
    const headers: any = await mapHeaders(path.value, skiprowsStore.skiprows);
    headerStore.headers = headers;
    const { columnView, dataView } = await toJson(
      path.value,
      skiprowsStore.skiprows
    );
    tableColumn.value = columnView;
    tableData.value = dataView;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}
</script>

<template>
  <div class="page-container node-container w-[400px]">
    <div class="text-center p-[5px]">
      <el-button
        circle
        link
        @click="deleteBtn"
        :icon="CloseBold"
        size="small"
        class="absolute top-[-2.5px] right-[-2.5px] z-10"
      />
      <span class="block font-bold"> Start </span>
      <el-button @click="selectFile()" :icon="FolderOpened">
        <span v-if="isPath">
          <el-tooltip :content="path" effect="light">
            <span>{{ shortFileName(path) }}</span>
          </el-tooltip>
        </span>
        <span v-else>Open File</span>
      </el-button>
    </div>
    <el-table
      :data="tableData"
      show-overflow-tooltip
      tooltip-effect="light"
      height="200px"
    >
      <el-table-column
        v-for="column in tableColumn"
        :prop="column.prop"
        :label="column.label"
        :key="column.prop"
      />
    </el-table>
    <Handle
      type="source"
      :position="Position.Right"
      id="output"
      class="handle-style"
    />
  </div>
</template>

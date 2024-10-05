<script setup lang="ts">
import { ref, reactive } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { IceCreamRound, FolderOpened } from "@element-plus/icons-vue";

const isLoading = ref(false);
const isPath = ref(false);
const writeRows = ref(0);
const runtime = ref(0.0);
const columns = ref("");
const originalColumns = ref([]);
const data = reactive({
  filePath: "",
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"],
  sep: ",",
  mode: "equal",
  condition: "银行存款|应收账款"
});

listen("runtime", (event: any) => {
  runtime.value = event.payload;
});
listen("equal_err", (event: any) => {
  const equalErr = event.payload;
  ElNotification({
    title: "Equal Error",
    message: equalErr,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});
listen("equal_count", (event: any) => {
  const count: any = event.payload;
  writeRows.value = count;
});
listen("contains_err", (event: any) => {
  const containsErr = event.payload;
  ElNotification({
    title: "Contains Error",
    message: containsErr,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});
listen("contains_count", (event: any) => {
  const count: any = event.payload;
  writeRows.value = count;
});
listen("startswith_err", (event: any) => {
  const startswithErr = event.payload;
  ElNotification({
    title: "Startwith Error",
    message: startswithErr,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});

async function selectFile() {
  isLoading.value = false;
  isPath.value = false;
  const selected = await open({
    multiple: false,
    filters: [
      {
        name: "csv",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.filePath = selected.toString();
  } else if (selected === null) {
    return;
  } else {
    data.filePath = selected;
  }
  isPath.value = true;

  const header: any = await invoke("get_search_headers", {
    path: data.filePath,
    sep: data.sep
  });
  originalColumns.value = header;
}

// search data
async function searchData() {
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }
  if (columns.value.length === 0) {
    ElNotification({
      title: "Column not defined",
      message: "未选择columns",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  const outputPath = await save({
    title: "Export",
    defaultPath: `search_${new Date().getTime()}.csv`,
    filters: [{ name: "CSV", extensions: ["csv"] }]
  });
  if (outputPath === "" || outputPath === null) {
    ElNotification({
      title: "File not found",
      message: "未选择保存文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  if (data.filePath !== "") {
    isLoading.value = true;

    await invoke("search", {
      path: data.filePath,
      sep: data.sep,
      column: columns.value,
      mode: data.mode,
      condition: data.condition,
      outputPath: outputPath
    });

    isLoading.value = false;
    ElNotification({
      message:
        "Search done, search rows: " +
        writeRows.value +
        " lines, elapsed time: " +
        runtime.value,
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
  }
}
</script>

<template>
  <div class="page-container">
    <div
      style="
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        position: sticky;
      "
    >
      <div style="display: flex; align-items: flex-start">
        <el-button
          type="primary"
          @click="selectFile()"
          :icon="FolderOpened"
          plain
        >
          Open File
        </el-button>
        <el-select v-model="data.sep" style="margin-left: 16px; width: 100px">
          <el-option label="," value="," />
          <el-option label="|" value="|" />
          <el-option label="\t" value="\t" />
          <el-option label=";" value=";" />
        </el-select>
      </div>

      <el-text type="primary" size="large">
        <el-icon> <IceCreamRound /> </el-icon>
        <span v-if="isPath">{{ data.filePath }}</span>
        <span v-else>Select fields matching rows</span>
      </el-text>
    </div>
    <p />
    <div style="margin-top: 10px">
      <el-select v-model="data.mode" style="width: 112px">
        <el-option label="equal" value="equal" />
        <el-option label="contains" value="contains" />
        <el-option label="startswith" value="startswith" />
      </el-select>
      <el-select
        v-model="columns"
        filterable
        style="margin-left: 16px; width: 200px"
        placeholder="please choose column"
      >
        <el-option
          v-for="item in originalColumns"
          :key="item.value"
          :label="item.label"
          :value="item.value"
        />
      </el-select>
      <el-button
        type="success"
        @click="searchData()"
        :loading="isLoading"
        :icon="IceCreamRound"
        plain
        style="margin-left: 16px"
      >
        Search
      </el-button>
    </div>
    <div style="margin-top: 20px">
      <el-text> conditions </el-text>
      <el-input
        v-model="data.condition"
        autosize
        type="textarea"
        placeholder="Please input conditions"
      />
    </div>
  </div>
</template>

<style>
.page-container {
  margin-bottom: 20px;
  padding: 20px;
  border-radius: 10px;
  background-color: #fff;
}
</style>

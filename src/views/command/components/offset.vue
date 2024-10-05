<script setup lang="ts">
import { ref, reactive } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { IceCreamRound, FolderOpened } from "@element-plus/icons-vue";

const isLoading = ref(false);
const isPath = ref(false);
const runtime = ref(0.0);
const columns = ref("");
const conditonColumns = ref("");
const originalColumns = ref([]);
const data = reactive({
  filePath: "",
  fileFormats: ["*"],
  sep: ",",
  hasCond: false
});

listen("runtime", (event: any) => {
  runtime.value = event.payload;
});
listen("offset_err", (event: any) => {
  const equalErr = event.payload;
  ElNotification({
    title: "Offset Error",
    message: equalErr,
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

  const header: any = await invoke("get_offset_headers", {
    filePath: data.filePath,
    sep: data.sep
  });
  originalColumns.value = header;
}

// net amount
async function netAmount() {
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择文件",
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
    defaultPath: `offset_${new Date().getTime()}`,
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

    const cols = Object.values(conditonColumns.value).join("|");

    await invoke("offset", {
      filePath: data.filePath,
      sep: data.sep,
      amount: columns.value,
      cond: cols,
      hasCond: data.hasCond,
      outputPath: outputPath
    });

    isLoading.value = false;
    ElNotification({
      message: "Offset done, elapsed time: " + runtime.value,
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
        <el-select
          v-model="data.hasCond"
          style="margin-left: 16px; width: 120px"
        >
          <el-option label="no_cond" :value="false" />
          <el-option label="has_cond" :value="true" />
        </el-select>
      </div>

      <el-text type="primary" size="large">
        <el-icon> <IceCreamRound /> </el-icon>
        <span v-if="isPath">{{ data.filePath }}</span>
        <span v-else>Net amount</span>
      </el-text>
    </div>
    <p />
    <div style="margin-top: 10px">
      <el-select
        v-model="columns"
        filterable
        style="width: 25%"
        placeholder="Amount column"
      >
        <el-option
          v-for="item in originalColumns"
          :key="item.value"
          :label="item.label"
          :value="item.value"
        />
      </el-select>
      <el-select
        v-model="conditonColumns"
        multiple
        filterable
        style="margin-left: 16px; width: 50%"
        placeholder="Condition column"
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
        @click="netAmount()"
        :loading="isLoading"
        :icon="IceCreamRound"
        plain
        style="margin-left: 16px"
      >
        Offset
      </el-button>
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

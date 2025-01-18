<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { IceCreamRound, FolderOpened } from "@element-plus/icons-vue";

const isLoading = ref(false);
const isPath = ref(false);
const columns = ref("");
const conditonColumns = ref("");
const originalColumns = ref([]);
const data = reactive({
  filePath: "",
  fileFormats: ["*"],
  hasCond: false
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

  try {
    const header: any = await invoke("get_offset_headers", {
      filePath: data.filePath
    });

    if (JSON.stringify(header).startsWith("get header error:")) {
      throw JSON.stringify(header).toString();
    }

    originalColumns.value = header;
  } catch (err) {
    ElNotification({
      title: "Open File error",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
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

  isLoading.value = true;

  try {
    const cols = Object.values(conditonColumns.value).join("|");

    const result: string = await invoke("offset", {
      filePath: data.filePath,
      amount: columns.value,
      cond: cols,
      hasCond: data.hasCond
    });

    if (JSON.stringify(result).startsWith("offset failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Offset done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
  } catch (err) {
    ElNotification({
      title: "Invoke offset error",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
  isLoading.value = false;
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
        <el-select
          v-model="data.hasCond"
          style="margin-left: 16px; width: 120px"
        >
          <el-option label="no_cond" :value="false" />
          <el-option label="has_cond" :value="true" />
        </el-select>
      </div>

      <el-text type="primary" size="large">
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

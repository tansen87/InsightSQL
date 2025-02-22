<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { IceCreamRound, FolderOpened } from "@element-plus/icons-vue";
import { message } from "@/utils/message";
import { viewOpenFile } from "@/utils/view";

const isLoading = ref(false);
const isPath = ref(false);
const columns = ref("");
const conditonColumns = ref("");
const originalColumns = ref([]);
const data = reactive({
  filePath: "",
  hasCond: false
});

async function selectFile() {
  isPath.value = false;

  data.filePath = await viewOpenFile(false, "csv", ["*"]);
  if (data.filePath === null) {
    return;
  }

  try {
    const header: any = await invoke("get_offset_headers", {
      filePath: data.filePath
    });

    originalColumns.value = header;
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
}

// net amount
async function netAmount() {
  if (data.filePath === "") {
    message("File not selected", { type: "warning" });
    return;
  }
  if (columns.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  try {
    isLoading.value = true;

    const cols = Object.values(conditonColumns.value).join("|");
    const result: string = await invoke("offset", {
      filePath: data.filePath,
      amount: columns.value,
      cond: cols,
      hasCond: data.hasCond
    });

    message(`Offset done, elapsed time: ${result} s`, { duration: 5000 });
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
  isLoading.value = false;
}
</script>

<template>
  <div class="page-container">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
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

      <el-text>
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
        @click="netAmount()"
        :loading="isLoading"
        :icon="IceCreamRound"
        style="margin-left: 16px"
      >
        Offset
      </el-button>
    </div>
  </div>
</template>

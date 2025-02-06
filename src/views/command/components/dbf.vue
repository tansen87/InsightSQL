<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  FolderOpened,
  SwitchFilled,
  Loading,
  Select,
  CloseBold
} from "@element-plus/icons-vue";
import { useDynamicFormHeight, filterFileStatus } from "@/utils/utils";
import { message } from "@/utils/message";

const [isLoading, selectedFiles] = [ref(false), ref([])];
const data = reactive({
  filePath: "",
  sep: "|"
});
const { formHeight } = useDynamicFormHeight(134);

listen("start_convert", (event: any) => {
  const startConvert: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert) {
      file.status = "loading";
    }
  });
});
listen("dbf2csv_msg", (event: any) => {
  const dbf2csvMsg: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === dbf2csvMsg) {
      file.status = "completed";
    }
  });
});

async function selectFile() {
  selectedFiles.value = [];
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "dbf",
        extensions: ["*"]
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.filePath = selected.join("|").toString();
    const nonEmptyRows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = nonEmptyRows.map((file: any) => {
      return { filename: file, status: "" };
    });
  } else if (selected === null) {
    return;
  } else {
    data.filePath = selected;
  }
}

// invoke dbf
async function convertData() {
  if (data.filePath === "") {
    message("Fle not selected", { type: "warning" });
    return;
  }

  isLoading.value = true;

  try {
    const result: string = await invoke("dbf", {
      filePath: data.filePath,
      sep: data.sep
    });

    message(`Convert done, elapsed time: ${result} s`, { duration: 5000 });
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
  isLoading.value = false;
}
</script>

<template>
  <el-form class="page-container" :style="formHeight">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened">
          Open File
        </el-button>
        <el-tooltip content="Write the delimiter for CSV" effect="light">
          <el-select v-model="data.sep" style="margin-left: 10px; width: 100px">
            <el-option label="," value="," />
            <el-option label="|" value="|" />
            <el-option label="\t" value="\t" />
            <el-option label=";" value=";" />
          </el-select>
        </el-tooltip>

        <el-button
          @click="convertData()"
          :loading="isLoading"
          :icon="SwitchFilled"
          style="margin-left: 10px"
        >
          Convert
        </el-button>
      </div>
      <el-text> Convert dbf file to CSV </el-text>
    </div>

    <el-table :data="selectedFiles" :height="formHeight" style="width: 100%">
      <el-table-column prop="filename" label="file" style="width: 80%" />
      <el-table-column
        prop="status"
        label="status"
        :filters="[
          { text: 'x', value: 'error' },
          { text: '√', value: 'completed' }
        ]"
        :filter-method="filterFileStatus"
        width="100"
      >
        <template #default="scope">
          <ElIcon v-if="scope.row.status === 'loading'" class="is-loading">
            <Loading />
          </ElIcon>
          <ElIcon v-else-if="scope.row.status === 'completed'" color="#00CD66">
            <Select />
          </ElIcon>
          <ElIcon v-else-if="scope.row.status === 'error'" color="#FF0000">
            <CloseBold />
          </ElIcon>
        </template>
      </el-table-column>
    </el-table>
  </el-form>
</template>

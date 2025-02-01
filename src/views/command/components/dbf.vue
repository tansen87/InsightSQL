<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import {
  FolderOpened,
  SwitchFilled,
  Loading,
  Select,
  CloseBold
} from "@element-plus/icons-vue";
import {
  useDynamicFormHeight,
  customColors,
  filterFileStatus
} from "@/utils/utils";

const [isLoading, selectedFiles, progress] = [ref(false), ref([]), ref(0)];
const data = reactive({
  filePath: "",
  fileFormats: ["*"],
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
listen("dbf2csv_progress", (event: any) => {
  const pgs: any = event.payload;
  progress.value = pgs;
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
  progress.value = 0;

  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "dbf",
        extensions: data.fileFormats
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
    ElNotification({
      title: "File not found",
      message: "未选择文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  isLoading.value = true;

  try {
    const result: string = await invoke("dbf", {
      filePath: data.filePath,
      sep: data.sep
    });

    if (JSON.stringify(result).startsWith("dbf failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Convert done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 5000
    });
  } catch (err) {
    ElNotification({
      title: "Dbf failed",
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
  <el-form class="page-container" :style="formHeight">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened" plain>
          Open File
        </el-button>
        <el-tooltip
          content="Write the delimiter for CSV"
          placement="top"
          effect="light"
        >
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
          plain
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

    <el-progress
      v-if="isLoading"
      :percentage="progress"
      :color="customColors"
    />
  </el-form>
</template>

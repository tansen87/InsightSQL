<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification, ElIcon } from "element-plus";
import { Loading, FolderOpened, Grape } from "@element-plus/icons-vue";
import { shortFileName, useDynamicFormHeight } from "@/utils/utils";

const isLoading = ref(false);
const progress = ref(0);
const selectedFiles = ref([]);
const tableRef = ref(null);
const customColors = [
  { color: "#98FB98", percentage: 20 },
  { color: "#7CFC00", percentage: 40 },
  { color: "#7FFF00", percentage: 60 },
  { color: "#ADFF2F", percentage: 80 },
  { color: "#9ACD32", percentage: 100 }
];
const data = reactive({
  filePath: "",
  fileFormats: ["*"]
});
const { formHeight } = useDynamicFormHeight(134);

listen("start_convert", (event: any) => {
  const startConvert: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (shortFileName(file.filename) === shortFileName(startConvert)) {
      file.status = "";
    }
  });
});
listen("count_msg", (event: any) => {
  const countMsg: any = event.payload;
  const basename = shortFileName(countMsg.split("|")[0]);
  selectedFiles.value.forEach(file => {
    if (shortFileName(file.filename) === basename) {
      file.status = countMsg.split("|")[1];
    }
  });
});
listen("count_progress", (event: any) => {
  const pgs: any = event.payload;
  progress.value = pgs;
});

// open file
async function selectFile() {
  isLoading.value = false;
  progress.value = 0;

  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "csv",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.filePath = selected.join("|").toString();
    const nonEmptyRows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = nonEmptyRows.map((file: any) => {
      return { filename: shortFileName(file), status: " " };
    });
  } else if (selected === null) {
    return;
  } else {
    data.filePath = selected;
  }
}

// count csv rows
async function countData() {
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  isLoading.value = true;

  try {
    const result: string = await invoke("count", {
      path: data.filePath
    });

    if (JSON.stringify(result).startsWith("count failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Count done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 5000
    });
  } catch (err) {
    ElNotification({
      title: "Invoke Count Error",
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
    <el-form>
      <div
        style="
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          position: sticky;
        "
      >
        <div style="display: flex; align-items: flex-start">
          <el-button @click="selectFile()" :icon="FolderOpened" plain>
            Open File
          </el-button>
          <el-button
            @click="countData()"
            :loading="isLoading"
            :icon="Grape"
            plain
            style="margin-left: 16px"
          >
            Count
          </el-button>
        </div>
        <el-text> Count the rows of CSV files </el-text>
      </div>

      <el-table
        ref="tableRef"
        :data="selectedFiles"
        :height="formHeight"
        style="width: 100%"
      >
        <el-table-column type="index" width="50" />
        <el-table-column prop="filename" label="file" style="width: 60%" />
        <el-table-column label="rows (include header)" width="200">
          <template #default="scope">
            <ElIcon v-if="scope.row.status === ''" class="is-loading">
              <Loading />
            </ElIcon>
            <span>{{ scope.row.status }}</span>
          </template>
        </el-table-column>
      </el-table>
    </el-form>
    <el-progress
      v-if="isLoading"
      :percentage="progress"
      :color="customColors"
    />
  </el-form>
</template>

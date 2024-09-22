<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from "vue";
import { open } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { ElNotification, ElIcon } from "element-plus";
import { Loading, FolderOpened, Grape } from "@element-plus/icons-vue";

const isLoading = ref(false);
const progress = ref(0);
const selectedFiles = ref([]);
const runtime = ref(0.0);
const tableRef = ref(null);
const windowHeight = ref(window.innerHeight);
const customColors = [
  { color: "#98FB98", percentage: 20 },
  { color: "#7CFC00", percentage: 40 },
  { color: "#7FFF00", percentage: 60 },
  { color: "#ADFF2F", percentage: 80 },
  { color: "#9ACD32", percentage: 100 }
];
const data = reactive({
  filePath: "",
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"],
  sep: ","
});

const formHeight = computed(() => {
  const height = 220;
  return windowHeight.value - height;
});

const updateWindowHeight = () => {
  windowHeight.value = window.innerHeight;
};

onMounted(() => {
  window.addEventListener("resize", updateWindowHeight);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", updateWindowHeight);
});

listen("start_convert", (event: any) => {
  const startConvert: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert.split("|")[0]) {
      file.status = "loading";
    }
  });
});
listen("runtime", (event: any) => {
  runtime.value = event.payload;
});
listen("count_err", (event: any) => {
  const countErr = event.payload;
  ElNotification({
    title: "Count Error",
    message: countErr,
    position: "bottom-right",
    type: "error",
    duration: 0
  });
  isLoading.value = false;
});
listen("count_msg", (event: any) => {
  const countMsg: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename.split("\\").pop() === countMsg.split("|")[0]) {
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
      return { filename: file, status: "" };
    });
  } else if (selected === null) {
    return;
  } else {
    data.filePath = selected;
  }
}

// count csv rows
async function countData() {
  if (data.filePath == "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  isLoading.value = true;

  await invoke("count", {
    path: data.filePath,
    sep: data.sep
  });

  ElNotification({
    message: "Count done, elapsed time: " + runtime.value,
    position: "bottom-right",
    type: "success",
    duration: 0
  });
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
          <el-button
            type="success"
            @click="countData()"
            :icon="Grape"
            plain
            style="margin-left: 16px"
          >
            Count
          </el-button>
        </div>
        <el-text type="primary" size="large">
          <el-icon> <Grape /> </el-icon>
          Count the rows of CSV files
        </el-text>
      </div>

      <el-table
        ref="tableRef"
        :data="selectedFiles"
        :height="formHeight"
        style="width: 100%"
      >
        <el-table-column prop="filename" label="file" style="width: 80%" />
        <el-table-column label="rows" width="100">
          <template #default="scope">
            <ElIcon v-if="scope.row.status === 'loading'" class="is-loading">
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

<style lang="scss">
.page-container {
  margin-bottom: 20px;
  padding: 20px;
  border-radius: 10px;
  background-color: #fff;
}
</style>

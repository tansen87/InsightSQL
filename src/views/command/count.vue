<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { ElMessage, ElIcon } from "element-plus";
import { Loading, FolderOpened, Grape } from "@element-plus/icons-vue";

const isProcessing = ref(false);
const progress = ref(0);
const selectedFiles = ref([]);
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

listen("start_convert", (event: any) => {
  const startConvert: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert.split("|")[0]) {
      file.status = "loading";
    }
  });
});
listen("count_err", (event: any) => {
  const error: any = event.payload;
  const countErrMsg: any = "count error: " + error;
  ElMessage.error(countErrMsg);
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

// count csv rows
async function countData() {
  if (data.filePath == "") {
    ElMessage.warning("未选择csv文件");
    return;
  }
  isProcessing.value = true;
  ElMessage.info("Running...");
  await invoke("count", {
    path: data.filePath,
    sep: data.sep
  });

  ElMessage.success("count done.");
}

async function selectFile() {
  isProcessing.value = false;
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
</script>

<template>
  <div class="page-container">
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

      <el-table :data="selectedFiles" height="760" style="width: 100%">
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
      v-if="isProcessing"
      :percentage="progress"
      :color="customColors"
    />
  </div>
</template>

<style lang="scss">
.page-container {
  margin-bottom: 20px;
  padding: 20px;
  border-radius: 10px;
  background-color: #fff;
}
</style>

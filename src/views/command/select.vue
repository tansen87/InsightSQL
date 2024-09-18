<script setup lang="ts">
import { ref, reactive, computed } from "vue";
import { VueDraggable } from "vue-draggable-plus";
import { open } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { ElMessage } from "element-plus";
import {
  SuccessFilled,
  Loading,
  Cherry,
  FolderOpened
} from "@element-plus/icons-vue";

const data = reactive({
  filePath: "",
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"],
  sep: ","
});
const originalList = ref([]);
const selectList = ref([]);
const isLoading = ref(false);
const isFinish = ref(false);
const isPath = ref(false);

listen("select_err", (event: any) => {
  const error: any = "select_err: " + event.payload;
  ElMessage.error(error);
});
listen("wtr_err", (event: any) => {
  const wtrMsg = event.payload;
  ElMessage.error("wtr_err: " + wtrMsg);
  isLoading.value = false;
});

// open file
async function selectFile() {
  isLoading.value = false;
  isFinish.value = false;
  isPath.value = false;
  originalList.value = [];
  selectList.value = [];
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

  const headers: any = await invoke("get_select_headers", {
    path: data.filePath,
    sep: data.sep
  });
  originalList.value = headers;
}

// select data
async function selectColumns() {
  if (data.filePath === "") {
    ElMessage.warning("未选择文件");
    return;
  }
  if (selectList.value.length === 0) {
    ElMessage.warning("未选择columns");
    return;
  }

  const names = computed(() => {
    return selectList.value.map(item => item.name).join("|");
  });

  isLoading.value = true;
  isPath.value = true;
  if (data.filePath != "") {
    ElMessage.info("Running...");
    await invoke("select", {
      path: data.filePath,
      sep: data.sep,
      cols: names.value
    });
    isLoading.value = false;
    isFinish.value = true;
    ElMessage.success("done.");
  }
}
</script>

<template>
  <div class="flex flex-col">
    <!-- Top section -->
    <el-form>
      <div
        style="
          display: flex;
          justify-content: space-between;
          align-items: center;
          width: 100%;
        "
      >
        <div style="display: flex; align-items: center">
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
            @click="selectColumns()"
            :icon="Cherry"
            plain
            style="margin-left: 16px"
          >
            Select
          </el-button>
        </div>
        <!-- Loading status -->
        <el-form-item>
          <el-icon v-if="isLoading" color="#FF8C00" class="is-loading">
            <Loading />
          </el-icon>
          <el-icon v-if="isFinish" color="#32CD32"> <SuccessFilled /> </el-icon>
        </el-form-item>

        <!-- Title -->
        <el-text type="primary" size="large">
          <el-icon> <Cherry /> </el-icon>
          <span v-if="isPath">{{ data.filePath }}</span>
          <span v-else>Select, re-order, duplicate or drop columns</span>
        </el-text>
      </div>
    </el-form>

    <!-- Middle and Bottom sections -->
    <div class="flex grow mt-4">
      <div class="w-full">
        <div class="text-center mb-2">Original Columns</div>
        <el-form
          class="flex-grow mr-4"
          style="display: flex; flex-direction: column; align-items: flex-start"
        >
          <VueDraggable
            class="flex flex-col gap-2 p-4 w-full h-full bg-gray-500/5 rounded overflow-auto"
            v-model="originalList"
            animation="150"
            ghostClass="ghost"
            group="selectGroup"
          >
            <div
              v-for="item in originalList"
              :key="item.id"
              class="cursor-move h-30 bg-gray-500/5 rounded p-3"
            >
              {{ item.name }}
            </div>
          </VueDraggable>
        </el-form>
      </div>
      <div class="w-full">
        <div class="text-center mb-2">Select Columns</div>
        <el-form
          class="flex-grow"
          style="display: flex; flex-direction: column; align-items: flex-start"
        >
          <VueDraggable
            class="flex flex-col gap-2 p-4 w-full h-full bg-gray-500/5 rounded overflow-auto"
            v-model="selectList"
            animation="150"
            group="selectGroup"
            ghostClass="ghost"
          >
            <div
              v-for="item in selectList"
              :key="item.id"
              class="cursor-move h-30 bg-gray-500/5 rounded p-3"
            >
              {{ item.name }}
            </div>
          </VueDraggable>
        </el-form>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
.flex {
  overflow: hidden;
}
</style>

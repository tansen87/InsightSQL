<script setup lang="ts">
import { ref, reactive, computed } from "vue";
import { VueDraggable } from "vue-draggable-plus";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { Cherry, FolderOpened } from "@element-plus/icons-vue";

const data = reactive({
  filePath: "",
  fileFormats: ["*"]
});
const originalList = ref([]);
const selectList = ref([]);
const isLoading = ref(false);
const isPath = ref(false);

// open file
async function selectFile() {
  isLoading.value = false;
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

  try {
    const headers: any = await invoke("get_select_headers", {
      path: data.filePath
    });
    originalList.value = headers;

    if (JSON.stringify(headers).startsWith("get header error:")) {
      throw JSON.stringify(headers).toString();
    }
  } catch (err) {
    ElNotification({
      title: "Open file error",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
}

// select data
async function selectColumns() {
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }
  if (selectList.value.length === 0) {
    ElNotification({
      title: "Column not defined",
      message: "未选择columns",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  const names = computed(() => {
    return selectList.value.map(item => item.name).join("|");
  });

  isLoading.value = true;
  isPath.value = true;

  try {
    const result: string = await invoke("select", {
      path: data.filePath,
      cols: names.value
    });

    if (JSON.stringify(result).startsWith("Select failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Select done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
  } catch (err) {
    ElNotification({
      title: "Invoke Select Error",
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
          <el-button
            type="success"
            @click="selectColumns()"
            :loading="isLoading"
            :icon="Cherry"
            plain
            style="margin-left: 16px"
          >
            Select
          </el-button>
        </div>

        <!-- Title -->
        <el-text type="primary" size="large">
          <span v-if="isPath">{{ data.filePath }}</span>
          <span v-else>Select, re-order columns</span>
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

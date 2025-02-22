<script setup lang="ts">
import { ref, reactive, computed, watch } from "vue";
import { VueDraggable } from "vue-draggable-plus";
import { invoke } from "@tauri-apps/api/core";
import { Cherry, FolderOpened } from "@element-plus/icons-vue";
import { message } from "@/utils/message";
import { viewOpenFile } from "@/utils/view";

const [originalList, selectList, displayedList, isLoading, isPath, searchText] =
  [ref([]), ref([]), ref([]), ref(false), ref(false), ref("")];
const data = reactive({
  path: "",
  skipRows: "0"
});
const onDragEnd = () => {
  originalList.value = originalList.value.filter(
    item => !selectList.value.includes(item)
  );
};

async function selectFile() {
  originalList.value = [];
  selectList.value = [];
  isPath.value = false;
  searchText.value = "";

  data.path = await viewOpenFile(false, "csv", ["*"]);
  if (data.path === null) {
    return;
  }

  try {
    const headers: string[] = await invoke("get_select_headers", {
      path: data.path,
      skipRows: data.skipRows
    });
    originalList.value = headers;

    const updateDisplayedList = () => {
      if (!searchText.value) {
        displayedList.value = [...originalList.value];
      } else {
        displayedList.value = originalList.value.filter(item =>
          item.name.toLowerCase().includes(searchText.value.toLowerCase())
        );
      }
    };
    watch(searchText, updateDisplayedList);
    updateDisplayedList();
    isPath.value = true;
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
}

// invoke select
async function selectColumns() {
  if (data.path === "") {
    message("CSV file not selected", { type: "warning" });
    return;
  }
  if (selectList.value.length === 0) {
    message("Column not selected", { type: "warning" });
    return;
  }

  const names = computed(() => {
    return selectList.value.map(item => item.name).join("|");
  });

  isLoading.value = true;
  isPath.value = true;

  try {
    const result: string = await invoke("select", {
      path: data.path,
      cols: names.value,
      skipRows: data.skipRows
    });

    message(`Select done, elapsed time: ${result} s`, { duration: 5000 });
  } catch (err) {
    message(err.toString(), { type: "error", duration: 10000 });
  }
  isLoading.value = false;
}
</script>

<template>
  <div class="page-container">
    <div class="flex flex-col">
      <el-form>
        <div class="custom-container1">
          <div class="custom-container2">
            <el-button @click="selectFile()" :icon="FolderOpened">
              Open File
            </el-button>
            <el-tooltip content="skip rows" effect="light">
              <el-input
                v-model="data.skipRows"
                style="margin-left: 10px; width: 50px"
              />
            </el-tooltip>
            <el-button
              @click="selectColumns()"
              :loading="isLoading"
              :icon="Cherry"
              style="margin-left: 10px"
            >
              Select
            </el-button>
          </div>
          <el-text>
            <span v-if="isPath">{{ data.path }}</span>
            <span v-else>Select, re-order columns</span>
          </el-text>
        </div>
      </el-form>

      <el-input
        v-model="searchText"
        style="margin-top: 12px"
        placeholder="Type to search original headers"
      />

      <el-form
        class="flex grow mt-4"
        :style="{ height: '466px', overflowY: 'auto' }"
      >
        <div class="w-full">
          <div class="text-center mb-2">Original Columns</div>
          <div
            class="flex-grow mr-4"
            style="
              display: flex;
              flex-direction: column;
              align-items: flex-start;
            "
          >
            <VueDraggable
              class="flex flex-col gap-2 p-4 w-full h-full bg-gray-500/5 rounded overflow-auto"
              v-model="displayedList"
              animation="150"
              ghostClass="ghost"
              group="selectGroup"
              @end="onDragEnd"
            >
              <div
                v-for="item in displayedList"
                :key="item.id"
                class="cursor-move h-30 bg-gray-500/5 rounded p-3"
              >
                {{ item.name }}
              </div>
            </VueDraggable>
          </div>
        </div>

        <div class="w-full">
          <div class="text-center mb-2">Select Columns</div>
          <div
            class="flex-grow"
            style="
              display: flex;
              flex-direction: column;
              align-items: flex-start;
            "
          >
            <VueDraggable
              class="flex flex-col gap-2 p-4 w-full h-full bg-gray-500/5 rounded overflow-auto"
              v-model="selectList"
              animation="150"
              group="selectGroup"
              @end="onDragEnd"
            >
              <div
                v-for="item in selectList"
                :key="item.id"
                class="cursor-move h-30 bg-gray-500/5 rounded p-3"
              >
                {{ item.name }}
              </div>
            </VueDraggable>
          </div>
        </div>
      </el-form>
    </div>
  </div>
</template>

<style lang="scss">
.flex {
  overflow: hidden;
}
</style>

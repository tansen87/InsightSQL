<script setup lang="ts">
import { ref, reactive, computed } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { FolderOpened, Refresh } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";

const [tableData, isLoading, isPath, search] = [
  ref([]),
  ref(false),
  ref(false),
  ref("")
];
const data = reactive({
  path: "",
  fileFormats: ["*"],
  skipRows: "0"
});
const { formHeight } = useDynamicFormHeight(134);
const filterTableData = computed(() =>
  tableData.value.filter(
    (data: any) =>
      !search.value ||
      data.col1.toLowerCase().includes(search.value.toLowerCase())
  )
);

async function selectFile() {
  tableData.value = [];
  isPath.value = false;
  search.value = "";

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
    data.path = selected.toString();
  } else if (selected === null) {
    return;
  } else {
    data.path = selected;
  }

  isPath.value = true;

  try {
    const headers: string[] = await invoke("get_rename_headers", {
      path: data.path,
      skipRows: data.skipRows
    });

    if (JSON.stringify(headers).startsWith("get header error:")) {
      throw JSON.stringify(headers).toString();
    }

    for (let i = 0; i < headers.length; i++) {
      const colData = {
        col1: headers[i],
        col2: headers[i % headers.length]
      };
      tableData.value.push(colData);
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

// invoke rename
async function renameData() {
  if (data.path === "") {
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
    const headersStringArray = tableData.value.map((row: any) => row.col2);
    const headersString = headersStringArray.join(",");

    const result: string = await invoke("rename", {
      path: data.path,
      headers: headersString,
      skipRows: data.skipRows
    });

    if (JSON.stringify(result).startsWith("rename failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Rename done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
  } catch (err) {
    ElNotification({
      title: "Rename failed",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
  isLoading.value = false;
}

async function headerEdit(row: any) {
  return row;
}
</script>

<template>
  <el-form class="page-container" :style="formHeight">
    <el-form>
      <div class="custom-container1">
        <div class="custom-container2">
          <el-button @click="selectFile()" :icon="FolderOpened" plain>
            Open File
          </el-button>

          <el-tooltip content="skip rows" placement="top" effect="light">
            <el-input
              v-model="data.skipRows"
              style="margin-left: 10px; width: 50px"
              placeholder="skip rows"
            />
          </el-tooltip>

          <el-button
            @click="renameData()"
            :loading="isLoading"
            :icon="Refresh"
            plain
            style="margin-left: 10px"
          >
            Rename
          </el-button>
        </div>

        <el-text>
          <span v-if="isPath">{{ data.path }}</span>
          <span v-else>Rename the columns of a CSV</span>
        </el-text>
      </div>

      <el-table
        :data="filterTableData"
        :height="formHeight"
        style="width: 100%"
        empty-text=""
      >
        <el-table-column prop="col1" label="headers" style="width: 50%" />
        <el-table-column prop="col2" label="new headers" width="300">
          <template #default="{ row }">
            <el-input
              v-model="row.col2"
              placeholder="new header"
              class="custom-header-input"
              @blur="headerEdit(row)"
            />
          </template>
        </el-table-column>
        <el-table-column>
          <template #header>
            <el-input
              v-model="search"
              size="small"
              placeholder="Type to search headers"
            />
          </template>
        </el-table-column>
      </el-table>
    </el-form>
  </el-form>
</template>

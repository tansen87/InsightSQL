<script setup lang="ts">
import { ref, reactive } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { Watermelon, FolderOpened } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";

const tableData: any = ref([]);
const selectedFiles = ref([]);
const runtime = ref(0.0);
const isLoading = ref(false);
const tableRef = ref(null);
const data = reactive({
  filePath: "",
  fileFormats: ["*"],
  sep: "_"
});
const { formHeight } = useDynamicFormHeight(205);

listen("runtime", (event: any) => {
  runtime.value = event.payload;
});
listen("modify_err", (event: any) => {
  const modifyError = event.payload;
  ElNotification({
    title: "Modify Error",
    message: modifyError,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});

// open file
async function selectFile() {
  tableData.value = [];
  isLoading.value = false;

  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.filePath = selected.join("|").toString();
    selectedFiles.value = selected.map(file => file.match(/[^\\/]+$/)[0]);
  } else if (selected === null) {
    return;
  } else {
    data.filePath = selected;
  }

  for (let i = 0; i < selectedFiles.value.length; i++) {
    const filename = selectedFiles.value[i];
    const { baseName, extension } = splitFilename(filename);

    const filenames = {
      col1: baseName,
      col2: baseName,
      col3: extension
    };

    tableData.value.push(filenames);
  }
}

// modify filename
async function modifyFilename() {
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  const filenameArray = tableData.value.map(
    (row: any) => row.col2 + "." + row.col3
  );
  const filename = filenameArray.join("|");

  isLoading.value = true;

  await invoke("modify", {
    filePath: data.filePath,
    fileName: filename
  });

  isLoading.value = false;
  ElNotification({
    message: `Modify done, elapsed time: ${runtime.value} s`,
    position: "bottom-right",
    type: "success",
    duration: 10000
  });
}

// 同步编辑文件名
function filenameEdit(editedRow) {
  // 获取原始文件名和新文件名
  const originalFilename = editedRow.col1;
  const newFilename = `${editedRow.col2}`;

  // 分割原始文件名和新文件名
  const originalParts = originalFilename.split(data.sep);
  const newParts = newFilename.split(data.sep);

  // 初始化前缀、后缀和完全替换列表
  const prefixChanges = [];
  const suffixChanges = [];
  const fullReplacements = [];

  // 比较原始部分与新部分，找出变化的部分
  const minLen = Math.min(originalParts.length, newParts.length);

  for (let i = 0; i < minLen; i++) {
    if (originalParts[i] !== newParts[i]) {
      // 检查前缀变化
      if (!newParts[i].startsWith(originalParts[i])) {
        prefixChanges.push({
          index: i,
          change: newParts[i].substring(
            0,
            newParts[i].indexOf(originalParts[i])
          )
        });
      }

      // 检查后缀变化
      if (!newParts[i].endsWith(originalParts[i])) {
        suffixChanges.push({
          index: i,
          change: newParts[i].substring(originalParts[i].length)
        });
      }

      // 检查是否是完全替换
      if (newParts[i].indexOf(originalParts[i]) === -1) {
        fullReplacements.push({
          index: i,
          newPart: newParts[i]
        });
      }
    }
  }

  // 更新所有相关行
  tableData.value = tableData.value.map(row => {
    if (row.col1 !== originalFilename) {
      // 分割当前行的文件名
      const partsOriginal = row.col1.split(data.sep);

      // 应用前缀、后缀和完全替换的变化
      const modifiedParts = partsOriginal.map((part, index) => {
        // 检查是否需要添加前缀或后缀
        let modifiedPart = part;

        // 应用前缀变化
        prefixChanges.forEach(change => {
          if (change.index === index && !part.startsWith(change.change)) {
            modifiedPart = change.change + part;
          }
        });

        // 应用后缀变化
        suffixChanges.forEach(change => {
          if (change.index === index && !part.endsWith(change.change)) {
            modifiedPart = modifiedPart + change.change;
          }
        });

        // 应用完全替换
        fullReplacements.forEach(change => {
          if (change.index === index) {
            modifiedPart = change.newPart;
          }
        });

        return modifiedPart;
      });

      // 更新文件名和文件类型
      row.col2 = modifiedParts.join(data.sep).split(".")[0];
    }
    return row;
  });
}

function splitFilename(filename) {
  const lastIndex = filename.lastIndexOf(".");

  if (lastIndex <= 0) {
    return { baseName: filename, extension: "" };
  }

  const baseName = filename.substring(0, lastIndex);
  const extension = filename.substring(lastIndex + 1);

  return { baseName, extension };
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
            <el-option label="-" value="-" />
            <el-option label="_" value="_" />
            <el-option label=";" value=";" />
            <el-option label="." value="." />
            <el-option label="(" value="(" />
            <el-option label=")" value=")" />
            <el-option label="[" value="[" />
            <el-option label="]" value="]" />
          </el-select>
          <el-button
            type="success"
            @click="modifyFilename()"
            :loading="isLoading"
            :icon="Watermelon"
            plain
            style="margin-left: 16px"
          >
            Modify
          </el-button>
        </div>

        <el-text type="primary" size="large">
          <el-icon> <Watermelon /> </el-icon>
          <span>Batch modify filenames</span>
        </el-text>
      </div>
      <el-table
        ref="tableRef"
        :data="tableData"
        :height="formHeight"
        style="width: 100%"
      >
        <el-table-column prop="col1" label="filename" style="width: 50%" />
        <el-table-column prop="col2" label="new filename" style="width: 50%">
          <template #default="{ row }">
            <el-input
              v-model="row.col2"
              class="custom-header-input"
              @input="filenameEdit(row)"
            />
          </template>
        </el-table-column>
        <el-table-column prop="col3" label="extention" width="120px" />
      </el-table>
    </el-form>
  </el-form>
</template>

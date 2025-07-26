<script setup lang="ts">
import { ref } from "vue";
import { Handle, Position, useNode, useVueFlow } from "@vue-flow/core";
import { FolderOpened, CloseBold } from "@element-plus/icons-vue";
import { mapHeaders, viewOpenFile } from "@/utils/view";
import { message } from "@/utils/message";
import { shortFileName } from "@/utils/utils";
import { usePath, useHeaders } from "@/store/modules/flow";

const path = ref("");
const isPath = ref(false);
const headerStore = useHeaders();
const pathStore = usePath();
const node = useNode();
const { removeNodes } = useVueFlow();

function deleteBtn() {
  removeNodes(node.id);
}

async function selectFile() {
  path.value = "";
  isPath.value = false;

  path.value = await viewOpenFile(false, "csv", ["*"]);
  if (path.value === null) return;
  pathStore.path = path.value;
  isPath.value = true;
  try {
    const headers = await mapHeaders(path.value, "0");
    headerStore.headers = headers;
  } catch (err) {
    message(err.toString(), { type: "error" });
  }
}
</script>

<template>
  <div class="page-container">
    <div class="node-container">
      <div style="text-align: center; width: 100%; padding: 5px">
        <el-tooltip content="Delete" effect="light">
          <el-button
            class="del-btn"
            circle
            link
            @click="deleteBtn"
            :icon="CloseBold"
            size="small"
          />
        </el-tooltip>
        <span style="display: block; font-weight: bold; margin-bottom: 10px">
          Start
        </span>
        <el-button @click="selectFile()" :icon="FolderOpened">
          <span v-if="isPath">
            <el-tooltip :content="path" effect="light">
              <span>{{ shortFileName(path) }}</span>
            </el-tooltip>
          </span>
          <span v-else>Open File</span>
        </el-button>
      </div>
      <Handle
        type="source"
        :position="Position.Right"
        id="output"
        class="handle-style"
      />
    </div>
  </div>
</template>

<style scoped>
.del-btn {
  position: absolute;
  top: -2.5px;
  right: -2.5px;
  z-index: 10;
}
</style>

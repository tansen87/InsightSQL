<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { SwitchButton, CloseBold } from "@element-plus/icons-vue";
import { Handle, Position, useNode, useVueFlow } from "@vue-flow/core";
import { message } from "@/utils/message";
import {
  usePath,
  useFilter,
  useSelect,
  useStr,
  useNodeStore,
  getExecutionConfig,
  isValidExecutionPath
} from "@/store/modules/flow";

const isLoading = ref(false);
const pathStore = usePath();
const filterStore = useFilter();
const selectStore = useSelect();
const strStore = useStr();
const nodeStore = useNodeStore();
const node = useNode();
const { removeNodes } = useVueFlow();

function deleteBtn() {
  removeNodes(node.id);
}

// invoke flow
async function endFlow() {
  try {
    isLoading.value = true;
    const nodes = nodeStore.nodes;
    const edges = nodeStore.edges;
    const { isValid, path } = isValidExecutionPath(nodes, edges);
    if (!isValid) {
      message(
        "Flow must start with the <Start> node and end with the <End> node",
        {
          type: "warning"
        }
      );
      isLoading.value = false;
      return;
    }
    if (pathStore.path === null || pathStore.path === "") {
      message("CSV file not selected", { type: "warning" });
      isLoading.value = false;
      return;
    }
    // const path = getNodesInEdgeOrder(nodes, edges);
    const config = getExecutionConfig(path, {
      selectStore,
      filterStore,
      strStore
    });
    const jsonConfig = JSON.stringify(config);
    const rtime: string = await invoke("flow", {
      path: pathStore.path,
      jsonConfig: jsonConfig
    });
    isLoading.value = false;
    message(`Flow done, elapsed time: ${rtime} s`, { type: "success" });
  } catch (err) {
    isLoading.value = false;
    message(err.toString(), { type: "error" });
  }
}
</script>

<template>
  <div class="page-container">
    <div class="node-container">
      <Handle
        type="target"
        :position="Position.Left"
        id="input"
        class="handle-style"
      />
      <div style="text-align: center; padding: 5px">
        <el-button
          circle
          link
          @click="deleteBtn"
          :icon="CloseBold"
          size="small"
          style="position: absolute; top: -2.5px; right: -2.5px; z-index: 10"
        />
        <span style="display: block; font-weight: bold; margin-bottom: 6px">
          End
        </span>
        <el-button @click="endFlow()" :icon="SwitchButton" :loading="isLoading">
          Run Flow
        </el-button>
      </div>
    </div>
  </div>
</template>

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
  useRename,
  useNodeStore,
  getExecutionConfig,
  isValidExecutionPath
} from "@/store/modules/flow";

const isLoading = ref(false);
const pathStore = usePath();
const filterStore = useFilter();
const selectStore = useSelect();
const strStore = useStr();
const renameStore = useRename();
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
    const { isValid, path, reason } = isValidExecutionPath(nodes, edges);
    if (!isValid) {
      let msg = "Invalid flow configuration.";
      switch (reason) {
        case "no_start":
          msg = "Flow must start with exactly one <Start> node.";
          break;
        case "multi_start":
          msg = "Flow must have only one <Start> node. Multiple found.";
          break;
        case "no_end":
          msg = "Flow must end with exactly one <End> node.";
          break;
        case "multi_end":
          msg = "Flow must have only one <End> node. Multiple found.";
          break;
        case "no_path":
          msg = "No valid execution path from <Start> to <End>.";
          break;
        default:
          msg = "Flow validation failed.";
      }

      message(msg, { type: "warning" });
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
      strStore,
      renameStore
    });
    const jsonConfig = JSON.stringify(config);
    console.log(jsonConfig);
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
      <div class="text-center p-[5px]">
        <el-button
          circle
          link
          @click="deleteBtn"
          :icon="CloseBold"
          size="small"
          class="absolute top-[-2.5px] right-[-2.5px] z-10"
        />
        <span class="block font-bold"> End </span>
        <el-button @click="endFlow()" :icon="SwitchButton" :loading="isLoading">
          Run Flow
        </el-button>
      </div>
    </div>
  </div>
</template>

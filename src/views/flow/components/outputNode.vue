<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { SwitchButton } from "@element-plus/icons-vue";
import { Handle, Position } from "@vue-flow/core";
import { message } from "@/utils/message";
import {
  usePath,
  useFilter,
  useSelect,
  useSlice,
  useStr,
  useNode,
  getExecutionOrder,
  getExecutionConfig
} from "@/store/modules/flow";

const isLoading = ref(false);
const pathStore = usePath();
const filterStore = useFilter();
const selectStore = useSelect();
const sliceStore = useSlice();
const strStore = useStr();
const nodeStore = useNode();

// invoke flow
async function endFlow() {
  try {
    isLoading.value = true;
    if (pathStore.path === null || pathStore.path === "") {
      message("CSV file not selected", { type: "warning" });
      isLoading.value = false;
      return;
    }
    const nodes = nodeStore.nodes;
    const edges = nodeStore.edges;
    const order = getExecutionOrder(nodes, edges);
    const config = getExecutionConfig(order, {
      selectStore,
      filterStore,
      sliceStore,
      strStore
    });
    const jsonConfig = JSON.stringify(config);
    if (jsonConfig === "{}" || jsonConfig === "[]") {
      message("operation is null", { type: "warning" });
      isLoading.value = false;
      return;
    }
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
      <div style="text-align: center; width: 100%; padding: 5px">
        <span style="display: block; font-weight: bold; margin-bottom: 10px">
          End
        </span>
        <el-button @click="endFlow()" :icon="SwitchButton" :loading="isLoading">
          Run Flow
        </el-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";
import {
  VueFlow,
  useVueFlow,
  type Connection,
  MarkerType,
  type Node,
  type Edge
} from "@vue-flow/core";
import { Background } from "@vue-flow/background";
import "@vue-flow/core/dist/style.css";
import {
  Plus,
  Select,
  Delete,
  Download,
  Upload,
  ArrowRight
} from "@element-plus/icons-vue";
import SelectNode from "@/views/flow/components/selectNode.vue";
import FilterNode from "@/views/flow/components/filterNode.vue";
import StringNode from "@/views/flow/components/stringNode.vue";
import RenameNode from "@/views/flow/components/renameNode.vue";
import InputNode from "@/views/flow/components/inputNode.vue";
import {
  getExecutionConfig,
  isValidExecutionPath,
  useFilter,
  usePath,
  useRename,
  useSelect,
  useStr
} from "@/store/modules/flow";
import { useWorkflowStore } from "@/store/modules/workflow";
import { useWorkflowManager } from "@/utils/workflowManager";
import { message } from "@/utils/message";
import { invoke } from "@tauri-apps/api/core";
import { nanoid } from "nanoid";

const isLoading = ref(false);
const nodeTypes = ["start", "select", "filter", "str", "rename"];
const customNodeTypes = {
  select: SelectNode,
  filter: FilterNode,
  str: StringNode,
  rename: RenameNode,
  start: InputNode
};
const vueFlowRef = ref();
const workflowStore = useWorkflowStore();
const pathStore = usePath();
const filterStore = useFilter();
const selectStore = useSelect();
const strStore = useStr();
const renameStore = useRename();

const { getNodes, getEdges } = useVueFlow();

const initialNodes = computed(() => {
  if (!workflowStore.currentId) return [];
  return workflowStore.getWorkflowData(workflowStore.currentId)?.nodes || [];
});

const initialEdges = computed(() => {
  if (!workflowStore.currentId) return [];
  return workflowStore.getWorkflowData(workflowStore.currentId)?.edges || [];
});

const onDragStart = (event: DragEvent, type: string) => {
  event.dataTransfer?.setData("application/vueflow", type);
  event.dataTransfer.effectAllowed = "move";
};

const onDragOver = (event: DragEvent) => {
  event.preventDefault();
  event.dataTransfer.dropEffect = "move";
};

const onDrop = (event: DragEvent) => {
  event.preventDefault();
  const vueFlow = vueFlowRef.value;
  if (!vueFlow || !workflowStore.currentId) return;

  const type = event.dataTransfer?.getData("application/vueflow");
  if (!type || !nodeTypes.includes(type)) return;

  const position = vueFlow.project({ x: event.offsetX, y: event.offsetY });

  const newNode: Node = {
    id: nanoid(),
    type,
    position,
    data: { label: `${type} Node` }
  };

  workflowStore.addNode(newNode);
};

function handleConnect(connection: Connection) {
  if (!connection.source || !connection.target) return;
  if (connection.source === connection.target) return;

  const newEdge: Edge = {
    id: nanoid(),
    ...connection,
    markerEnd: {
      type: MarkerType.Arrow,
      color: "#666",
      width: 20,
      height: 20
    }
  };

  workflowStore.addEdge(newEdge);
}

function loadCurrentWorkflow() {}

const {
  createWorkflow,
  saveWorkflow,
  deleteWorkflow,
  exportWorkflow,
  importWorkflow
} = useWorkflowManager(loadCurrentWorkflow, getNodes, getEdges);

onMounted(() => {
  if (workflowStore.list.length === 0) {
    workflowStore.create("Default");
  }
});

async function runWorkflow() {
  try {
    isLoading.value = true;

    const nodes: Node[] = getNodes.value;
    const edges: Edge[] = getEdges.value;

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
        case "no_path":
          msg = "No valid execution path from <Start>.";
          break;
        default:
          msg = "Flow validation failed.";
      }
      message(msg, { type: "warning" });
      isLoading.value = false;
      return;
    }

    if (!pathStore.path) {
      message("CSV file not selected", { type: "warning" });
      isLoading.value = false;
      return;
    }

    const config = getExecutionConfig(path, {
      selectStore,
      filterStore,
      strStore,
      renameStore
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
  <div class="page-container flex flex-col h-[calc(100vh-36px)]">
    <div class="p-2 border-b flex items-center">
      <el-button @click="createWorkflow" :icon="Plus" text> New </el-button>
      <el-button
        v-if="workflowStore.currentId"
        @click="saveWorkflow"
        :icon="Select"
        text
      >
        Save
      </el-button>
      <el-button
        v-if="workflowStore.currentId && workflowStore.list.length > 1"
        @click="deleteWorkflow"
        :icon="Delete"
        type="danger"
        text
      >
        Delete
      </el-button>
      <el-button @click="exportWorkflow" :icon="Download" text>
        Export
      </el-button>
      <el-button @click="importWorkflow" :icon="Upload" text>
        Import
      </el-button>
      <el-button
        @click="runWorkflow"
        :icon="ArrowRight"
        type="success"
        text
        :loading="isLoading"
      >
        Run
      </el-button>

      <el-select
        v-if="workflowStore.list.length > 0"
        v-model="workflowStore.currentId"
        style="width: 120px"
        class="ml-auto"
      >
        <el-option
          v-for="wf in workflowStore.list"
          :key="wf.id"
          :label="wf.name"
          :value="wf.id"
        />
      </el-select>
    </div>

    <div class="flex flex-1 overflow-hidden">
      <div class="w-[80px] p-[5px] border-r border-[#ddd]">
        <div
          v-for="type in nodeTypes"
          :key="type"
          class="draggable-node mb-2 text-center cursor-move p-1 bg-gray-100 rounded"
          draggable="true"
          @dragstart="onDragStart($event, type)"
        >
          {{ type }}
        </div>
      </div>

      <div class="flex-1 relative">
        <VueFlow
          ref="vueFlowRef"
          :node-types="customNodeTypes"
          :nodes="initialNodes"
          :edges="initialEdges"
          @connect="handleConnect"
          @drop="onDrop"
          @dragover="onDragOver"
          fit-view-on-init
        >
          <Background />
        </VueFlow>
      </div>
    </div>
  </div>
</template>

<style scoped>
.draggable-node {
  padding: 5px;
  margin-bottom: 10px;
  border: 1px solid #ccc;
  cursor: grab;
  border-radius: 5px;
  text-align: center;
  user-select: none;
}
.draggable-node:hover {
  background-color: #f0e6e6;
  transform: scale(1.02);
}
.vue-flow__node {
  transition: none !important;
}
</style>

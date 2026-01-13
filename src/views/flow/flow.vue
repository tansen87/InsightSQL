<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  VueFlow,
  useVueFlow,
  type Connection,
  MarkerType
} from "@vue-flow/core";
import { Background } from "@vue-flow/background";
import "@vue-flow/core/dist/style.css";
import {
  Plus,
  Select,
  Delete,
  Download,
  Upload
} from "@element-plus/icons-vue";
import SelectNode from "@/views/flow/components/selectNode.vue";
import FilterNode from "@/views/flow/components/filterNode.vue";
import StringNode from "@/views/flow/components/stringNode.vue";
import RenameNode from "@/views/flow/components/renameNode.vue";
import InputNode from "@/views/flow/components/inputNode.vue";
import OutputNode from "@/views/flow/components/outputNode.vue";
import { useNodeStore } from "@/store/modules/flow";
import { useWorkflowStore } from "@/store/modules/workflow";
import { useWorkflowManager } from "@/utils/workflowManager";

const nodeTypes = ["start", "select", "filter", "str", "rename", "end"];
const customNodeTypes = {
  select: SelectNode,
  filter: FilterNode,
  str: StringNode,
  rename: RenameNode,
  start: InputNode,
  end: OutputNode
};
const vueFlowRef = ref();
const flowKey = ref(0);
const nodeStore = useNodeStore();
const workflowStore = useWorkflowStore();
const { addNodes, addEdges, onNodesChange, onEdgesChange, getNodes, getEdges } =
  useVueFlow();

onNodesChange(() => {
  nodeStore.nodes = getNodes.value;
});

onEdgesChange(() => {
  nodeStore.edges = getEdges.value;
});

let nodeIdCounter = 1;
function generateId() {
  return `${nodeIdCounter++}`;
}

const handleConnect = (connection: Connection) => {
  if (!connection.target) return;
  addEdges([
    {
      ...connection,
      markerEnd: {
        type: MarkerType.Arrow,
        color: "#666",
        width: 20,
        height: 20
      }
    }
  ]);
};

const onDragStart = (event: DragEvent, type: string) => {
  event.dataTransfer?.setData("application/vueflow", type);
  event.dataTransfer.effectAllowed = "move";
};

const onDrop = (event: DragEvent) => {
  event.preventDefault();
  const vueFlow = vueFlowRef.value;
  if (!vueFlow) return;

  const type = event.dataTransfer?.getData("application/vueflow");
  if (!type || !nodeTypes.includes(type)) return;

  const position = vueFlow.project({ x: event.offsetX, y: event.offsetY });

  const newNode = {
    id: generateId(),
    type,
    position,
    data: { label: `${type} Node` }
  };

  addNodes([newNode]);
};

const onDragOver = (event: DragEvent) => {
  event.preventDefault();
  event.dataTransfer.dropEffect = "move";
};

const initialNodes = computed(() => {
  if (!workflowStore.currentId) return [];
  return workflowStore.getWorkflowData(workflowStore.currentId)?.nodes || [];
});

const initialEdges = computed(() => {
  if (!workflowStore.currentId) return [];
  return workflowStore.getWorkflowData(workflowStore.currentId)?.edges || [];
});

// 加载当前选中的工作区到画布
function loadCurrentWorkflow() {
  if (!workflowStore.currentId) return;
  const data = workflowStore.getWorkflowData(workflowStore.currentId);
  if (!data) return;

  if (flowKey.value >= 9999) {
    flowKey.value = 0;
  } else {
    flowKey.value += 1;
  }
}

const {
  createWorkflow,
  saveWorkflow,
  deleteWorkflow,
  exportWorkflow,
  importWorkflow
} = useWorkflowManager(loadCurrentWorkflow);

onMounted(() => {
  // 如果还没有任何工作区,创建一个默认的
  if (workflowStore.list.length === 0) {
    workflowStore.create("Default");
  }

  // 如果有当前选中的工作区,加载它
  if (workflowStore.currentId) {
    loadCurrentWorkflow();
  }
});
</script>

<template>
  <div class="page-container flex flex-col h-[calc(100vh-36px)]">
    <div class="p-2 border-b flex items-center">
      <!-- 左侧操作按钮 -->
      <el-button @click="createWorkflow" :icon="Plus"> New </el-button>

      <el-button
        v-if="workflowStore.currentId"
        @click="saveWorkflow"
        :icon="Select"
        type="success"
      >
        Save
      </el-button>

      <el-button
        v-if="workflowStore.currentId && workflowStore.list.length > 1"
        @click="deleteWorkflow"
        :icon="Delete"
        type="danger"
      >
        Delete
      </el-button>

      <el-button @click="exportWorkflow" :icon="Download">Export</el-button>

      <el-button @click="importWorkflow" :icon="Upload">Import</el-button>

      <el-select
        v-if="workflowStore.list.length > 0"
        v-model="workflowStore.currentId"
        @change="loadCurrentWorkflow"
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
          class="draggable-node"
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
</style>

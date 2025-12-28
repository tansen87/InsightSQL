<script setup lang="ts">
import { ref } from "vue";
import {
  VueFlow,
  useVueFlow,
  type Connection,
  MarkerType
} from "@vue-flow/core";
import { Background } from "@vue-flow/background";
import "@vue-flow/core/dist/style.css";
import SelectNode from "./components/selectNode.vue";
import FilterNode from "./components/filterNode.vue";
import StringNode from "./components/stringNode.vue";
import InputNode from "./components/inputNode.vue";
import OutputNode from "./components/outputNode.vue";
import { useNodeStore } from "@/store/modules/flow";

const nodeTypes = ["start", "select", "filter", "str", "end"];
const customNodeTypes = {
  select: SelectNode,
  filter: FilterNode,
  str: StringNode,
  start: InputNode,
  end: OutputNode
};
const vueFlowRef = ref();
const nodeStore = useNodeStore();
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
</script>

<template>
  <div class="page-container flex h-[calc(100vh-36px)]">
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
        @connect="handleConnect"
        @drop="onDrop"
        @dragover="onDragOver"
      >
        <Background />
      </VueFlow>
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

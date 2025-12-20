<script setup lang="ts">
import { ref } from "vue";
import {
  VueFlow,
  useVueFlow,
  applyChanges,
  NodeChange,
  EdgeChange,
  Connection,
  MarkerType
} from "@vue-flow/core";
import { Background } from "@vue-flow/background";
import "@vue-flow/core/dist/style.css";
import "@vue-flow/controls/dist/style.css";
import SelectNode from "./components/selectNode.vue";
import FilterNode from "./components/filterNode.vue";
import StringNode from "./components/stringNode.vue";
import InputNode from "./components/inputNode.vue";
import OutputNode from "./components/outputNode.vue";
import { useNodeStore } from "@/store/modules/flow";

const vueFlowRef = ref();
const nodes = ref([]);
const edges = ref([]);
const nodeTypes = ["start", "select", "filter", "str", "end"];
const customNodeTypes = {
  select: SelectNode,
  filter: FilterNode,
  str: StringNode,
  start: InputNode,
  end: OutputNode
};
const { addEdges } = useVueFlow();
const nodeStore = useNodeStore();
let nodeIdCounter = 1;

function generateId() {
  return `${nodeIdCounter++}`;
}

const onNodesChange = (changes: NodeChange[]) => {
  const newNodes = applyChanges(changes, nodes.value);
  nodes.value = newNodes;
  nodeStore.addNode(newNodes);
};
const onEdgesChange = (changes: EdgeChange[]) => {
  const newEdge = applyChanges(changes, edges.value);
  edges.value = newEdge;
  nodeStore.addEdge(newEdge);
};
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
  const vueFlow = vueFlowRef.value;
  const type = event.dataTransfer?.getData("application/vueflow");
  const position = vueFlow.project({ x: event.offsetX, y: event.offsetY });
  const newNode = {
    id: generateId(),
    type,
    position,
    data: {
      label: `${type} Node`
    }
  };
  nodes.value = [...nodes.value, newNode];
};
const onDragOver = (event: DragEvent) => {
  event.preventDefault();
  event.dataTransfer.dropEffect = "move";
};
</script>

<template>
  <div class="page-container flex h-[90vh]">
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
        :nodes="nodes"
        :edges="edges"
        :node-types="customNodeTypes"
        @nodes-change="onNodesChange"
        @edges-change="onEdgesChange"
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
}
.draggable-node:hover {
  background-color: #cebdbd;
  transform: scale(1.02);
}
.canvas {
  flex: 1;
}
</style>

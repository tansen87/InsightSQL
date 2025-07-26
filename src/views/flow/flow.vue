<script setup lang="ts">
import { ref } from "vue";
import {
  VueFlow,
  Panel,
  addEdge,
  applyNodeChanges,
  applyEdgeChanges
} from "@vue-flow/core";
import { Background } from "@vue-flow/background";
import "@vue-flow/core/dist/style.css";
import "@vue-flow/controls/dist/style.css";
import SelectNode from "./components/selectNode.vue";
import FilterNode from "./components/filterNode.vue";
import SliceNode from "./components/sliceNode.vue";
import StringNode from "./components/stringNode.vue";
import InputNode from "./components/inputNode.vue";
import OutputNode from "./components/outputNode.vue";
import { useNode } from "@/store/modules/flow";

const vueFlowRef = ref();
const nodes = ref([]);
const edges = ref([]);
const nodeTypes = ["start", "select", "filter", "str", "slice", "end"];
const customNodeTypes = {
  select: SelectNode,
  filter: FilterNode,
  slice: SliceNode,
  str: StringNode,
  start: InputNode,
  end: OutputNode
};
const nodeStore = useNode();
let nodeIdCounter = 1;

function generateId() {
  return `${nodeIdCounter++}`;
}
const onNodesChange = changes => {
  const newNodes = applyNodeChanges(changes, nodes.value);
  nodes.value = newNodes;
  nodeStore.addNode(newNodes);
};
const onEdgesChange = changes => {
  const newEdge = applyEdgeChanges(changes, edges.value);
  edges.value = newEdge;
  nodeStore.addEdge(newEdge);
};
const onConnect = params => {
  edges.value = addEdge(params, edges.value);
};
const onDragStart = (event, type) => {
  event.dataTransfer.setData("application/vueflow", type);
  event.dataTransfer.effectAllowed = "move";
};
const onDrop = event => {
  const vueFlow = vueFlowRef.value;
  const type = event.dataTransfer.getData("application/vueflow");
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
const onDragOver = event => {
  event.preventDefault();
  event.dataTransfer.dropEffect = "move";
};
</script>

<template>
  <div class="page-container" style="display: flex; height: 90vh">
    <div class="sidebar">
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
    <div style="flex: 1; position: relative">
      <VueFlow
        ref="vueFlowRef"
        :nodes="nodes"
        :edges="edges"
        :node-types="customNodeTypes"
        @nodes-change="onNodesChange"
        @edges-change="onEdgesChange"
        @connect="onConnect"
        @drop="onDrop"
        @dragover="onDragOver"
      >
        <Panel position="top-right">
          <span>Csv Flow</span>
        </Panel>
        <Background />
      </VueFlow>
    </div>
  </div>
</template>

<style scoped>
.sidebar {
  width: 100px;
  padding: 5px;
  border-right: 1px solid #ddd;
}
.draggable-node {
  padding: 10px;
  margin-bottom: 10px;
  border: 1px solid #ccc;
  cursor: grab;
}
.canvas {
  flex: 1;
}
</style>

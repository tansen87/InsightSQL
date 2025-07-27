import { defineStore } from "pinia";
// import { Node, Edge } from "@vue-flow/core";

// export function getExecutionOrder(nodes: Node[], edges: Edge[]) {
//   const startNode = nodes.find(n => !edges.some(e => e.target === n.id));
//   if (!startNode) return [];
//   const order = [];
//   let currentId = startNode.id;
//   while (currentId) {
//     const node = nodes.find(n => n.id === currentId);
//     if (!node) break;
//     order.push(node);
//     if (node.type === "end") break;
//     const edge = edges.find(e => e.source === currentId);
//     currentId = edge ? edge.target : null;
//   }
//   return order;
// }

export function getNodesInEdgeOrder(nodes, edges) {
  const nodeMap = new Map(nodes.map(n => [n.id, n]));
  const graph = new Map();
  edges.forEach(edge => {
    if (!graph.has(edge.source)) graph.set(edge.source, []);
    graph.get(edge.source).push(edge.target);
  });
  const visited = new Set();
  const result = [];
  function dfs(id) {
    if (visited.has(id)) return;
    visited.add(id);
    result.push(nodeMap.get(id));
    (graph.get(id) || []).forEach(dfs);
  }
  const startNodes = nodes.filter(n => !edges.some(e => e.target === n.id));
  startNodes.forEach(n => dfs(n.id));

  return result;
}

export function getExecutionConfig(order, stores) {
  return order
    .map(node => {
      if (node.type === "start" || node.type === "end") {
        return null;
      }
      switch (node.type) {
        case "select":
          return stores.selectStore.selects.find(s => s.id === node.id);
        case "filter":
          return stores.filterStore.filters.find(f => f.id === node.id);
        case "slice":
          return stores.sliceStore.slices.find(s => s.id === node.id);
        case "str":
          return stores.strStore.strs.find(s => s.id === node.id);
        default:
          return null;
      }
    })
    .filter(Boolean);
}

// mapHeaders
export const useHeaders = defineStore("headers", {
  state: () => ({
    headers: [] as Array<{ label: string; value: string }>
  })
});

// path
export const usePath = defineStore("path", {
  state: () => ({
    path: "" as string
  })
});

// node
export const useNodeStore = defineStore("node", {
  state: () => ({
    nodes: [],
    edges: []
  }),
  actions: {
    addNode(node) {
      this.nodes = node;
    },
    addEdge(edge) {
      this.edges = edge;
    }
  }
});

// filter node
export const useFilter = defineStore("filter", {
  state: () => ({
    filters: [] as Array<{
      id: string;
      op: string;
      mode: string;
      column: string;
      value: string;
    }>
  }),
  actions: {
    addFilter(data: {
      id: string;
      op: string;
      mode: string;
      column: string;
      value: string;
    }) {
      const index = this.filters.findIndex(f => f.id === data.id);
      if (index > -1) {
        this.filters[index] = data;
      } else {
        this.filters.push(data);
      }
    }
  }
});

// select node
export const useSelect = defineStore("select", {
  state: () => ({
    selects: [] as Array<{
      id: string;
      op: string;
      column: string;
    }>
  }),
  actions: {
    addSelect(data: { id: string; op: string; column: string }) {
      const index = this.selects.findIndex(f => f.id === data.id);
      if (index > -1) {
        this.selects[index] = data;
      } else {
        this.selects.push(data);
      }
    }
  }
});

// slice node
export const useSlice = defineStore("slice", {
  state: () => ({
    slices: [] as Array<{
      id: string;
      op: string;
      mode: string;
      column: string;
      offset: string;
      length: string;
    }>
  }),
  actions: {
    addSlice(data: {
      id: string;
      op: string;
      mode: string;
      column: string;
      offset: string;
      length: string;
    }) {
      const index = this.slices.findIndex(s => s.id === data.id);
      if (index > -1) {
        this.slices[index] = data;
      } else {
        this.slices.push(data);
      }
    }
  }
});

// str node
export const useStr = defineStore("str", {
  state: () => ({
    strs: [] as Array<{
      id: string;
      op: string;
      mode: string;
      column: string;
      comparand: string;
      replacement: string;
    }>
  }),
  actions: {
    addString(data: {
      id: string;
      op: string;
      mode: string;
      column: string;
      comparand: string;
      replacement: string;
    }) {
      const index = this.strs.findIndex(s => s.id === data.id);
      if (index > -1) {
        this.strs[index] = data;
      } else {
        this.strs.push(data);
      }
    }
  }
});

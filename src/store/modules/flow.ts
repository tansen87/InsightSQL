import { defineStore } from "pinia";
import type { Node, Edge } from "@vue-flow/core";

export function getNodesInEdgeOrder(nodes: Node[], edges: Edge[]): Node[] {
  const nodeMap = new Map(nodes.map(n => [n.id, n]));
  const graph = new Map<string, string[]>();

  edges.forEach(edge => {
    if (edge.source && edge.target) {
      if (!graph.has(edge.source)) graph.set(edge.source, []);
      graph.get(edge.source)!.push(edge.target);
    }
  });

  const visited = new Set<string>();
  const result: Node[] = [];

  function dfs(id: string) {
    if (visited.has(id)) return;
    visited.add(id);

    const node = nodeMap.get(id);
    if (node) result.push(node);

    const neighbors = graph.get(id) || [];
    neighbors.forEach(neighbor => dfs(neighbor));
  }

  const startNodes = nodes.filter(
    n =>
      edges.some(e => e.source === n.id) && !edges.some(e => e.target === n.id)
  );

  startNodes.forEach(n => dfs(n.id));

  return result;
}

// Flow must start with the <Start> node and end with the <End> node
export function isValidExecutionPath(
  nodes: Node[],
  edges: Edge[]
): { isValid: boolean; path: Node[]; reason?: string } {
  const nodeMap = new Map(nodes.map(n => [n.id, n]));
  const graph = new Map<string, string[]>();

  edges.forEach(edge => {
    if (edge.source && edge.target) {
      if (!graph.has(edge.source)) graph.set(edge.source, []);
      graph.get(edge.source)!.push(edge.target);
    }
  });

  const startNodes = nodes.filter(n => n.type === "start");
  const endNodes = nodes.filter(n => n.type === "end");

  if (startNodes.length === 0)
    return { isValid: false, path: [], reason: "no_start" };
  if (endNodes.length === 0)
    return { isValid: false, path: [], reason: "no_end" };
  if (startNodes.length > 1)
    return { isValid: false, path: [], reason: "multi_start" };
  if (endNodes.length > 1)
    return { isValid: false, path: [], reason: "multi_end" };

  const startNode = startNodes[0];
  const endNode = endNodes[0];

  // BFS
  const visited = new Set<string>();
  const queue: { id: string; path: string[] }[] = [];
  queue.push({ id: startNode.id, path: [startNode.id] });
  visited.add(startNode.id);

  while (queue.length > 0) {
    const { id, path } = queue.shift()!;

    const neighbors = graph.get(id) || [];
    for (const neighbor of neighbors) {
      if (neighbor === endNode.id) {
        const fullPath = [...path, neighbor]
          .map(id => nodeMap.get(id)!)
          .filter(Boolean);
        return { isValid: true, path: fullPath };
      }

      if (!visited.has(neighbor)) {
        visited.add(neighbor);
        queue.push({ id: neighbor, path: [...path, neighbor] });
      }
    }
  }

  return { isValid: false, path: [], reason: "no_path" };
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
  }),

  actions: {
    setHeaderForNode(nodeId: string, label: string) {
      if (label.trim() === "") {
        // 如果label无效,从headers中移除
        this.headers = this.headers.filter(h => h.value !== nodeId);
        return;
      }

      const existingIndex = this.headers.findIndex(h => h.value === nodeId);
      if (existingIndex >= 0) {
        this.headers[existingIndex].label = label;
      } else {
        this.headers.push({ label, value: nodeId });
      }
    }
  }
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
    addStr(data: {
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

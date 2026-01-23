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

// Flow must start with the <Start> node
export function isValidExecutionPath(
  nodes: Node[],
  edges: Edge[]
): { isValid: boolean; path: Node[]; reason?: string } {
  const nodeMap = new Map(nodes.map(n => [n.id, n]));
  const graph = new Map<string, string[]>();
  const reverseGraph = new Map<string, string[]>();

  edges.forEach(edge => {
    if (edge.source && edge.target) {
      if (!graph.has(edge.source)) graph.set(edge.source, []);
      graph.get(edge.source)!.push(edge.target);

      if (!reverseGraph.has(edge.target)) reverseGraph.set(edge.target, []);
      reverseGraph.get(edge.target)!.push(edge.source);
    }
  });

  const startNodes = nodes.filter(n => n.type === "start");
  if (startNodes.length === 0) {
    return { isValid: false, path: [], reason: "no_start" };
  }
  if (startNodes.length > 1) {
    return { isValid: false, path: [], reason: "multi_start" };
  }

  const startNode = startNodes[0];

  const leafNodes = nodes.filter(node => {
    if (node.id === startNode.id && !graph.has(node.id)) return true;
    return !graph.has(node.id) && node.type !== "start";
  });

  if (leafNodes.length === 0) {
    return { isValid: false, path: [], reason: "no_leaf_node" };
  }

  const visited = new Set<string>();
  const queue: { id: string; path: string[] }[] = [];
  queue.push({ id: startNode.id, path: [startNode.id] });
  visited.add(startNode.id);

  while (queue.length > 0) {
    const { id, path } = queue.shift()!;

    const neighbors = graph.get(id) || [];
    if (neighbors.length === 0) {
      const fullPath = path.map(pid => nodeMap.get(pid)!).filter(Boolean);
      return { isValid: true, path: fullPath };
    }

    for (const neighbor of neighbors) {
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
        case "rename":
          return stores.renameStore.renames.find(r => r.id === node.id);
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

  getters: {
    usedFieldNames(state) {
      return new Set(state.headers.map(h => h.label));
    },
    getFieldNameByNodeId: state => (nodeId: string) => {
      const item = state.headers.find(h => h.value === nodeId);
      return item ? item.label : null;
    }
  },

  actions: {
    setHeaderForNode(nodeId: string, baseLabel: string) {
      if (!baseLabel?.trim()) {
        this.headers = this.headers.filter(h => h.value !== nodeId);
        return;
      }

      let finalLabel = baseLabel.trim();
      let counter = 1;

      while (this.usedFieldNames.has(finalLabel)) {
        const existingItem = this.headers.find(h => h.label === finalLabel);
        if (existingItem && existingItem.value === nodeId) {
          break;
        }
        finalLabel = `${baseLabel.trim()}_${counter}`;
        counter++;
      }

      // 更新或添加
      const existingIndex = this.headers.findIndex(h => h.value === nodeId);
      if (existingIndex >= 0) {
        this.headers[existingIndex].label = finalLabel;
      } else {
        this.headers.push({ label: finalLabel, value: nodeId });
      }
    },

    removeHeaderForNode(nodeId: string) {
      this.headers = this.headers.filter(h => h.value !== nodeId);
    }
  },

  persist: true
});

// path
export const usePath = defineStore("path", {
  state: () => ({
    path: "" as string
  }),
  persist: true
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
  },
  persist: true
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
  },
  persist: true
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
  },
  persist: true
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
  },
  persist: true
});

// rename node
export const useRename = defineStore("rename", {
  state: () => ({
    renames: [] as Array<{
      id: string;
      op: string;
      column: string;
      value: string;
    }>
  }),
  actions: {
    addRename(data: { id: string; op: string; column: string; value: string }) {
      const index = this.renames.findIndex(s => s.id === data.id);
      if (index > -1) {
        this.renames[index] = data;
      } else {
        this.renames.push(data);
      }
    }
  },
  persist: true
});

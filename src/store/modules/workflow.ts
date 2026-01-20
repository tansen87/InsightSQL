import { defineStore } from "pinia";
import { nanoid } from "nanoid";
import type { Node, Edge } from "@vue-flow/core";

interface Workflow {
  id: string;
  name: string;
  nodes: Node[];
  edges: Edge[];
  createdAt: string;
  updatedAt: string;
}

export const useWorkflowStore = defineStore("workflow", {
  state: () => ({
    // 当前选中的工作区ID
    currentId: null as string | null,
    // 所有工作区列表
    list: [] as Workflow[]
  }),

  getters: {
    currentWorkflow: state => {
      return state.list.find(w => w.id === state.currentId) || null;
    }
  },

  actions: {
    // 创建新工作区(不自动加载节点/边,等用户拖拽后手动保存)
    create(name: string) {
      const newWorkflow: Workflow = {
        id: nanoid(),
        name,
        nodes: [],
        edges: [],
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString()
      };
      this.list.push(newWorkflow);
      this.currentId = newWorkflow.id;
      return newWorkflow.id;
    },

    addNode(node: Node) {
      if (!this.currentId) return;
      const workflow = this.list.find(w => w.id === this.currentId);
      if (workflow) {
        workflow.nodes = [...workflow.nodes, node];
        workflow.updatedAt = new Date().toISOString();
      }
    },

    addEdge(edge: Edge) {
      if (!this.currentId) return;
      const workflow = this.list.find(w => w.id === this.currentId);
      if (workflow) {
        workflow.edges = [...workflow.edges, { ...edge }];
        workflow.updatedAt = new Date().toISOString();
      }
    },

    // 手动保存当前画布状态到当前工作区
    saveCurrent(nodes: Node[], edges: Edge[]) {
      if (!this.currentId) {
        throw new Error("No selected workspace");
      }
      const workflow = this.list.find(w => w.id === this.currentId)!;
      workflow.nodes = [...nodes];
      workflow.edges = [...edges];
      workflow.updatedAt = new Date().toISOString();
    },

    // 切换工作区(用于加载)
    switchTo(id: string) {
      const exists = this.list.some(w => w.id === id);
      if (!exists) throw new Error("The workspace does not exist");
      this.currentId = id;
    },

    // 删除工作区
    remove(id: string) {
      this.list = this.list.filter(w => w.id !== id);
      if (this.currentId === id) {
        this.currentId = this.list[0]?.id || null;
      }
    },

    removeNodes(nodeIds: string[]) {
      if (!this.currentId) return;
      const workflow = this.list.find(w => w.id === this.currentId);
      if (!workflow) return;

      const newNodes = workflow.nodes.filter(n => !nodeIds.includes(n.id));
      const newEdges = workflow.edges.filter(
        e => !nodeIds.includes(e.source) && !nodeIds.includes(e.target)
      );

      workflow.nodes = newNodes;
      workflow.edges = newEdges;
      workflow.updatedAt = new Date().toISOString();
    },

    removeEdges(edgeIds: string[]) {
      if (!this.currentId) return;
      const workflow = this.list.find(w => w.id === this.currentId);
      if (workflow) {
        workflow.edges = workflow.edges.filter(e => !edgeIds.includes(e.id));
        workflow.updatedAt = new Date().toISOString();
      }
    },

    // 加载指定工作区的节点和边(返回数据,由组件负责addNodes/addEdges)
    getWorkflowData(id: string) {
      const wf = this.list.find(w => w.id === id);
      if (!wf) throw new Error("The workspace does not exist");
      return { nodes: wf.nodes, edges: wf.edges };
    }
  },

  // 启用持久化(刷新不丢失)
  persist: true
});

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  VueFlow,
  useVueFlow,
  type Connection,
  MarkerType
} from "@vue-flow/core";
import { Background } from "@vue-flow/background";
import { save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import "@vue-flow/core/dist/style.css";
import {
  Plus,
  Select,
  Delete,
  Download,
  Upload
} from "@element-plus/icons-vue";
import { ElMessageBox } from "element-plus";
import SelectNode from "./components/selectNode.vue";
import FilterNode from "./components/filterNode.vue";
import StringNode from "./components/stringNode.vue";
import InputNode from "./components/inputNode.vue";
import OutputNode from "./components/outputNode.vue";
import { useNodeStore } from "@/store/modules/flow";
import { useWorkflowStore } from "@/store/modules/workflow";
import { trimOpenFile } from "@/utils/view";
import { message } from "@/utils/message";

const nodeTypes = ["start", "select", "filter", "str", "end"];
const customNodeTypes = {
  select: SelectNode,
  filter: FilterNode,
  str: StringNode,
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

// 保存当前画布
function saveWorkflow() {
  if (!workflowStore.currentId) {
    message("Please create or select a workspace first", { type: "warning" });
    return;
  }
  workflowStore.saveCurrent(getNodes.value, getEdges.value);
  message("Workflow saved", { type: "success" });
}

// 创建新工作区
async function createWorkflow() {
  try {
    const { value } = await ElMessageBox.prompt("New workspace", {
      confirmButtonText: "Ok",
      cancelButtonText: "Cancel",
      inputPlaceholder: "Example name: ws001",
      inputValidator: val => {
        const name = val?.trim();
        if (!name) {
          return "The workspace name cannot be empty";
        }
        // 检查是否已存在
        const exists = workflowStore.list.some(wf => wf.name === name);
        if (exists) {
          return "This name already exists. Please choose another one.";
        }
      }
    });

    const name = value?.trim();
    if (name) {
      workflowStore.create(name);
    }
  } catch (error) {
    // 点击取消或关闭弹窗,静默处理
  }
}

// 加载当前选中的工作区到画布
function loadCurrentWorkflow() {
  if (!workflowStore.currentId) return;
  const data = workflowStore.getWorkflowData(workflowStore.currentId);
  if (!data) return;

  flowKey.value += 1;
}

// 删除工作区
function deleteWorkflow() {
  if (!workflowStore.currentId) return;
  const current = workflowStore.currentWorkflow;
  if (!current) return;

  ElMessageBox.confirm(
    `Are you sure you want to delete workspace "${current.name}", this operation is irreversible`,
    "Delete workspace",
    {
      confirmButtonText: "Ok",
      cancelButtonText: "Cancel",
      type: "warning"
    }
  )
    .then(() => {
      workflowStore.remove(workflowStore.currentId!);
      // 如果还有其他工作区,自动加载第一个,否则创建新默认
      if (workflowStore.list.length > 0) {
        workflowStore.currentId = workflowStore.list[0].id;
        loadCurrentWorkflow();
      } else {
        createWorkflow();
      }
    })
    .catch(() => {});
}

// 导出工作区
async function exportWorkflow() {
  const current = workflowStore.currentWorkflow;

  try {
    const outputPath = await save({
      title: "Export Workflow",
      defaultPath: `workflow_${current.name}.json`,
      filters: [{ name: "JSON", extensions: ["json"] }]
    });

    if (!outputPath) {
      return;
    }

    await writeTextFile(outputPath, JSON.stringify(current, null, 2));

    message(`Workflow saved to: ${outputPath}`, { type: "success" });
  } catch (err) {
    message(`Failed to export workflow: ${err}`, { type: "error" });
  }
}

// 导入工作区
async function importWorkflow() {
  const trimFile = await trimOpenFile(false, "Workflow", ["json"], {
    includeStatus: false
  });
  const selected = trimFile?.filePath;

  if (!selected) return;

  try {
    const content = await readTextFile(selected);
    const workflow = JSON.parse(content);

    if (!workflow?.id || !Array.isArray(workflow.nodes)) {
      throw new Error("Invalid workflow file");
    }

    // 检查是否已存在相同ID的工作区
    const exists = workflowStore.list.some(w => w.id === workflow.id);
    if (exists) {
      message(`Workflow "${workflow.name}" already exists`, {
        type: "warning"
      });
      return;
    }

    workflowStore.list.push(workflow);
    workflowStore.currentId = workflow.id;
    loadCurrentWorkflow();

    message(`Imported workflow: ${workflow.name}`, { type: "success" });
  } catch (err) {
    message(`Failed to import workflow: ${err}`, { type: "error" });
  }
}

onMounted(() => {
  // 如果还没有任何工作区,创建一个默认的
  if (workflowStore.list.length === 0) {
    workflowStore.create("Default");
  }

  // 如果有当前选中的工作区，加载它
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

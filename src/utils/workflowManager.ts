import { ElMessageBox } from "element-plus";
import { save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { useVueFlow } from "@vue-flow/core";
import { message } from "@/utils/message";
import { trimOpenFile } from "@/utils/view";
import { useWorkflowStore } from "@/store/modules/workflow";

export function useWorkflowManager(loadCurrentWorkflow: () => void) {
  const workflowStore = useWorkflowStore();
  const { getNodes, getEdges } = useVueFlow();

  // 创建新工作区
  async function createWorkflow() {
    ElMessageBox.prompt("New workspace", {
      confirmButtonText: "Ok",
      cancelButtonText: "Cancel",
      inputPlaceholder: "Example name: ws001",
      inputValidator: val => {
        const name = val?.trim();
        if (!name) {
          return "The workspace name cannot be empty";
        }
        // 检查是否存在
        const exists = workflowStore.list.some(wf => wf.name === name);
        if (exists) {
          return "This name already exists. Please choose another one.";
        }
      }
    })
      .then(({ value }) => {
        const name = value?.trim();
        if (name) {
          workflowStore.create(name);
        }
      })
      .catch(() => {});
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

  // 保存当前画布
  function saveWorkflow() {
    if (!workflowStore.currentId) {
      message("Please create or select a workspace first", { type: "warning" });
      return;
    }
    workflowStore.saveCurrent(getNodes.value, getEdges.value);
    message("Workflow saved", { type: "success" });
  }

  return {
    createWorkflow,
    saveWorkflow,
    deleteWorkflow,
    exportWorkflow,
    importWorkflow
  };
}

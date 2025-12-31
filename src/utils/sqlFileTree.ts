import { computed, reactive, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { message } from "@/utils/message";
import { useSqlHistory } from "@/store/modules/sqlHistory";
import fileIcon from "@iconify-icons/ri/file-line";
import fileTextIcon from "@iconify-icons/ri/file-text-line";
import fileExcelIcon from "@iconify-icons/ri/file-excel-2-line";
import fileJsonIcon from "@iconify-icons/ri/file-code-line";
import fileParquetIcon from "@iconify-icons/ri/database-2-line";
import textIcon from "@iconify-icons/ri/text";
import intIcon from "@iconify-icons/ri/number-1";
import floatIcon from "@iconify-icons/ri/hashtag";
import boolIcon from "@iconify-icons/ri/checkbox-circle-line";
import dateIcon from "@iconify-icons/ri/calendar-event-line";
import listIcon from "@iconify-icons/ri/list-unordered";
import unknowIcon from "@iconify-icons/ri/question-mark";

export function useSqlFileTree() {
  const sqlHistory = useSqlHistory();
  const contextMenuVisible = ref(false);
  const contextMenuPosition = reactive({ x: 0, y: 0 });
  const contextMenuItem = ref<any>(null);

  const viewFileMeta = computed(() => {
    if (!sqlHistory.path) return [];
    return sqlHistory.path.split("|").map(path => {
      const fullFileName = path.split(/[/\\]/).pop() || path;
      const ext = fullFileName.includes(".")
        ? fullFileName.slice(fullFileName.lastIndexOf(".") + 1).toLowerCase()
        : "";
      return { fullPath: path, fullFileName, ext };
    });
  });

  const fileTreeData = computed(() => {
    return viewFileMeta.value
      .map(fileMeta => {
        const schema = sqlHistory.dtypesByFile[fileMeta.fullFileName];
        if (!schema || Object.keys(schema).length === 0) return null;

        const children = Object.entries(schema).map(([field, dtype]) => ({
          label: field,
          dtype,
          key: `${fileMeta.fullFileName}-${field}`,
          type: "field"
        }));

        return {
          label: fileMeta.fullFileName,
          ext: fileMeta.ext,
          children,
          key: fileMeta.fullFileName,
          type: "file",
          fullPath: fileMeta.fullPath
        };
      })
      .filter(Boolean);
  });

  const getFileIcon = (ext: string) => {
    switch (ext) {
      case "csv":
      case "tsv":
      case "txt":
      case "dat":
        return fileTextIcon;
      case "xlsx":
      case "xls":
      case "xlsb":
      case "xlsm":
      case "ods":
        return fileExcelIcon;
      case "json":
      case "jsonl":
      case "ndjson":
        return fileJsonIcon;
      case "parquet":
        return fileParquetIcon;
      default:
        return fileIcon;
    }
  };

  const getFieldIcon = (dtype: string) => {
    const d = dtype.toLowerCase();
    if (d.includes("str") || d.includes("utf8")) return textIcon;
    if (d.includes("i64")) return intIcon;
    if (d.includes("f64")) return floatIcon;
    if (d.includes("bool")) return boolIcon;
    if (d.includes("date") || d.includes("time")) return dateIcon;
    if (d.includes("list") || d.includes("struct")) return listIcon;
    return unknowIcon;
  };

  const getNodeIcon = (node: any) => {
    return node.type === "file"
      ? getFileIcon(node.ext || "")
      : getFieldIcon(node.dtype || "");
  };

  async function selectFile() {
    const selected = await open({
      multiple: true,
      filters: [
        { name: "All", extensions: ["*"] },
        { name: "csv", extensions: ["csv", "tsv", "psv", "txt", "dat"] },
        { name: "excel", extensions: ["xls", "xlsx", "xlsb", "xlsm", "ods"] },
        { name: "json", extensions: ["json"] },
        { name: "jsonl", extensions: ["jsonl", "ndjson"] },
        { name: "parquet", extensions: ["parquet"] }
      ]
    });

    if (!selected) return;

    const newPaths = Array.isArray(selected) ? selected : [selected];
    const existingPaths = sqlHistory.path ? sqlHistory.path.split("|") : [];
    const allPathsSet = new Set([...existingPaths, ...newPaths]);
    sqlHistory.path = Array.from(allPathsSet).join("|");

    await Promise.all(
      newPaths.map(async path => {
        const fullFileName = path.split(/[/\\]/).pop() || path;
        if (sqlHistory.dtypesByFile[fullFileName]) return;

        try {
          const rawResult = await invoke("query", {
            path,
            sqlQuery: `SELECT * FROM "${fullFileName}" LIMIT 10`,
            varchar: false,
            limit: true,
            write: false,
            writeFormat: "csv",
            outputPath: ""
          });
          const result =
            typeof rawResult === "string" ? JSON.parse(rawResult) : rawResult;
          sqlHistory.dtypesByFile[fullFileName] = result.schema;
        } catch (err) {
          message(`Failed to load schema for ${fullFileName}: ${err}`, {
            type: "error"
          });
        }
      })
    );
  }

  function closeContextMenu() {
    contextMenuVisible.value = false;
    contextMenuItem.value = null;
  }

  function openContextMenu(event: MouseEvent, nodeData: any) {
    event.preventDefault();
    contextMenuItem.value = nodeData;
    contextMenuPosition.x = event.clientX;
    contextMenuPosition.y = event.clientY;
    contextMenuVisible.value = true;
  }

  async function copyPath() {
    if (!contextMenuItem.value?.fullPath) return;
    try {
      await navigator.clipboard.writeText(contextMenuItem.value.fullPath);
      message("Copied file path", { type: "success" });
      closeContextMenu();
    } catch (err) {
      message("Failed to copy", { type: "error" });
    }
  }

  async function copyFileName() {
    const item = contextMenuItem.value;
    if (!item) return;
    const textToCopy =
      item.type === "file" ? item.fullFileName || item.label : item.label;
    try {
      await navigator.clipboard.writeText(textToCopy);
      message("Copied file name", { type: "success" });
      closeContextMenu();
    } catch (err) {
      message("Failed to copy", { type: "error" });
    }
  }

  async function copyFieldName() {
    if (!contextMenuItem.value?.label) return;
    try {
      await navigator.clipboard.writeText(contextMenuItem.value.label);
      message("Copied field name", { type: "success" });
      closeContextMenu();
    } catch (err) {
      message("Failed to copy", { type: "error" });
    }
  }

  function deleteFile() {
    const item = contextMenuItem.value;
    if (!item || item.type !== "file") return;

    const fullFileName = item.fullFileName || item.label;
    const fullPath = item.fullPath;

    const paths = sqlHistory.path.split("|").filter(p => p !== fullPath);
    sqlHistory.path = paths.join("|");

    // 删除schema缓存
    delete sqlHistory.dtypesByFile[fullFileName];

    message(`Deleted "${fullFileName}"`, { type: "success" });
    closeContextMenu();
  }

  function rightClick(event: MouseEvent, nodeData: any) {
    const dataForMenu = { ...nodeData };
    if (nodeData.type === "file" && !dataForMenu.fullPath) {
      const meta = viewFileMeta.value.find(
        m => m.fullFileName === nodeData.label
      );
      if (meta) {
        dataForMenu.fullPath = meta.fullPath;
        dataForMenu.fullFileName = meta.fullFileName;
      }
    }
    openContextMenu(event, dataForMenu);
  }

  return {
    // 数据
    viewFileMeta,
    fileTreeData,
    getNodeIcon,

    // 文件操作
    selectFile,

    // 右键菜单状态
    contextMenuVisible,
    contextMenuPosition,
    contextMenuItem,
    closeContextMenu,

    // 右键菜单操作
    rightClick,
    copyPath,
    copyFileName,
    copyFieldName,
    deleteFile
  };
}

import type { Ref } from "vue";
import { ref, computed, triggerRef, markRaw } from "vue";
import { v4 as uuidv4 } from "uuid";
import { invoke } from "@tauri-apps/api/core";
import { message } from "@/utils/message";
import { save } from "@tauri-apps/plugin-dialog";

export interface ResultTab {
  id: string;
  title: string;
  columns: { prop: string; label: string }[];
  data: any[];
  currentPage: number;
  pageSize: number;
  total: number;
}

interface ResultTabOptions {
  sqlQuery: Ref<string>;
  path: Ref<string>;
  varchar: Ref<boolean>;
  limit: Ref<boolean>;
}

export function useSqlTabManager(options: ResultTabOptions) {
  const tabs = ref<ResultTab[]>([]);
  const activeTabId = ref<string | null>(null);

  const activeTab = computed(() => {
    return tabs.value.find(t => t.id === activeTabId.value) || null;
  });

  const pagedTableData = computed(() => {
    if (!activeTab.value) return [];
    const start = (activeTab.value.currentPage - 1) * activeTab.value.pageSize;
    return activeTab.value.data.slice(start, start + activeTab.value.pageSize);
  });

  const tablePanelSize = computed(() => {
    return pagedTableData.value?.length > 0 ? "40%" : "35";
  });

  function createEmptyTab(): ResultTab {
    const count = tabs.value.length + 1;
    return {
      id: uuidv4(),
      title: `Query ${count}`,
      columns: [],
      data: [],
      currentPage: 1,
      pageSize: 50,
      total: 0
    };
  }

  async function executeQuery(
    tab: ResultTab,
    write: boolean,
    writeOptions?: {
      outputPath?: string;
      writeFormat?: string;
    }
  ): Promise<boolean> {
    const { sqlQuery, path, varchar, limit } = options;

    if (path.value === "") {
      message("No file selected. Add data first.", { type: "warning" });
      return false;
    }

    if (sqlQuery.value.trim() === "") {
      message("SQL script is empty.", { type: "warning" });
      return false;
    }

    try {
      const rawResult = await invoke("query", {
        path: path.value,
        sqlQuery: sqlQuery.value,
        varchar: varchar.value,
        limit: limit.value,
        write,
        writeFormat: writeOptions?.writeFormat || "csv",
        outputPath: writeOptions?.outputPath || ""
      });

      if (!write) {
        const result =
          typeof rawResult === "string" ? JSON.parse(rawResult) : rawResult;
        const jsonData = JSON.parse(result.data);
        const arrayData = Array.isArray(jsonData) ? jsonData : [jsonData];

        tab.columns = result.columns.map(key => ({ prop: key, label: key }));
        tab.data = markRaw(arrayData);
        tab.total = arrayData.length;
        tab.currentPage = 1;
        triggerRef(tabs);
      } else {
        message("Export done", { type: "success" });
      }
      return true;
    } catch (err) {
      message(err.toString(), { type: "error", duration: 5000 });
      return false;
    }
  }

  async function runCurrentTab() {
    if (!activeTab.value) {
      const newTab = createEmptyTab();
      tabs.value.push(newTab);
      activeTabId.value = newTab.id;
      await executeQuery(newTab, false);
    } else {
      activeTab.value.columns = [];
      activeTab.value.data = [];
      activeTab.value.total = 0;
      await executeQuery(activeTab.value, false);
    }
  }

  async function runNewTab() {
    const newTab = createEmptyTab();
    tabs.value.push(newTab);
    activeTabId.value = newTab.id;
    await executeQuery(newTab, false);
  }

  function sizeChange(newSize: number) {
    if (activeTab.value) {
      activeTab.value.pageSize = newSize;
      activeTab.value.currentPage = 1;
    }
  }

  function currentChange(newPage: number) {
    if (activeTab.value) {
      activeTab.value.currentPage = newPage;
    }
  }

  async function exportActiveTab() {
    if (!activeTab.value) return;

    const outputPath = await save({
      title: "Export Data",
      defaultPath: `export_${new Date().getTime()}`,
      filters: [
        { name: "CSV", extensions: ["csv"] },
        { name: "Excel", extensions: ["xlsx"] },
        { name: "Parquet", extensions: ["parquet"] },
        { name: "Json", extensions: ["json"] },
        { name: "NdJson", extensions: ["jsonl"] }
      ]
    });

    if (!outputPath) return;

    let writeFormat = "csv";
    if (outputPath.endsWith(".xlsx")) {
      writeFormat = "xlsx";
    } else if (outputPath.endsWith(".parquet")) {
      writeFormat = "parquet";
    } else if (outputPath.endsWith(".json")) {
      writeFormat = "json";
    } else if (outputPath.endsWith(".jsonl")) {
      writeFormat = "jsonl";
    }

    await executeQuery(activeTab.value, true, {
      outputPath,
      writeFormat
    });
  }

  function removeTab(targetId: string) {
    const index = tabs.value.findIndex(t => t.id === targetId);
    if (index !== -1) {
      tabs.value.splice(index, 1);
      if (activeTabId.value === targetId) {
        activeTabId.value = tabs.value.length > 0 ? tabs.value[0].id : null;
      }
    }
  }

  return {
    tabs,
    activeTabId,
    activeTab,
    pagedTableData,
    tablePanelSize,
    runCurrentTab,
    runNewTab,
    sizeChange,
    currentChange,
    exportActiveTab,
    removeTab
  };
}

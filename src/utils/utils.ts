import { TableColumnCtx } from "element-plus";
import { ref, onMounted, onBeforeUnmount, computed, Ref } from "vue";

export function shortFileName(path: string) {
  return path.split("\\").pop().split("/").pop();
}

export function useDynamicHeight(fixedHeight: number) {
  const windowHeight = ref(0);

  const dynamicHeight = computed(() => {
    return windowHeight.value - fixedHeight;
  });

  const updateWindowHeight = () => {
    windowHeight.value = window.innerHeight;
  };

  onMounted(() => {
    updateWindowHeight();
    window.addEventListener("resize", updateWindowHeight);
  });

  onBeforeUnmount(() => {
    window.removeEventListener("resize", updateWindowHeight);
  });

  return {
    dynamicHeight
  };
}

interface FileStatus {
  filename: string;
  status: string;
}

export const filterFileStatus = (
  value: string,
  row: FileStatus,
  column: TableColumnCtx<FileStatus>
) => {
  const property = column["property"];
  return row[property] === value;
};

export interface ListenEvent {
  filename: string;
  status?: string;
  currentRows?: string;
  totalRows?: string;
  message?: string;
  sheets?: string[];
  selectSheet?: string;
}

export const updateEvent = (
  fileSelect: Ref<ListenEvent[]>,
  filename: string,
  updater: (file: ListenEvent) => void
) => {
  const file = fileSelect.value.find(f => f.filename === filename);
  if (file) {
    updater(file);
  } else {
    const newFile: ListenEvent = {
      filename,
      status: "error",
      message: `File not found during update: "${filename}". Possibly failed to load.`
    };
    fileSelect.value.push(newFile);
    updater(newFile);
  }
};

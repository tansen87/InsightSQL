import { TableColumnCtx } from "element-plus";
import { ref, onMounted, onBeforeUnmount, computed } from "vue";

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

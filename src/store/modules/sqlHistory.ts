import { defineStore } from "pinia";

export const useSqlHistory = defineStore("sqlHistory", {
  state: () => ({
    path: "",
    dtypesByFile: {} as Record<string, Record<string, string>>
  }),
  persist: true
});

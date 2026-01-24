import { defineStore } from "pinia";

export const useQuoting = defineStore("quoting", {
  state: () => ({
    quoting: true
  }),
  actions: {
    toggleQuoting() {
      this.quoting = !this.quoting;
    },
    setQuoting(value) {
      this.quoting = !!value;
    }
  },
  persist: true
});

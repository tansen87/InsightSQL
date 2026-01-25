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

export const useFlexible = defineStore("flexible", {
  state: () => ({
    flexible: false
  }),
  actions: {
    setFlexible(value) {
      this.flexible = !!value;
    }
  },
  persist: true
});

export const useProgress = defineStore("progress", {
  state: () => ({
    progress: true
  }),
  actions: {
    setProgress(value) {
      this.progress = !!value;
    }
  },
  persist: true
});

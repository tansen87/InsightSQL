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

export const useSkiprows = defineStore("skiprows", {
  state: () => ({
    skiprows: 0
  }),
  actions: {
    setSkiprows(value: string) {
      this.skiprows = Math.max(0, parseInt(value) || 0);
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

export const useThreads = defineStore("threads", {
  state: () => ({
    threads: 0
  }),
  actions: {
    setThreads(value: string) {
      this.threads = Math.max(0, parseInt(value) || 0);
    }
  },
  persist: true
});

export const useDelimiter = defineStore("delimiter", {
  state: () => ({
    delimiter: "|"
  }),
  actions: {
    setDelimiter(value: string) {
      this.delimiter = value;
    }
  },
  persist: true
});

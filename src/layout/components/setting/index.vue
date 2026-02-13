<script setup lang="ts">
import { ref, unref, watch, onBeforeMount, reactive } from "vue";
import { debounce, useDark, useGlobal } from "@pureadmin/utils";
import { emitter } from "@/utils/mitt";
import { toggleTheme } from "@pureadmin/theme/dist/browser-utils";
import { useAppStoreHook } from "@/store/modules/app";
import { useDataThemeChange } from "@/layout/hooks/useDataThemeChange";
import {
  useDelimiter,
  useFlexible,
  useProgress,
  useQuoting,
  useSkiprows,
  useThreads
} from "@/store/modules/options";

const { $storage } = useGlobal<GlobalPropertiesApi>();
const { layoutTheme, dataThemeChange } = useDataThemeChange();
const dialog = ref(false);

emitter.on("openPanel", () => {
  dialog.value = true;
});

if (unref(layoutTheme)) {
  const layout = unref(layoutTheme).layout;
  const theme = unref(layoutTheme).theme;
  toggleTheme({ scopeName: `layout-theme- ${theme}` });
  setLayoutModel(layout);
}

const _settings = reactive({
  showModel: $storage.configure.showModel,
  multiTagsCache: $storage.configure.multiTagsCache
});

function toggleClass(flag: boolean, clsName: string, target?: HTMLElement) {
  const targetEl = target || document.body;
  let { className } = targetEl;
  className = className.replace(clsName, "").trim();
  targetEl.className = flag ? `${className} ${clsName}` : className;
}

const quotingStore = useQuoting();
const flexibleStore = useFlexible();
const skiprowsStore = useSkiprows();
const progressStore = useProgress();
const threadsStore = useThreads();
const delimiterStore = useDelimiter();

const mixRef = ref();
const verticalRef = ref();
const horizontalRef = ref();

function setFalse(Doms): any {
  Doms.forEach(v => {
    toggleClass(false, "is-select", unref(v));
  });
}

function setLayoutModel(layout: string) {
  layoutTheme.value.layout = layout;
  window.document.body.setAttribute("layout", layout);
  $storage.layout = {
    layout,
    theme: layoutTheme.value.theme,
    darkMode: $storage.layout?.darkMode,
    sidebarStatus: $storage.layout?.sidebarStatus,
    epThemeColor: $storage.layout?.epThemeColor
  };
  useAppStoreHook().setLayout(layout);
}

watch($storage, ({ layout }) => {
  switch (layout["layout"]) {
    case "vertical":
      toggleClass(true, "is-select", unref(verticalRef));
      debounce(setFalse([horizontalRef]), 50);
      debounce(setFalse([mixRef]), 50);
      break;
    case "horizontal":
      toggleClass(true, "is-select", unref(horizontalRef));
      debounce(setFalse([verticalRef]), 50);
      debounce(setFalse([mixRef]), 50);
      break;
    case "mix":
      toggleClass(true, "is-select", unref(mixRef));
      debounce(setFalse([verticalRef]), 50);
      debounce(setFalse([horizontalRef]), 50);
      break;
  }
});

onBeforeMount(() => {
  dataThemeChange();
});

const opts = ref("general");
const options = [
  { label: "General", value: "general" },
  { label: "Read/Write", value: "readwrite" }
];
const { isDark } = useDark();
</script>

<template>
  <el-dialog v-model="dialog" title="Setting" width="70%">
    <div class="mode-toggle w-[200px] ml-[0px] mb-[5px]">
      <span
        v-for="item in options"
        :key="item.value"
        class="mode-item"
        :class="{
          active: opts === item.value,
          'active-dark': isDark && opts === item.value
        }"
        @click="opts = item.value"
      >
        {{ item.label }}
      </span>
    </div>
    <el-scrollbar max-height="60vh">
      <div v-if="opts === 'general'" class="mt-1">
        <el-card class="setting-card">
          <div class="setting-item">
            <div class="setting-label">
              <span class="setting-title">skiprows</span>
              <span class="setting-desc"> Number of lines skipped </span>
            </div>
            <el-input-number
              v-model="skiprowsStore.skiprows"
              :min="0"
              size="small"
            />
          </div>
        </el-card>
        <el-card class="setting-card">
          <div class="setting-item">
            <div class="setting-label">
              <span class="setting-title">progress</span>
              <span class="setting-desc">
                When set to false, no progress bar
              </span>
            </div>
            <el-switch
              :model-value="progressStore.progress"
              @change="progressStore.setProgress"
              inline-prompt
              class="setting-switch"
              active-text="true"
              inactive-text="false"
            />
          </div>
        </el-card>
        <el-card class="setting-card">
          <div class="setting-item">
            <div class="setting-label">
              <span class="setting-title">threads</span>
              <span class="setting-desc"> Number of threads used </span>
            </div>
            <el-input-number
              v-model="threadsStore.threads"
              :min="0"
              size="small"
            />
          </div>
        </el-card>
      </div>

      <div v-if="opts === 'readwrite'" class="mt-1">
        <el-card class="setting-card">
          <div class="setting-item">
            <div class="setting-label">
              <span class="setting-title">quoting</span>
              <span class="setting-desc">
                When set to false, ignore all double quotes
              </span>
            </div>
            <el-switch
              :model-value="quotingStore.quoting"
              @change="quotingStore.setQuoting"
              inline-prompt
              class="setting-switch"
              active-text="true"
              inactive-text="false"
            />
          </div>
        </el-card>
        <el-card class="setting-card">
          <div class="setting-item">
            <div class="setting-label">
              <span class="setting-title">flexible</span>
              <span class="setting-desc">
                When set to false, enable column count check
              </span>
            </div>
            <el-switch
              :model-value="flexibleStore.flexible"
              @change="flexibleStore.setFlexible"
              inline-prompt
              class="setting-switch"
              active-text="true"
              inactive-text="false"
            />
          </div>
        </el-card>
        <el-card class="setting-card">
          <div class="setting-item">
            <div class="setting-label">
              <span class="setting-title">delimiter</span>
              <span class="setting-desc"> Write the delimiter for CSV </span>
            </div>
            <el-select style="width: 50px" v-model="delimiterStore.delimiter">
              <el-option label="|" value="|" />
              <el-option label="," value="," />
              <el-option label=";" value=";" />
              <el-option label="\t" value="\t" />
            </el-select>
          </div>
        </el-card>
      </div>
    </el-scrollbar>
  </el-dialog>
</template>

<style lang="scss" scoped>
.setting-card {
  border-radius: 8px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
}
.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
}
.setting-label {
  display: flex;
  flex-direction: column;
}
.setting-title {
  font-weight: bold;
  font-size: 18px;
}
.setting-desc {
  font-size: 12px;
}
.setting-switch {
  --el-switch-on-color: #13ce66;
  --el-switch-off-color: #000;
}

:deep(.el-input-number) {
  width: 80px !important;
}
</style>

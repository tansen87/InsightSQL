<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  Handle,
  Position,
  useNodeId,
  useNode,
  useVueFlow
} from "@vue-flow/core";
import { CloseBold } from "@element-plus/icons-vue";
import { useHeaders, useStr } from "@/store/modules/flow";

const mode = ref("len");
const [comparand, replacement, columns] = [ref(""), ref(""), ref("")];
const nodeId = useNodeId();
const headerStore = useHeaders();
const strStore = useStr();
const node = useNode();
const { removeNodes } = useVueFlow();
const strData = computed(() => {
  return {
    op: "str",
    mode: mode.value,
    column: columns.value,
    comparand: comparand.value,
    replacement: replacement.value
  };
});

watch(
  strData,
  newData => {
    if (nodeId && (newData.mode || newData.column)) {
      strStore.addString({
        id: nodeId,
        ...newData
      });
    }
  },
  { deep: true, immediate: true }
);

function deleteBtn() {
  removeNodes(node.id);
}
</script>

<template>
  <div class="page-container">
    <div class="node-container">
      <Handle
        type="target"
        :position="Position.Left"
        id="input"
        class="handle-style"
      />
      <div style="text-align: center; width: 100%; padding: 5px">
        <el-button
          circle
          link
          @click="deleteBtn"
          :icon="CloseBold"
          size="small"
          style="position: absolute; top: -2.5px; right: -2.5px; z-index: 10"
        />
        <span style="display: block; font-weight: bold; margin-bottom: 6px">
          Str
        </span>
        <el-select
          v-model="columns"
          filterable
          placeholder="Select column"
          style="margin-bottom: 6px"
        >
          <el-option
            v-for="item in headerStore.headers"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
        <el-select v-model="mode" filterable style="margin-bottom: 6px">
          <el-option label="copy" value="copy" />
          <el-option label="abs" value="abs" />
          <el-option label="neg" value="neg" />
          <el-option label="reverse" value="reverse" />
          <el-option label="strip" value="strip" />
          <el-option label="squeeze" value="squeeze" />
          <el-option label="round" value="round" />
          <el-option label="len" value="len" />
          <el-option label="replace" value="replace" />
          <el-option label="RegexReplace" value="regex_replace" />
          <el-option label="trim" value="trim" />
          <el-option label="ltrim" value="ltrim" />
          <el-option label="rtrim" value="rtrim" />
          <el-option label="upper" value="upper" />
          <el-option label="lower" value="lower" />
          <el-option label="pinyin" value="pinyin" />
          <el-option label="fill" value="fill" />
          <el-option label="ForwardFill" value="f_fill" />
        </el-select>
        <el-input
          v-if="mode === 'replace' || mode === 'regex_replace'"
          v-model="comparand"
          style="margin-bottom: 6px"
          placeholder="comparand"
        />
        <el-input
          v-if="mode === 'replace' || mode === 'regex_replace'"
          v-model="replacement"
          placeholder="replacement"
        />
        <el-input
          v-if="mode === 'fill'"
          v-model="replacement"
          placeholder="fill value"
        />
        <el-select v-if="mode === 'pinyin'" v-model="replacement">
          <el-option label="upper" value="upper" />
          <el-option label="lower" value="lower" />
        </el-select>
      </div>
      <Handle
        type="source"
        :position="Position.Right"
        id="output"
        class="handle-style"
      />
    </div>
  </div>
</template>

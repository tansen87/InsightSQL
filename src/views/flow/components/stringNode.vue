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
        <el-tooltip content="Delete" effect="light">
          <el-button
            class="del-btn"
            circle
            link
            @click="deleteBtn"
            :icon="CloseBold"
            size="small"
          />
        </el-tooltip>
        <span style="display: block; font-weight: bold; margin-bottom: 10px">
          Str
        </span>
        <el-select
          v-model="columns"
          filterable
          placeholder="Select column"
          style="width: 100%; margin-bottom: 10px"
        >
          <el-option
            v-for="item in headerStore.headers"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
        <el-tooltip content="Str mode" effect="light">
          <el-select
            v-model="mode"
            filterable
            style="width: 100%; margin-bottom: 10px"
          >
            <el-option label="copy" value="copy" />
            <el-option label="abs" value="abs" />
            <el-option label="neg" value="neg" />
            <el-option label="reverse" value="reverse" />
            <el-option label="strip" value="strip" />
            <el-option label="squeeze" value="squeeze" />
            <el-option label="round" value="round" />
            <el-option label="len" value="len" />
            <el-option label="replace" value="replace" />
            <el-option label="ltrim" value="ltrim" />
            <el-option label="rtrim" value="rtrim" />
            <el-option label="trim" value="trim" />
            <el-option label="upper" value="upper" />
            <el-option label="lower" value="lower" />
            <el-option label="pinyin" value="pinyin" />
            <el-option label="fill" value="fill" />
            <el-option label="forwardFill" value="f_fill" />
          </el-select>
        </el-tooltip>
        <el-tooltip content="replace - from" effect="light">
          <el-input
            v-if="mode === 'replace'"
            v-model="comparand"
            style="width: 100%; margin-bottom: 10px"
            placeholder="replace - from"
          />
        </el-tooltip>
        <el-tooltip content="replace - to" effect="light">
          <el-input
            v-if="mode === 'replace'"
            v-model="replacement"
            style="width: 100%"
            placeholder="replace - to"
          />
        </el-tooltip>
        <el-input
          v-if="mode === 'fill'"
          v-model="replacement"
          style="width: 100%"
          placeholder="fill value"
        />
        <el-tooltip
          v-if="mode === 'pinyin'"
          content="pinyin mode"
          effect="light"
        >
          <el-select v-model="replacement" style="width: 100%">
            <el-option label="upper" value="upper" />
            <el-option label="lower" value="lower" />
          </el-select>
        </el-tooltip>
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

<style scoped>
.del-btn {
  position: absolute;
  top: -2.5px;
  right: -2.5px;
  z-index: 10;
}
</style>

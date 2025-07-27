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
import { useHeaders, useSlice } from "@/store/modules/flow";

const mode = ref("left");
const [columns, offset, length] = [ref(""), ref(""), ref("")];
const nodeId = useNodeId();
const node = useNode();
const { removeNodes } = useVueFlow();
const headerStore = useHeaders();
const sliceStore = useSlice();
const sliceData = computed(() => {
  return {
    op: "slice",
    mode: mode.value,
    column: columns.value,
    offset: offset.value,
    length: length.value
  };
});

watch(
  sliceData,
  newData => {
    if (
      nodeId &&
      (newData.mode || newData.column || newData.offset || newData.length)
    ) {
      sliceStore.addSlice({
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
      <div style="text-align: center; padding: 5px">
        <el-button
          circle
          link
          @click="deleteBtn"
          :icon="CloseBold"
          size="small"
          style="position: absolute; top: -2.5px; right: -2.5px; z-index: 10"
        />
        <span style="display: block; font-weight: bold; margin-bottom: 6px">
          Slice
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
          <el-option label="left" value="left" />
          <el-option label="right" value="right" />
          <el-option label="slice" value="slice" />
        </el-select>
        <el-input
          v-if="mode === 'slice'"
          v-model="offset"
          style="margin-bottom: 6px"
          placeholder="start index"
        />
        <el-input v-model="length" placeholder="length" />
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

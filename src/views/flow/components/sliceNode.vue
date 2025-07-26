<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Handle, Position, useNodeId } from "@vue-flow/core";
import { useHeaders, useSlice } from "@/store/modules/flow";

const columns = ref("");
const mode = ref("left");
const [offset, length] = [ref("1"), ref("1")];
const nodeId = useNodeId();
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
        <span style="display: block; font-weight: bold; margin-bottom: 10px">
          Slice
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
        <el-tooltip content="Slice mode" effect="light">
          <el-select
            v-model="mode"
            filterable
            style="width: 100%; margin-bottom: 10px"
          >
            <el-option label="left" value="left" />
            <el-option label="right" value="right" />
            <el-option label="slice" value="slice" />
          </el-select>
        </el-tooltip>
        <div v-if="mode === 'slice'">
          <el-tooltip content="start index" effect="light">
            <el-input
              v-model="offset"
              style="width: 100%; margin-bottom: 10px"
              placeholder="start index"
            />
          </el-tooltip>
          <el-tooltip content="length" effect="light">
            <el-input
              v-model="length"
              style="width: 100%"
              placeholder="length"
            />
          </el-tooltip>
        </div>
        <div v-else>
          <el-tooltip content="length" effect="light">
            <el-input
              v-model="length"
              style="width: 100%"
              placeholder="length"
            />
          </el-tooltip>
        </div>
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

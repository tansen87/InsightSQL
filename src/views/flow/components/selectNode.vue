<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Handle, Position, useNodeId } from "@vue-flow/core";
import { useHeaders, useSelect } from "@/store/modules/flow";

const columns = ref([]);
const nodeId = useNodeId();
const headerStore = useHeaders();
const selectStore = useSelect();
const selCols = computed(() => columns.value.join("|"));
const selectData = computed(() => {
  return {
    op: "select",
    column: selCols.value
  };
});

watch(
  selectData,
  newData => {
    if (nodeId && newData.column) {
      selectStore.addSelect({
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
          Select
        </span>
        <el-select
          v-model="columns"
          multiple
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

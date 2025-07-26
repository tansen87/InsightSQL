<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Handle, Position, useNodeId } from "@vue-flow/core";
import { useHeaders, useFilter } from "@/store/modules/flow";

const [columns, condition] = [ref(""), ref("")];
const mode = ref("equal");
const nodeId = useNodeId();
const headerStore = useHeaders();
const filterStore = useFilter();
const filterData = computed(() => {
  return {
    op: "filter",
    mode: mode.value,
    column: columns.value,
    value: condition.value
  };
});

watch(
  filterData,
  newData => {
    if (nodeId && (newData.mode || newData.column || newData.value)) {
      filterStore.addFilter({
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
          Filter
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
        <el-tooltip content="Filter mode" effect="light">
          <el-select
            v-model="mode"
            filterable
            style="width: 100%; margin-bottom: 10px"
          >
            <el-option label="Equal" value="equal" />
            <el-option label="NotEqual" value="notequal" />
            <el-option label="Contains" value="contains" />
            <el-option label="NotContains" value="notcontains" />
            <el-option label="StartsWith" value="startswith" />
            <el-option label="NotStartsWtih" value="notstartswith" />
            <el-option label="EndsWith" value="endswith" />
            <el-option label="NotEndsWith" value="notendswith" />
            <el-option label="IsNull" value="isnull" />
            <el-option label="IsNotNull" value="isnotnull" />
            <el-option label="gt (>)" value="gt" />
            <el-option label="ge (≥)" value="ge" />
            <el-option label="lt (<)" value="lt" />
            <el-option label="le (≤)" value="le" />
            <el-option label="Between" value="between" />
          </el-select>
        </el-tooltip>
        <el-input
          v-model="condition"
          placeholder="Filter condition..."
          style="width: 100%"
        />
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

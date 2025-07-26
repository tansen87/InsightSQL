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
import { useHeaders, useFilter } from "@/store/modules/flow";

const [columns, condition] = [ref(""), ref("")];
const mode = ref("equal");
const node = useNode();
const { removeNodes } = useVueFlow();
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
        <span style="display: block; font-weight: bold"> Filter </span>
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
          <el-option label="Equal" value="equal" />
          <el-option label="NotEqual" value="not_equal" />
          <el-option label="Contains" value="contains" />
          <el-option label="NotContains" value="not_contains" />
          <el-option label="StartsWith" value="starts_with" />
          <el-option label="NotStartsWith" value="not_starts_with" />
          <el-option label="EndsWith" value="ends_with" />
          <el-option label="NotEndsWith" value="not_ends_with" />
          <el-option label="IsNull" value="is_null" />
          <el-option label="IsNotNull" value="is_not_null" />
          <el-option label="gt(>)" value="gt" />
          <el-option label="ge(≥)" value="ge" />
          <el-option label="lt(<)" value="lt" />
          <el-option label="le(≤)" value="le" />
          <el-option label="Between" value="between" />
        </el-select>
        <el-input
          v-if="mode !== 'is_null' && mode !== 'is_not_null'"
          v-model="condition"
          placeholder="Filter conditions"
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

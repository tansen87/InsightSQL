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
import { useHeaders, useRename } from "@/store/modules/flow";

const [old_col, new_col] = [ref(""), ref("")];
const node = useNode();
const { removeNodes } = useVueFlow();
const nodeId = useNodeId();
const headerStore = useHeaders();
const renameStore = useRename();
const renameData = computed(() => {
  return {
    op: "rename",
    column: old_col.value,
    value: new_col.value
  };
});

watch(
  renameData,
  newData => {
    if (nodeId && (newData.column || newData.value)) {
      renameStore.addRename({
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
      <div class="text-center p-[5px]">
        <el-button
          circle
          link
          @click="deleteBtn"
          :icon="CloseBold"
          size="small"
          class="absolute top-[-2.5px] right-[-2.5px] z-10"
        />
        <span class="block font-bold">Rename</span>

        <el-select
          v-model="old_col"
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

        <el-input
          v-model="new_col"
          placeholder="New name"
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

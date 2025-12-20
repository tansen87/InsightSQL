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
      strStore.addStr({
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
        <span class="block font-bold"> Str </span>
        <el-select
          v-if="!new Set(['cat', 'calcconv']).has(mode)"
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
          <el-option label="Cat" value="cat" />
          <el-option label="CalcConv" value="calcconv" />
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
          <el-option label="left" value="left" />
          <el-option label="right" value="right" />
          <el-option label="slice" value="slice" />
          <el-option label="split" value="split" />
          <el-option label="PadLeft" value="pad_left" />
          <el-option label="PadRight" value="pad_right" />
          <el-option label="PadBoth" value="pad_both" />
        </el-select>
        <template v-if="['replace', 'regex_replace'].includes(mode)">
          <el-input
            v-model="comparand"
            style="margin-bottom: 6px"
            placeholder="comparand"
          />
          <el-input v-model="replacement" placeholder="replacement" />
        </template>
        <template v-if="['split'].includes(mode)">
          <el-input
            v-model="comparand"
            style="margin-bottom: 6px"
            placeholder="delimiter"
          />
          <el-input v-model="replacement" placeholder="n" />
        </template>
        <el-input
          v-if="['cat', 'calcconv'].includes(mode)"
          v-model="comparand"
          placeholder="{col1}-{col2}+{col3}"
        />
        <el-input
          v-if="['slice'].includes(mode)"
          v-model="comparand"
          style="margin-bottom: 6px"
          placeholder="start index"
        />
        <el-input
          v-if="
            [
              'left',
              'right',
              'slice',
              'pad_left',
              'pad_right',
              'pad_both'
            ].includes(mode)
          "
          v-model="replacement"
          style="margin-bottom: 6px"
          placeholder="length"
        />
        <el-input
          v-if="['pad_left', 'pad_right', 'pad_both'].includes(mode)"
          v-model="comparand"
          placeholder="fill char"
        />
        <el-input
          v-if="['fill'].includes(mode)"
          v-model="replacement"
          placeholder="fill value"
        />
        <el-select v-if="['pinyin'].includes(mode)" v-model="replacement">
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

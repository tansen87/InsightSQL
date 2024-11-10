<script setup lang="ts">
import { useRouter } from "vue-router";
import { ref, computed } from "vue";

const router = useRouter();
const commands = ref([
  {
    title: "Modify",
    description: "Batch modify filenames.",
    route: "/operation/components/modify"
  },
  {
    title: "Traverse",
    description: "Traverse the directory to obtain filenames.",
    route: "/operation/components/traverse"
  }
]);
const searchText = ref("");
const filteredCommands = computed(() => {
  return commands.value.filter(command =>
    command.title.toLowerCase().includes(searchText.value.toLowerCase())
  );
});

function toCommands(route) {
  router.push(route);
}
</script>

<template>
  <div class="page-container">
    <el-input
      placeholder="Search for command..."
      v-model="searchText"
      class="search-input"
    />
    <el-row :gutter="20">
      <el-col
        :xs="24"
        :sm="24"
        :md="8"
        :lg="8"
        :xl="8"
        v-for="(item, index) in filteredCommands"
        :key="index"
      >
        <el-card
          class="box-card"
          shadow="hover"
          @click="toCommands(item.route)"
        >
          <span class="title-color">{{ item.title }}</span>
          <p class="description-color">{{ item.description }}</p>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style lang="scss" scoped>
// .page-container {
//   display: flex;
//   flex-direction: column;
//   height: 100%;
// }
.search-input {
  position: sticky;
  top: 0;
  z-index: 1000;
  padding: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  margin-bottom: 16px;
}
.box-card {
  margin-bottom: 16px;
}
.title-color {
  font-weight: bold;
  font-size: 30px;
}
.description-color {
  font-size: 15px;
}
</style>

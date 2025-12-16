<script setup lang="ts">
import { useRouter } from "vue-router";
import { ref, computed } from "vue";
import { storeToRefs } from "pinia";
import { Icon } from "@iconify/vue";
import { useCommandStore } from "@/store/modules/commands";

const router = useRouter();
const commandStore = useCommandStore();
const { commands } = storeToRefs(commandStore);
const searchText = ref("");
const filteredCommands = computed(() => {
  return commands.value.filter(command =>
    command.title.toLowerCase().includes(searchText.value.toLowerCase())
  );
});

function navigateToCommand(route) {
  router.push(route);
}
</script>

<template>
  <div>
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
          @click="navigateToCommand(item.route)"
        >
          <div class="card-content">
            <div class="header-row">
              <Icon :icon="item.icon" width="20" height="20" />
              <span class="title-color">{{ item.title }}</span>
            </div>
            <p class="description-color">{{ item.description }}</p>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style lang="scss" scoped>
.search-input {
  position: sticky;
  top: 0;
  z-index: 500;
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
.card-content {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.header-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>

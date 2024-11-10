<script setup lang="ts">
import { useRouter } from "vue-router";
import { ref, computed } from "vue";

const router = useRouter();
const commands = ref([
  {
    title: "Cat",
    description: "Cat CSV and Excel files.",
    route: "/command/components/cat"
  },
  {
    title: "Excel to csv",
    description: "Exports Excel to csv files.",
    route: "/command/components/excel"
  },
  {
    title: "Count",
    description: "Count the rows of CSV files.",
    route: "/command/components/count"
  },
  {
    title: "Csv to xlsx",
    description: "Exports csv to xlsx files.",
    route: "/command/components/csv"
  },
  {
    title: "Rename",
    description: "Rename the columns of a CSV.",
    route: "/command/components/rename"
  },
  {
    title: "Select",
    description: "Select, re-order columns.",
    route: "/command/components/select"
  },
  {
    title: "Search",
    description: "Select fields matching rows.",
    route: "/command/components/search"
  },
  {
    title: "Fill",
    description: "Fill empty fields in selected columns of a CSV.",
    route: "/command/components/fill"
  },
  {
    title: "Split",
    description: "Split one CSV file into many CSV files.",
    route: "/command/components/split"
  },
  {
    title: "Access to csv",
    description: "Convert Access Database to CSV.",
    route: "/command/components/access"
  },
  {
    title: "Dbf to csv",
    description: "Convert dbf file to CSV.",
    route: "/command/components/dbf"
  },
  {
    title: "Drop csv headers",
    description: "Drop headers from CSV.",
    route: "/command/components/behead"
  },
  {
    title: "Offset",
    description: "net amount.",
    route: "/command/components/offset"
  },
  {
    title: "Add index",
    description: "Add an index for a CSV.",
    route: "/command/components/addIndex"
  },
  {
    title: "Chinese to Pinyin",
    description: "Convert Chinese to Pinyin for specific column in CSV.",
    route: "/command/components/pinyin"
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

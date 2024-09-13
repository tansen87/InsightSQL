<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();

function handleCardClick() {
  router.push("/command/cat");
}
</script>

<template>
  <div class="app-container">
    <el-row>
      <el-col>
        <el-card
          class="box-card"
          style="width: 25%"
          shadow="hover"
          @click="handleCardClick"
        >
          <span class="titleColor">cat</span>
          <p class="descriptionColor">
            Concatenate CSV and Excel files by column.
          </p>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style lang="scss" scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.el-form {
  flex: 1;
}
.el-table {
  flex: 1;
}
.box-card {
  margin-bottom: 20px;

  :deep(.el-card__header) {
    padding-bottom: 0;
    border: none;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-weight: 600;
  }

  .card-content {
    ::deep(.el-progress-bar__outer) {
      height: 17px !important;
    }

    .numerical-value {
      display: flex;
      align-items: flex-end;
      justify-content: space-between;
      margin-bottom: 10px;

      .number {
        color: var(--text-color-primary);
        font-size: var(--font-size-extra-large);
        font-weight: 600;
      }
    }
  }
}

.titleColor {
  font-weight: bold;
  font-size: 30px;
}
.descriptionColor {
  font-size: 15px;
}
</style>

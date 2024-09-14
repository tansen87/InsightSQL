<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();

function toCat() {
  router.push("/command/cat");
}
function toExcelToCsv() {
  router.push("/command/excel");
}
function toCsvToXlsx() {
  router.push("/command/csv");
}
function toCount() {
  router.push("/command/count");
}
function toRename() {
  router.push("/command/rename");
}
</script>

<template>
  <div class="app-container">
    <el-row :gutter="20">
      <el-col :xs="24" :sm="24" :md="12" :lg="12" :xl="12">
        <el-card class="box-card" shadow="hover" @click="toCat">
          <span class="titleColor">Cat</span>
          <p class="descriptionColor">
            Concatenate CSV and Excel files by column.
          </p>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="24" :md="12" :lg="12" :xl="12">
        <el-card class="box-card" shadow="hover" @click="toExcelToCsv">
          <span class="titleColor">Excel to csv</span>
          <p class="descriptionColor">Exports Excel to csv files.</p>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="24" :md="12" :lg="12" :xl="12">
        <el-card class="box-card" shadow="hover" @click="toCsvToXlsx">
          <span class="titleColor">Csv to xlsx</span>
          <p class="descriptionColor">Exports csv to xlsx files.</p>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="24" :md="12" :lg="12" :xl="12">
        <el-card class="box-card" shadow="hover" @click="toCount">
          <span class="titleColor">Count</span>
          <p class="descriptionColor">Count the rows of CSV files.</p>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="24" :md="12" :lg="12" :xl="12">
        <el-card class="box-card" shadow="hover" @click="toRename">
          <span class="titleColor">Rename</span>
          <p class="descriptionColor">Rename the columns of a CSV.</p>
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

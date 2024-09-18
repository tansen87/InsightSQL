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
function toSelect() {
  router.push("/command/select");
}
function toSearch() {
  router.push("/command/search");
}
function toFill() {
  router.push("/command/fill");
}
</script>

<template>
  <div class="page-container">
    <el-row :gutter="20">
      <el-col :xs="24" :sm="24" :md="8" :lg="8" :xl="8">
        <el-card class="box-card" shadow="hover" @click="toCat">
          <span class="title-color">Cat</span>
          <p class="description-color">Concatenate CSV and Excel files.</p>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="24" :md="8" :lg="8" :xl="8">
        <el-card class="box-card" shadow="hover" @click="toExcelToCsv">
          <span class="title-color">Excel to csv</span>
          <p class="description-color">Exports Excel to csv files.</p>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="24" :md="8" :lg="8" :xl="8">
        <el-card class="box-card" shadow="hover" @click="toCount">
          <span class="title-color">Count</span>
          <p class="description-color">Count the rows of CSV files.</p>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="24" :md="8" :lg="8" :xl="8">
        <el-card class="box-card" shadow="hover" @click="toCsvToXlsx">
          <span class="title-color">Csv to xlsx</span>
          <p class="description-color">Exports csv to xlsx files.</p>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="24" :md="8" :lg="8" :xl="8">
        <el-card class="box-card" shadow="hover" @click="toRename">
          <span class="title-color">Rename</span>
          <p class="description-color">Rename the columns of a CSV.</p>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="24" :md="8" :lg="8" :xl="8">
        <el-card class="box-card" shadow="hover" @click="toSelect">
          <span class="title-color">Select</span>
          <p class="description-color">Select, re-order or drop columns.</p>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="24" :md="8" :lg="8" :xl="8">
        <el-card class="box-card" shadow="hover" @click="toSearch">
          <span class="title-color">Search</span>
          <p class="description-color">
            Select fields and shows only matching rows.
          </p>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="24" :md="8" :lg="8" :xl="8">
        <el-card class="box-card" shadow="hover" @click="toFill">
          <span class="title-color">Fill</span>
          <p class="description-color">
            Fill empty fields in selected columns of a CSV.
          </p>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style lang="scss" scoped>
.page-container {
  display: flex;
  flex-direction: column;
  height: 100%;
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

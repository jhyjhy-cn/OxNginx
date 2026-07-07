<template>
  <div class="forbidden">
    <el-result icon="warning" :title="t('forbidden.title')" :sub-title="t('forbidden.subtitle')">
      <template #extra>
        <el-button type="primary" @click="$router.push('/dashboard')">
          {{ t('forbidden.back') }}
        </el-button>
      </template>
    </el-result>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
</script>

<style scoped>
.forbidden {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 70vh;
}
</style>
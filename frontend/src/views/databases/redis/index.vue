<template>
  <div class="databases-page">
    <el-card>
      <div class="placeholder">
        <el-icon size="48"><Coin /></el-icon>
        <p>{{ t('dbm.redisPlaceholder') }}</p>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { Coin } from '@element-plus/icons-vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<style scoped>
.databases-page {
  padding: 0;
}
.placeholder {
  text-align: center;
  padding: 80px 20px;
  color: var(--el-text-color-secondary);
}
.placeholder .el-icon {
  font-size: 48px;
  margin-bottom: 16px;
}
.placeholder p {
  margin: 0;
  font-size: 14px;
}
</style>

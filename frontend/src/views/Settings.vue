<template>
  <div class="settings">
    <el-card>
      <template #header>
        <span>系统设置</span>
      </template>

      <el-form label-width="120px">
        <el-form-item label="Nginx路径">
          <el-input value="/usr/sbin/nginx" disabled />
        </el-form-item>
        <el-form-item label="配置文件路径">
          <el-input value="/etc/nginx/nginx.conf" disabled />
        </el-form-item>
        <el-form-item label="站点配置目录">
          <el-input value="/etc/nginx/sites-enabled" disabled />
        </el-form-item>
        <el-form-item label="acme.sh路径">
          <el-input value="/root/.acme.sh/acme.sh" disabled />
        </el-form-item>
      </el-form>
    </el-card>

    <el-card style="margin-top: 20px">
      <template #header>
        <span>系统信息</span>
      </template>

      <el-descriptions :column="2" border>
        <el-descriptions-item label="系统版本">{{ systemInfo.os }}</el-descriptions-item>
        <el-descriptions-item label="系统架构">{{ systemInfo.arch }}</el-descriptions-item>
        <el-descriptions-item label="Rust版本">Stable</el-descriptions-item>
        <el-descriptions-item label="OxNginx版本">1.0.0</el-descriptions-item>
      </el-descriptions>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

const systemInfo = ref({
  os: 'Linux',
  arch: 'x86_64',
})

onMounted(() => {
  // 从API获取系统信息
})
</script>

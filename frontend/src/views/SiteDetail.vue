<template>
  <div class="site-detail">
    <el-card v-loading="loading">
      <template #header>
        <div class="card-header">
          <span>站点详情</span>
          <el-button @click="$router.back()">返回</el-button>
        </div>
      </template>

      <template v-if="site">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="名称">{{ site.name }}</el-descriptions-item>
          <el-descriptions-item label="域名">{{ site.server_name }}</el-descriptions-item>
          <el-descriptions-item label="端口">{{ site.listen }}</el-descriptions-item>
          <el-descriptions-item label="SSL">
            <el-tag :type="site.ssl ? 'success' : 'info'" size="small">
              {{ site.ssl ? '是' : '否' }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="反向代理">{{ site.proxy_pass || '-' }}</el-descriptions-item>
          <el-descriptions-item label="根目录">{{ site.root_path || '-' }}</el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="site.status === 'enabled' ? 'success' : 'danger'" size="small">
              {{ site.status === 'enabled' ? '启用' : '禁用' }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="创建时间">{{ site.created_at }}</el-descriptions-item>
        </el-descriptions>

        <div style="margin-top: 20px">
          <h3>生成的配置</h3>
          <el-input
            v-model="configContent"
            type="textarea"
            :rows="15"
            readonly
            style="margin-top: 10px"
          />
        </div>
      </template>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import api from '@/api'

const route = useRoute()
const loading = ref(false)
const site = ref<any>(null)
const configContent = ref('')

onMounted(() => {
  fetchSite()
})

async function fetchSite() {
  loading.value = true
  try {
    const response = await api.get(`/api/sites/${route.params.id}`)
    if (response.data.code === 0) {
      site.value = response.data.data
      generateConfig()
    }
  } catch (error) {
    console.error('获取站点详情失败:', error)
  } finally {
    loading.value = false
  }
}

function generateConfig() {
  if (!site.value) return

  let config = 'server {\n'

  if (site.value.ssl) {
    config += `    listen ${site.value.listen} ssl;\n`
    config += '    listen [::]:443 ssl;\n'
  } else {
    config += `    listen ${site.value.listen};\n`
    config += `    listen [::]:${site.value.listen};\n`
  }

  config += `    server_name ${site.value.server_name};\n`

  if (site.value.ssl && site.value.certificate_path) {
    config += `    ssl_certificate ${site.value.certificate_path};\n`
    config += `    ssl_certificate_key ${site.value.key_path};\n`
    config += '    ssl_protocols TLSv1.2 TLSv1.3;\n'
  }

  if (site.value.proxy_pass) {
    config += '\n    location / {\n'
    config += `        proxy_pass ${site.value.proxy_pass};\n`
    config += '        proxy_set_header Host $host;\n'
    config += '        proxy_set_header X-Real-IP $remote_addr;\n'
    config += '        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n'
    config += '    }\n'
  } else if (site.value.root_path) {
    config += `\n    root ${site.value.root_path};\n`
    config += '    index index.html;\n'
    config += '\n    location / {\n'
    config += '        try_files $uri $uri/ /index.html;\n'
    config += '    }\n'
  }

  config += '}\n'
  configContent.value = config
}
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>

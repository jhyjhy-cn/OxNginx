<template>
  <OnDialog v-model="dialogVisible" :title="isEdit ? '编辑反向代理' : '添加反向代理'" width="500px" :maximizable="false">
    <el-form ref="formRef" :model="form" :rules="rules" label-width="80px">
      <el-form-item label="名称" prop="name">
        <el-input v-model="form.name" placeholder="API服务" />
      </el-form-item>
      <el-form-item label="代理目录" prop="proxy_dir">
        <el-input v-model="form.proxy_dir" placeholder="/api/" />
      </el-form-item>
      <el-form-item label="目标URL" prop="target_url">
        <el-input v-model="form.target_url" placeholder="http://127.0.0.1:3000" />
      </el-form-item>
      <el-form-item label="缓存">
        <el-switch v-model="form.cache" active-text="已开启" inactive-text="已关闭" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="dialogVisible = false">取消</el-button>
      <el-button type="primary" :loading="submitting" @click="submit">确定</el-button>
    </template>
  </OnDialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import type { FormInstance } from 'element-plus'
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'
import type { ReverseProxy } from './types'

const props = defineProps<{
  visible: boolean
  siteId: number
  proxy?: ReverseProxy | null
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  saved: []
}>()

const dialogVisible = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
})

const isEdit = computed(() => !!props.proxy)
const formRef = ref<FormInstance>()
const submitting = ref(false)
const form = reactive({
  name: '',
  proxy_dir: '/',
  target_url: '',
  cache: false,
})

const rules = {
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }],
  proxy_dir: [{ required: true, message: '请输入代理目录', trigger: 'blur' }],
  target_url: [{ required: true, message: '请输入目标URL', trigger: 'blur' }],
}

watch(
  () => props.visible,
  (val) => {
    if (val && props.proxy) {
      form.name = props.proxy.name
      form.proxy_dir = props.proxy.proxy_dir
      form.target_url = props.proxy.target_url
      form.cache = props.proxy.cache === 1
    } else if (val) {
      form.name = ''
      form.proxy_dir = '/'
      form.target_url = ''
      form.cache = false
    }
  }
)

async function submit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  submitting.value = true
  try {
    const data = {
      name: form.name,
      proxy_dir: form.proxy_dir,
      target_url: form.target_url,
      cache: form.cache ? 1 : 0,
    }
    if (isEdit.value && props.proxy) {
      await api.put(`/api/proxies/${props.proxy.id}`, data)
    } else {
      await api.post(`/api/sites/${props.siteId}/proxies`, data)
    }
    ElMessage.success('保存成功')
    dialogVisible.value = false
    emit('saved')
  } catch (e: any) {
    ElMessage.error(e.response?.data?.message || '操作失败')
  } finally {
    submitting.value = false
  }
}
</script>

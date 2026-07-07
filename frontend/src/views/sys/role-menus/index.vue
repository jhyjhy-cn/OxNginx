<template>
  <div class="rbac-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>菜单权限 - {{ role?.name || id }}</span>
          <el-button type="primary" @click="save" :loading="saving">{{ $t('common.save') }}</el-button>
        </div>
      </template>

      <el-tree
        ref="treeRef"
        :data="tree"
        show-checkbox
        node-key="id"
        :default-checked-keys="checked"
        :props="{ label: 'title', children: 'children' }"
      />
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import api from '@/api'

const route = useRoute()
const id = Number(route.params.id)
const role = ref<any>(null)
const tree = ref<any[]>([])
const checked = ref<number[]>([])
const treeRef = ref()
const saving = ref(false)

onMounted(async () => {
  const { data: rd } = await api.get('/api/rbac/roles')
  role.value = (rd.data || []).find((r: any) => r.id === id)

  const { data: md } = await api.get('/api/rbac/menus')
  const list: any[] = md.data || []
  const map = new Map<number, any>()
  list.forEach(m => map.set(m.id, { ...m, children: [] as any[] }))
  const roots: any[] = []
  for (const m of map.values()) {
    if (m.parent_id && map.has(m.parent_id)) map.get(m.parent_id).children.push(m)
    else roots.push(m)
  }
  tree.value = roots
})

async function save() {
  saving.value = true
  try {
    const checkedKeys = treeRef.value.getCheckedKeys() as number[]
    const halfChecked = treeRef.value.getHalfCheckedKeys() as number[]
    const all = [...checkedKeys, ...halfChecked]
    const { data } = await api.put(`/api/rbac/roles/${id}/menus`, { menu_ids: all })
    if (data.code === 0) ElMessage.success('ok')
    else ElMessage.error(data.message)
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.card-header { display: flex; justify-content: space-between; align-items: center; }
</style>
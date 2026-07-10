<template>
  <div class="rbac-page">
    <el-card>
      <div class="toolbar">
        <span>{{ $t('sys.rbac.menuPermission') }} - {{ role?.name || id }}</span>
        <el-button type="primary" @click="save" :loading="saving">{{ $t('common.save') }}</el-button>
      </div>

      <el-tree
        ref="treeRef"
        :data="tree"
        show-checkbox
        node-key="id"
        :default-checked-keys="checked"
        :props="{ label: 'title', children: 'children' }"
        style="margin-top: 12px"
      />
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useMessage } from '@/hooks'
import { listRoles, setRoleMenus } from '@/api/sys/roles'
import { listMenus } from '@/api/sys/menus'

const { success, error } = useMessage()
const route = useRoute()
const id = Number(route.params.id)
const role = ref<any>(null)
const tree = ref<any[]>([])
const checked = ref<number[]>([])
const treeRef = ref()
const saving = ref(false)

onMounted(async () => {
  try {
    const roleRes = await listRoles({ page: 1, page_size: 999 })
    const roleList: any[] = roleRes.list || []
    role.value = roleList.find((r: any) => r.id === id)

    const list: any[] = (await listMenus({ page: 1, page_size: 999 })) || []
    const map = new Map<number, any>()
    list.forEach((m) => map.set(m.id, { ...m, children: [] as any[] }))
    const roots: any[] = []
    for (const m of map.values()) {
      if (m.parent_id && map.has(m.parent_id)) map.get(m.parent_id).children.push(m)
      else roots.push(m)
    }
    tree.value = roots
  } catch (e: any) {
    error(e?.message || "common.fail")
  }
})

async function save() {
  saving.value = true
  try {
    const checkedKeys = treeRef.value.getCheckedKeys() as number[]
    const halfChecked = treeRef.value.getHalfCheckedKeys() as number[]
    const all = [...checkedKeys, ...halfChecked]
    await setRoleMenus(id, all)
    success("common.success")
  } catch (e: any) {
    error(e?.message || "common.fail")
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>

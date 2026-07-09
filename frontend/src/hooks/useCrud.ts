import { ref, reactive } from 'vue'
import { ElMessageBox } from 'element-plus'
import { useMessage } from './useMessage'

/**
 * CRUD 操作参数
 */
export interface CrudOptions<T = any> {
  /** 获取列表 API */
  getListApi: (params?: any) => Promise<any>
  /** 创建 API */
  createApi?: (data: T) => Promise<any>
  /** 更新 API */
  updateApi?: (id: any, data: T) => Promise<any>
  /** 删除 API */
  deleteApi?: (id: any) => Promise<any>
  /** 是否分页 */
  isPage?: boolean
  /** 默认分页大小 */
  pageSize?: number
  /** 搜索表单初始值 */
  searchForm?: Record<string, any>
}

/**
 * 通用 CRUD 组合式函数
 */
export function useCrud<T = any>(options: CrudOptions<T>) {
  const { getListApi, createApi, updateApi, deleteApi, isPage = true, pageSize = 20, searchForm: initSearchForm = {} } = options

  const { success, error } = useMessage()

  // 状态
  const loading = ref(false)
  const dataList = ref<any[]>([])
  const total = ref(0)
  const page = ref(1)
  const searchForm = reactive<Record<string, any>>({ ...initSearchForm })

  // 加载数据
  async function load(extraParams?: Record<string, any>) {
    loading.value = true
    try {
      const params: Record<string, any> = { ...searchForm, ...extraParams }
      if (isPage) {
        params.page = page.value
        params.page_size = pageSize
      }
      const res = await getListApi(params)
      if (isPage) {
        dataList.value = res.list || res.data?.list || []
        total.value = res.total || res.data?.total || 0
      } else {
        dataList.value = res.data || res || []
        total.value = Array.isArray(dataList.value) ? dataList.value.length : 0
      }
    } catch (e) {
      console.error('load error:', e)
    } finally {
      loading.value = false
    }
  }

  // 搜索
  function search() {
    page.value = 1
    load()
  }

  // 重置
  function reset() {
    Object.keys(searchForm).forEach((key) => {
      searchForm[key] = ''
    })
    search()
  }

  // 创建
  async function create(data: T): Promise<boolean> {
    if (!createApi) {
      console.warn('createApi not provided')
      return false
    }
    try {
      await createApi(data)
      success('common.success')
      return true
    } catch (e: any) {
      error(() => e?.response?.data?.message || 'common.fail')
      return false
    }
  }

  // 更新
  async function update(id: any, data: T): Promise<boolean> {
    if (!updateApi) {
      console.warn('updateApi not provided')
      return false
    }
    try {
      await updateApi(id, data)
      success('common.success')
      return true
    } catch (e: any) {
      error(() => e?.response?.data?.message || 'common.fail')
      return false
    }
  }

  // 删除
  async function remove(id: any, message = 'common.confirmDelete'): Promise<boolean> {
    if (!deleteApi) {
      console.warn('deleteApi not provided')
      return false
    }
    try {
      await ElMessageBox.confirm(message, 'common.tip', { type: 'warning' })
      await deleteApi(id)
      success('common.deleteSuccess')
      load()
      return true
    } catch (e: any) {
      if (e !== 'cancel') {
        error(() => e?.response?.data?.message || 'common.deleteFail')
      }
      return false
    }
  }

  return {
    loading,
    dataList,
    total,
    page,
    pageSize,
    searchForm,
    load,
    search,
    reset,
    create,
    update,
    remove,
  }
}

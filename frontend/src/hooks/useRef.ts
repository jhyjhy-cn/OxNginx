import { ref } from 'vue'

/**
 * 获取组件实例的类型化 ref
 */
export function useRef<T extends abstract new (...args: any) => any>(_comp: T) {
  return ref<InstanceType<T>>()
}

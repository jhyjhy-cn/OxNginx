import { getData, postData, putData, deleteData } from '@/api/http'
import type { PageQuery, PageResult } from '../users/type'
import type { Post } from './type'

export const listPosts = (params?: PageQuery) =>
  getData<PageResult<Post>>('/api/rbac/posts', params)

export const createPost = (payload: Partial<Post>) =>
  postData<Post>('/api/rbac/posts', payload)

export const updatePost = (id: number, payload: Partial<Post>) =>
  putData(`/api/rbac/posts/${id}`, payload)

export const deletePost = (id: number) => deleteData(`/api/rbac/posts/${id}`)
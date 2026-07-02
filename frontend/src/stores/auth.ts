import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import api from '@/api'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string>(localStorage.getItem('token') || '')
  const username = ref<string>(localStorage.getItem('username') || '')

  const isAuthenticated = computed(() => !!token.value)

  async function login(usernameInput: string, password: string) {
    const response = await api.post('/api/login', {
      username: usernameInput,
      password,
    })

    if (response.data.code === 0) {
      token.value = response.data.data.token
      username.value = response.data.data.username
      localStorage.setItem('token', token.value)
      localStorage.setItem('username', username.value)
      return true
    }

    throw new Error(response.data.message)
  }

  function logout() {
    token.value = ''
    username.value = ''
    localStorage.removeItem('token')
    localStorage.removeItem('username')
  }

  function updateUser(newToken: string, newUsername: string) {
    token.value = newToken
    username.value = newUsername
    localStorage.setItem('token', newToken)
    localStorage.setItem('username', newUsername)
  }

  return {
    token,
    username,
    isAuthenticated,
    login,
    logout,
    updateUser,
  }
})

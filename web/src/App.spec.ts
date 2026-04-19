import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import App from './App.vue'

vi.mock('./api/tunnelApi', () => ({
  getStoredToken: vi.fn(() => null),
  setStoredToken: vi.fn(),
  clearStoredToken: vi.fn(),
  login: vi.fn(),
  getTunnels: vi.fn().mockResolvedValue({ tunnels: [], total: 0 }),
  checkHealth: vi.fn().mockResolvedValue({ status: 'ok' }),
  connectWebSocket: vi.fn(),
  disconnectWebSocket: vi.fn(),
}))

import * as api from './api/tunnelApi'

describe('App authentication', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(api.getStoredToken).mockReturnValue(null)
    localStorage.clear()
  })

  it('shows login form when not authenticated', () => {
    const wrapper = mount(App)
    expect(wrapper.find('.login-container').exists()).toBe(true)
    expect(wrapper.find('.app-container').exists()).toBe(false)
  })

  it('shows management UI when token is stored', () => {
    vi.mocked(api.getStoredToken).mockReturnValue('existing-token')
    const wrapper = mount(App)
    expect(wrapper.find('.login-container').exists()).toBe(false)
    expect(wrapper.find('.app-container').exists()).toBe(true)
  })

  it('shows error message on wrong password', async () => {
    vi.mocked(api.login).mockRejectedValue(new Error('HTTP error! status: 401'))
    const wrapper = mount(App)
    await wrapper.find('input[type="password"]').setValue('wrongpassword')
    await wrapper.find('form').trigger('submit')
    await vi.waitFor(() => {
      expect(wrapper.find('.error-banner').exists()).toBe(true)
    })
    expect(wrapper.find('.error-banner').text()).toContain('密码错误')
  })

  it('shows management UI after successful login', async () => {
    vi.mocked(api.login).mockResolvedValue({ token: 'new-jwt', expires_in: 86400 })
    const wrapper = mount(App)
    await wrapper.find('input[type="password"]').setValue('correct')
    await wrapper.find('form').trigger('submit')
    await vi.waitFor(() => {
      expect(wrapper.find('.app-container').exists()).toBe(true)
    })
  })
})

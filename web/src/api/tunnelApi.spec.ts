import { describe, it, expect, beforeEach, vi } from 'vitest'
import {
  getStoredToken,
  setStoredToken,
  clearStoredToken,
  login,
} from './tunnelApi'

describe('token storage', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  it('returns null when no token is stored', () => {
    expect(getStoredToken()).toBeNull()
  })

  it('stores and retrieves a token', () => {
    setStoredToken('my-token')
    expect(getStoredToken()).toBe('my-token')
  })

  it('clears a stored token', () => {
    setStoredToken('my-token')
    clearStoredToken()
    expect(getStoredToken()).toBeNull()
  })
})

describe('login', () => {
  beforeEach(() => {
    vi.resetAllMocks()
  })

  it('returns token and expires_in on success', async () => {
    const mockResponse = { token: 'jwt-token', expires_in: 86400 }
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(mockResponse),
    }))

    const result = await login('correct-password')
    expect(result.token).toBe('jwt-token')
    expect(result.expires_in).toBe(86400)
  })

  it('throws on non-200 response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      status: 401,
    }))

    await expect(login('wrong-password')).rejects.toThrow('HTTP error! status: 401')
  })
})

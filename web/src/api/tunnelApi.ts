/**
 * 隧道管理 API 调用模块
 */

const API_BASE_URL = 'http://localhost:8080';

const TOKEN_KEY = 'auth_token'

export function getStoredToken(): string | null {
  return localStorage.getItem(TOKEN_KEY)
}

export function setStoredToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token)
}

export function clearStoredToken(): void {
  localStorage.removeItem(TOKEN_KEY)
}

function authHeaders(): Record<string, string> {
  const token = getStoredToken()
  return token ? { 'Authorization': `Bearer ${token}` } : {}
}

export interface Tunnel {
  id: string;
  subdomain: string;
  auth_token: string;
  local_port: number;
  status: string;
  created_at_iso: string;
  bytes_transferred: number;
}

export interface ListTunnelsResponse {
  tunnels: Tunnel[];
  total: number;
}

export interface CreateTunnelRequest {
  auth_token: string;
  local_port: number;
  subdomain: string;
  protocols: string[];
}

export interface CreateTunnelResponse {
  tunnel_id: string;
  public_url: string;
  server_domain: string;
}

/**
 * 获取隧道列表
 */
export const getTunnels = async (): Promise<ListTunnelsResponse> => {
  try {
    const res = await fetch(`${API_BASE_URL}/api/v1/tunnels`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        ...authHeaders(),
      },
    });

    if (!res.ok) {
      throw new Error(`HTTP error! status: ${res.status}`);
    }

    const data = await res.json();
    return data as ListTunnelsResponse;
  } catch (error) {
    throw error;
  }
};

/**
 * 创建隧道
 */
export const createTunnel = async (data: CreateTunnelRequest): Promise<CreateTunnelResponse> => {
  try {
    const res = await fetch(`${API_BASE_URL}/api/v1/tunnels`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...authHeaders(),
      },
      body: JSON.stringify(data),
    });

    if (!res.ok) {
      throw new Error(`HTTP error! status: ${res.status}`);
    }

    const responseData = await res.json();
    return responseData as CreateTunnelResponse;
  } catch (error) {
    throw error;
  }
};

/**
 * 删除隧道
 */
export const deleteTunnel = async (tunnelId: string): Promise<void> => {
  try {
    const res = await fetch(`${API_BASE_URL}/api/v1/tunnels/${tunnelId}`, {
      method: 'DELETE',
      headers: {
        'Content-Type': 'application/json',
        ...authHeaders(),
      },
    });

    if (!res.ok && res.status !== 204) {
      throw new Error(`HTTP error! status: ${res.status}`);
    }
  } catch (error) {
    throw error;
  }
};

/**
 * 健康检查
 */
export const checkHealth = async (): Promise<{ status: string }> => {
  try {
    const res = await fetch(`${API_BASE_URL}/health`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!res.ok) {
      throw new Error(`HTTP error! status: ${res.status}`);
    }

    return await res.json();
  } catch (error) {
    throw error;
  }
};

export interface LoginResponse {
  token: string
  expires_in: number
}

export const login = async (password: string): Promise<LoginResponse> => {
  const res = await fetch(`${API_BASE_URL}/api/v1/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ password }),
  })
  if (!res.ok) {
    throw new Error(`HTTP error! status: ${res.status}`)
  }
  return res.json()
}

// ===== WebSocket 实时订阅 =====

export interface TunnelConnectedEvent {
  courier_id: string
  subdomain: string
  public_url: string
  local_port: number
}

export interface TunnelDisconnectedEvent {
  courier_id: string
}

export interface TunnelStatsItem {
  courier_id: string
  bytes_transferred: number
}

export interface StatsUpdateEvent {
  tunnels: TunnelStatsItem[]
}

export type WsEventHandler = {
  onConnected: (evt: TunnelConnectedEvent) => void
  onDisconnected: (evt: TunnelDisconnectedEvent) => void
  onStatsUpdate: (evt: StatsUpdateEvent) => void
  onSnapshot: (evt: TunnelConnectedEvent) => void
}

let ws: WebSocket | null = null
let reconnectDelay = 1000

export function connectWebSocket(handlers: WsEventHandler): void {
  if (ws && ws.readyState === WebSocket.OPEN) return

  ws = new WebSocket('ws://localhost:8080/ws')

  ws.onopen = () => {
    reconnectDelay = 1000
    const token = getStoredToken() ?? ''
    ws!.send(JSON.stringify({ msg_type: 'subscribe', data: { token } }))
  }

  ws.onmessage = (event) => {
    try {
      const msg = JSON.parse(event.data as string) as { msg_type: string; data: unknown }
      switch (msg.msg_type) {
        case 'tunnel_connected':
          handlers.onConnected(msg.data as TunnelConnectedEvent)
          break
        case 'tunnel_disconnected':
          handlers.onDisconnected(msg.data as TunnelDisconnectedEvent)
          break
        case 'stats_update':
          handlers.onStatsUpdate(msg.data as StatsUpdateEvent)
          break
      }
    } catch {
      // 忽略无法解析的消息
    }
  }

  ws.onclose = () => {
    ws = null
    setTimeout(() => connectWebSocket(handlers), reconnectDelay)
    reconnectDelay = Math.min(reconnectDelay * 2, 30000)
  }

  ws.onerror = () => {
    ws?.close()
  }
}

export function disconnectWebSocket(): void {
  if (ws) {
    ws.onclose = null
    ws.close()
    ws = null
  }
}

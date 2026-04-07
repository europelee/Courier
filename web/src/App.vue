<template>
  <div class="app-container">
    <!-- 侧边栏 -->
    <nav class="sidebar">
      <div class="logo">
        <h1>隧道管理</h1>
      </div>
      <ul class="menu">
        <li :class="{ active: currentView === 'tunnels' }">
          <a @click="currentView = 'tunnels'">隧道列表</a>
        </li>
        <li :class="{ active: currentView === 'create' }">
          <a @click="currentView = 'create'">创建隧道</a>
        </li>
        <li :class="{ active: currentView === 'logs' }">
          <a @click="currentView = 'logs'">日志查看</a>
        </li>
        <li :class="{ active: currentView === 'stats' }">
          <a @click="currentView = 'stats'">监控数据</a>
        </li>
      </ul>
    </nav>

    <!-- 主内容区 -->
    <main class="main-content">
      <header class="header">
        <h2>{{ pageTitle }}</h2>
        <div class="header-info">
          <span>服务器状态: {{ serverStatus }}</span>
          <span>隧道数: {{ activeTunnels }}</span>
        </div>
      </header>

      <!-- 错误提示 -->
      <div v-if="errorMessage" class="error-banner">
        <span>⚠️ {{ errorMessage }}</span>
        <button @click="errorMessage = ''" class="close-btn">×</button>
      </div>

      <div class="content">
        <!-- 隧道列表 -->
        <div v-if="currentView === 'tunnels'" class="view-container">
          <h3>活跃隧道</h3>
          
          <!-- 加载状态 -->
          <div v-if="isLoading" class="loading">
            <div class="spinner"></div>
            <p>加载中...</p>
          </div>
          
          <!-- 空状态 -->
          <div v-else-if="tunnels.length === 0" class="empty-state">
            <p>暂无隧道</p>
            <button @click="currentView = 'create'" class="btn btn-primary">
              创建第一个隧道
            </button>
          </div>
          
          <!-- 隧道列表 -->
          <div v-else class="tunnel-list">
            <div v-for="tunnel in tunnels" :key="tunnel.id" class="tunnel-card">
              <div class="tunnel-header">
                <h4>{{ tunnel.subdomain }}</h4>
                <span :class="['status', tunnel.status]">{{ tunnel.status === 'connected' ? '已连接' : '未连接' }}</span>
              </div>
              <div class="tunnel-details">
                <p>本地端口: {{ tunnel.local_port }}</p>
                <p>流量: {{ formatBytes(tunnel.bytes_transferred) }}</p>
              </div>
              <div class="tunnel-actions">
                <button class="btn btn-small" @click="deleteTunnel(tunnel.id)">删除</button>
              </div>
            </div>
          </div>
        </div>

        <!-- 创建隧道 -->
        <div v-else-if="currentView === 'create'" class="view-container">
          <h3>创建新隧道</h3>
          <form @submit.prevent="createTunnel" class="form">
            <div class="form-group">
              <label>子域名 (可选，自动生成)</label>
              <input v-model="formData.subdomain" type="text" placeholder="my-tunnel">
            </div>
            <div class="form-group">
              <label>本地端口 (1-65535) *</label>
              <input v-model.number="formData.local_port" type="number" required min="1" max="65535">
            </div>
            <div class="form-group">
              <label>认证令牌 *</label>
              <input v-model="formData.token" type="password" required placeholder="输入安全的令牌">
            </div>
            <div class="form-group">
              <label>协议</label>
              <select v-model="formData.protocol">
                <option value="http">HTTP</option>
                <option value="https">HTTPS</option>
              </select>
            </div>
            <button type="submit" class="btn btn-primary" :disabled="isCreating">
              {{ isCreating ? '创建中...' : '创建隧道' }}
            </button>
          </form>
        </div>

        <!-- 日志查看 -->
        <div v-else-if="currentView === 'logs'" class="view-container">
          <h3>系统日志</h3>
          <div class="logs">
            <div v-if="logs.length === 0" class="empty-state">
              <p>暂无日志</p>
            </div>
            <div v-else>
              <div v-for="(log, index) in logs" :key="index" :class="['log-entry', log.level]">
                <span class="timestamp">{{ log.timestamp }}</span>
                <span class="level">{{ log.level }}</span>
                <span class="message">{{ log.message }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 监控数据 -->
        <div v-else-if="currentView === 'stats'" class="view-container">
          <h3>监控统计</h3>
          <div class="stats-grid">
            <div class="stat-card">
              <h4>活跃隧道</h4>
              <p class="stat-value">{{ activeTunnels }}</p>
            </div>
            <div class="stat-card">
              <h4>总流量</h4>
              <p class="stat-value">{{ formatBytes(totalBytes) }}</p>
            </div>
            <div class="stat-card">
              <h4>错误数</h4>
              <p class="stat-value">{{ errorCount }}</p>
            </div>
            <div class="stat-card">
              <h4>平均响应时间</h4>
              <p class="stat-value">{{ avgResponseTime }}ms</p>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import * as api from './api/tunnelApi'

interface Tunnel {
  id: string
  subdomain: string
  local_port: number
  status: 'connected' | 'disconnected'
  bytes_transferred: number
}

const currentView = ref<string>('tunnels')
const tunnels = ref<Tunnel[]>([])
const logs = ref<Array<{ timestamp: string; level: string; message: string }>>([])
const serverStatus = ref('正常')
const activeTunnels = ref(0)
const totalBytes = ref(0)
const errorCount = ref(0)
const avgResponseTime = ref(0)

// 加载状态
const isLoading = ref(false)
const isCreating = ref(false)
const errorMessage = ref('')

const formData = ref({
  subdomain: '',
  local_port: 3000,
  token: '',
  protocol: 'http'
})

const pageTitle = computed(() => {
  const titles: Record<string, string> = {
    tunnels: '隧道列表',
    create: '创建隧道',
    logs: '日志查看',
    stats: '监控数据'
  }
  return titles[currentView.value] || '隧道管理'
})

const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
}

// 获取隧道列表
const fetchTunnels = async () => {
  isLoading.value = true
  errorMessage.value = ''
  try {
    const response = await api.getTunnels()
    tunnels.value = response.tunnels.map(t => ({
      id: t.id,
      subdomain: t.subdomain,
      local_port: t.local_port,
      status: t.status === 'connected' ? 'connected' : 'disconnected',
      bytes_transferred: t.bytes_transferred
    }))
    activeTunnels.value = response.total
    addLog('INFO', `成功加载 ${response.total} 个隧道`)
  } catch (error) {
    console.error('获取隧道列表失败:', error)
    errorMessage.value = '获取隧道列表失败：' + String(error)
    addLog('ERROR', '获取隧道列表失败：' + String(error))
  } finally {
    isLoading.value = false
  }
}

// 创建隧道
const createTunnel = async () => {
  isCreating.value = true
  errorMessage.value = ''
  try {
    if (formData.value.local_port < 1 || formData.value.local_port > 65535) {
      throw new Error('本地端口必须在 1-65535 之间')
    }
    if (!formData.value.token) {
      throw new Error('认证令牌不能为空')
    }

    const response = await api.createTunnel({
      auth_token: formData.value.token,
      local_port: formData.value.local_port,
      subdomain: formData.value.subdomain,
      protocols: [formData.value.protocol]
    })

    addLog('INFO', `隧道创建成功：${response.tunnel_id}`)
    
    // 重置表单
    formData.value = { subdomain: '', local_port: 3000, token: '', protocol: 'http' }
    
    // 回到列表页面
    currentView.value = 'tunnels'
    
    // 刷新列表
    await fetchTunnels()
  } catch (error) {
    console.error('创建隧道失败:', error)
    errorMessage.value = '创建隧道失败：' + String(error)
    addLog('ERROR', '创建隧道失败：' + String(error))
  } finally {
    isCreating.value = false
  }
}

// 删除隧道
const deleteTunnel = async (tunnelId: string) => {
  if (!confirm('确定要删除这个隧道吗？')) return

  errorMessage.value = ''
  try {
    await api.deleteTunnel(tunnelId)
    addLog('INFO', `隧道已删除：${tunnelId}`)
    await fetchTunnels()
  } catch (error) {
    console.error('删除隧道失败:', error)
    errorMessage.value = '删除隧道失败：' + String(error)
    addLog('ERROR', '删除隧道失败：' + String(error))
  }
}

// 添加日志
const addLog = (level: string, message: string) => {
  const now = new Date().toLocaleTimeString()
  logs.value.unshift({ timestamp: now, level, message })
  if (logs.value.length > 100) logs.value.pop()
}

// 健康检查
const checkHealth = async () => {
  try {
    await api.checkHealth()
    serverStatus.value = '正常'
  } catch (error) {
    serverStatus.value = '异常'
    addLog('ERROR', '服务器健康检查失败')
  }
}

// 初始化
onMounted(async () => {
  addLog('INFO', '应用启动')
  await checkHealth()
  await fetchTunnels()
  
  // 定期刷新（每 5 秒）
  setInterval(async () => {
    await checkHealth()
    if (currentView.value === 'tunnels') {
      await fetchTunnels()
    }
  }, 5000)
})
</script>

<style scoped>
.app-container {
  display: flex;
  min-height: 100vh;
  background: #f5f5f5;
}

.sidebar {
  width: 250px;
  background: #2c3e50;
  color: white;
  padding: 20px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.logo h1 {
  font-size: 20px;
  margin-bottom: 30px;
  text-align: center;
  border-bottom: 1px solid #444;
  padding-bottom: 20px;
}

.menu {
  list-style: none;
}

.menu li {
  margin: 10px 0;
}

.menu li a {
  display: block;
  padding: 12px 15px;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.3s;
}

.menu li a:hover,
.menu li.active a {
  background: #667eea;
  color: white;
}

.main-content {
  flex: 1;
  overflow-y: auto;
}

.header {
  background: white;
  padding: 20px;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.header h2 {
  font-size: 24px;
  color: #2c3e50;
}

.header-info {
  display: flex;
  gap: 20px;
  font-size: 14px;
  color: #666;
}

.error-banner {
  background: #ffebee;
  color: #c62828;
  padding: 12px 20px;
  margin: 20px;
  border-radius: 4px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-left: 4px solid #c62828;
}

.close-btn {
  background: none;
  border: none;
  color: #c62828;
  font-size: 20px;
  cursor: pointer;
  padding: 0;
}

.content {
  padding: 30px;
}

.view-container {
  background: white;
  border-radius: 8px;
  padding: 25px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.view-container h3 {
  font-size: 18px;
  margin-bottom: 20px;
  color: #2c3e50;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px;
}

.spinner {
  border: 4px solid #f3f3f3;
  border-top: 4px solid #667eea;
  border-radius: 50%;
  width: 40px;
  height: 40px;
  animation: spin 1s linear infinite;
  margin-bottom: 10px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: #999;
}

.empty-state p {
  margin-bottom: 20px;
  font-size: 16px;
}

.tunnel-list {
  display: grid;
  gap: 15px;
}

.tunnel-card {
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  padding: 15px;
  background: #f9f9f9;
  transition: all 0.3s;
}

.tunnel-card:hover {
  background: #fff;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.tunnel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.tunnel-header h4 {
  color: #2c3e50;
}

.status {
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: bold;
}

.status.connected {
  background: #d4edda;
  color: #155724;
}

.status.disconnected {
  background: #f8d7da;
  color: #721c24;
}

.tunnel-details {
  font-size: 14px;
  color: #666;
  margin-bottom: 10px;
}

.tunnel-details p {
  margin: 5px 0;
}

.tunnel-actions {
  display: flex;
  gap: 10px;
}

.form {
  max-width: 500px;
}

.form-group {
  margin-bottom: 15px;
}

.form-group label {
  display: block;
  margin-bottom: 5px;
  color: #2c3e50;
  font-weight: 500;
}

.form-group input,
.form-group select {
  width: 100%;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.logs {
  max-height: 400px;
  overflow-y: auto;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  background: #f9f9f9;
}

.log-entry {
  padding: 10px;
  border-bottom: 1px solid #e0e0e0;
  font-family: monospace;
  font-size: 12px;
  display: flex;
  gap: 10px;
}

.log-entry.ERROR {
  color: #d32f2f;
  background: #ffebee;
}

.log-entry.WARN {
  color: #f57c00;
  background: #fff3e0;
}

.log-entry.INFO {
  color: #1976d2;
  background: #e3f2fd;
}

.timestamp,
.level {
  flex-shrink: 0;
  font-weight: bold;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
}

.stat-card {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.stat-card h4 {
  font-size: 14px;
  opacity: 0.9;
  margin-bottom: 10px;
}

.stat-value {
  font-size: 28px;
  font-weight: bold;
}

.btn {
  padding: 10px 15px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s;
}

.btn-primary {
  background: #667eea;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #5568d3;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-small {
  padding: 6px 12px;
  font-size: 12px;
  background: #f5f5f5;
  border: 1px solid #ddd;
  color: #2c3e50;
}

.btn-small:hover {
  background: #e0e0e0;
}
</style>

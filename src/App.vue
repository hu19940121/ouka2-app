<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRadioStore } from './stores/radio'
import StationCard from './components/StationCard.vue'
import AudioPlayer from './components/AudioPlayer.vue'
import StatusBar from './components/StatusBar.vue'
import CrawlProgress from './components/CrawlProgress.vue'
import type { Station } from './types'

const store = useRadioStore()

// 当前播放的电台
const currentStation = ref<Station | null>(null)
// 播放器 key，用于强制重建组件
const playerKey = ref(0)

// 消息提示
const toast = ref<{ message: string; type: 'success' | 'error' | 'info' } | null>(null)

// 显示提示
const showToast = (message: string, type: 'success' | 'error' | 'info' = 'info') => {
  toast.value = { message, type }
  setTimeout(() => {
    toast.value = null
  }, 3000)
}

// 播放电台
const handlePlay = async (station: Station) => {
  if (!store.serverStatus.running) {
    showToast('请先启动服务器', 'error')
    return
  }
  
  // 如果正在播放，先关闭当前的
  if (currentStation.value) {
    currentStation.value = null
    // 增加 key 来强制销毁旧的 audio 元素
    playerKey.value++
    // 等待一小段时间让浏览器断开连接
    await new Promise(resolve => setTimeout(resolve, 100))
  }
  
  // 播放新电台
  playerKey.value++
  currentStation.value = station
}

// 复制地址
const handleCopy = (_url: string) => {
  showToast('地址已复制到剪贴板', 'success')
}

// 关闭播放器
const handleClosePlayer = () => {
  currentStation.value = null
  playerKey.value++
}

// 启动服务器
const handleStartServer = async () => {
  await store.startServer()
  if (store.serverStatus.running) {
    showToast('服务器已启动', 'success')
  }
}

// 停止服务器
const handleStopServer = async () => {
  currentStation.value = null
  playerKey.value++
  await store.stopServer()
  showToast('服务器已停止', 'info')
}

// 刷新电台数据
const handleCrawl = async () => {
  await store.crawlStations()
  showToast(`已获取 ${store.stations.length} 个电台`, 'success')
}

// 安装到欧卡2
const handleInstall = async () => {
  try {
    const path = await store.installToEts2()
    showToast(`配置已安装到: ${path}`, 'success')
  } catch (e) {
    showToast(String(e), 'error')
  }
}

// 当前流地址
const currentStreamUrl = computed(() => {
  if (!currentStation.value) return ''
  return store.getStreamUrl(currentStation.value.id)
})

// 初始化
onMounted(async () => {
  // 检查 FFmpeg
  await store.checkFfmpeg()

  // 加载电台数据
  await store.loadStations()

  // 如果没有数据，提示用户
  if (store.stations.length === 0) {
    showToast('首次使用，请点击"刷新数据"获取电台', 'info')
  }

  // 定期刷新服务器状态
  setInterval(() => {
    if (store.serverStatus.running) {
      store.refreshServerStatus()
    }
  }, 5000)
})
</script>

<template>
  <div class="app">
    <!-- 顶部标题栏 -->
    <header class="header">
      <div class="header-left">
        <span class="logo">🚛</span>
        <h1>欧卡2中国电台</h1>
      </div>
      <div class="header-right">
        <span class="version">v0.1.1</span>
      </div>
    </header>

    <!-- 主内容区 -->
    <main class="main">
      <!-- 搜索和筛选栏 -->
      <div class="toolbar">
        <div class="search-box">
          <span class="search-icon">🔍</span>
          <input
            type="text"
            v-model="store.searchQuery"
            placeholder="搜索电台..."
            class="search-input"
          />
        </div>

        <div class="filters">
          <select v-model="store.selectedProvince" class="province-select">
            <option value="">全部地区</option>
            <option v-for="p in store.provinces" :key="p" :value="p">
              {{ p }}
            </option>
          </select>
        </div>

        <div class="station-count">
          显示 {{ store.filteredStations.length }} / {{ store.stations.length }} 个电台
        </div>
      </div>

      <!-- 电台列表 -->
      <div class="station-list" v-if="store.filteredStations.length > 0">
        <StationCard
          v-for="station in store.filteredStations"
          :key="station.id"
          :station="station"
          :server-port="store.serverStatus.port"
          @play="handlePlay"
          @copy="handleCopy"
        />
      </div>

      <!-- 空状态 -->
      <div class="empty-state" v-else-if="!store.isLoading">
        <div class="empty-icon">📻</div>
        <h3>暂无电台数据</h3>
        <p v-if="store.stations.length === 0">
          点击下方"刷新数据"按钮获取电台列表
        </p>
        <p v-else>
          没有找到匹配的电台，试试其他搜索条件？
        </p>
      </div>

      <!-- 加载状态 -->
      <div class="loading-state" v-if="store.isLoading">
        <div class="spinner"></div>
        <p>加载中...</p>
      </div>
    </main>

    <!-- 播放器 - 使用 key 强制重建 -->
    <AudioPlayer
      :key="playerKey"
      :station="currentStation"
      :stream-url="currentStreamUrl"
      @close="handleClosePlayer"
    />

    <!-- 状态栏 -->
    <StatusBar
      :status="store.serverStatus"
      :station-count="store.stations.length"
      :ffmpeg-status="store.ffmpegStatus"
      @start="handleStartServer"
      @stop="handleStopServer"
      @crawl="handleCrawl"
      @install="handleInstall"
    />

    <!-- 爬虫进度 -->
    <CrawlProgress
      v-if="store.isCrawling && store.crawlProgress"
      :progress="store.crawlProgress"
    />

    <!-- 消息提示 -->
    <Transition name="toast">
      <div v-if="toast" :class="['toast', `toast-${toast.type}`]">
        {{ toast.message }}
      </div>
    </Transition>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: linear-gradient(135deg, #0f0c29, #302b63, #24243e);
  color: white;
  min-height: 100vh;
  overflow: hidden;
}

.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

/* 顶部标题栏 */
.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  background: rgba(0, 0, 0, 0.3);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.logo {
  font-size: 1.8rem;
}

.header h1 {
  font-size: 1.4rem;
  font-weight: 600;
  background: linear-gradient(135deg, #4facfe, #00f2fe);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.version {
  font-size: 0.8rem;
  color: rgba(255, 255, 255, 0.5);
  background: rgba(255, 255, 255, 0.1);
  padding: 0.2rem 0.6rem;
  border-radius: 4px;
}

/* 主内容区 */
.main {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  padding: 1rem 1.5rem;
}

/* 工具栏 */
.toolbar {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
}

.search-box {
  flex: 1;
  max-width: 400px;
  position: relative;
}

.search-icon {
  position: absolute;
  left: 1rem;
  top: 50%;
  transform: translateY(-50%);
  font-size: 1rem;
}

.search-input {
  width: 100%;
  padding: 0.8rem 1rem 0.8rem 2.8rem;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.05);
  color: white;
  font-size: 0.95rem;
  outline: none;
  transition: all 0.2s ease;
}

.search-input:focus {
  border-color: #4facfe;
  background: rgba(255, 255, 255, 0.1);
}

.search-input::placeholder {
  color: rgba(255, 255, 255, 0.4);
}

.filters {
  display: flex;
  gap: 0.8rem;
}

.province-select {
  padding: 0.8rem 1rem;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.05);
  color: white;
  font-size: 0.95rem;
  outline: none;
  cursor: pointer;
  min-width: 150px;
}

.province-select option {
  background: #1a1a2e;
  color: white;
}

.station-count {
  margin-left: auto;
  color: rgba(255, 255, 255, 0.6);
  font-size: 0.9rem;
}

/* 电台列表 */
.station-list {
  flex: 1;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 1rem;
  padding-right: 0.5rem;
  align-content: start;
}

.station-list::-webkit-scrollbar {
  width: 6px;
}

.station-list::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 3px;
}

.station-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 3px;
}

.station-list::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}

/* 空状态 */
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.6);
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
  opacity: 0.5;
}

.empty-state h3 {
  font-size: 1.3rem;
  margin-bottom: 0.5rem;
  color: rgba(255, 255, 255, 0.8);
}

.empty-state p {
  font-size: 0.95rem;
}

/* 加载状态 */
.loading-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-top-color: #4facfe;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 1rem;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 消息提示 */
.toast {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  padding: 0.8rem 1.5rem;
  border-radius: 10px;
  font-size: 0.95rem;
  z-index: 200;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

.toast-success {
  background: linear-gradient(135deg, #4ade80, #22d3ee);
  color: #000;
}

.toast-error {
  background: linear-gradient(135deg, #f87171, #fb923c);
  color: #000;
}

.toast-info {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-20px);
}
</style>
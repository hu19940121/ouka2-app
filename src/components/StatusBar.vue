<script setup lang="ts">
import { computed } from 'vue'
import type { ServerStatus } from '../types'

const props = defineProps<{
  status: ServerStatus
  stationCount: number
  selectedStationCount: number
  ffmpegStatus: string | null
}>()

defineEmits<{
  start: []
  stop: []
  refresh: []
  crawl: []
  install: []
}>()

const statusText = computed(() => {
  return props.status.running ? '运行中' : '已停止'
})

const statusColor = computed(() => {
  return props.status.running ? '#4ade80' : '#f87171'
})
</script>

<template>
  <div class="status-bar">
    <div class="status-left">
      <div class="status-indicator">
        <span class="status-dot" :style="{ background: statusColor }"></span>
        <span class="status-text">服务器 {{ statusText }}</span>
        <span v-if="status.running" class="port-badge">端口 {{ status.port }}</span>
      </div>

      <div class="status-info">
        <span class="info-item">
          📻 {{ stationCount }} 个电台
        </span>
        <span class="info-item selected">
          📦 待安装 {{ selectedStationCount }} 个
        </span>
        <span v-if="status.running && status.active_streams > 0" class="info-item active">
          🎵 {{ status.active_streams }} 个活动流
        </span>
      </div>
    </div>

    <div class="status-right">
      <button
        v-if="!status.running"
        class="btn btn-start"
        @click="$emit('start')"
      >
        ▶ 启动服务器
      </button>
      <button
        v-else
        class="btn btn-stop"
        @click="$emit('stop')"
      >
        ⏹ 停止服务器
      </button>

      <button
        class="btn btn-secondary"
        @click="$emit('crawl')"
        title="刷新电台数据"
      >
        🔄 刷新数据
      </button>

      <button
        class="btn btn-primary"
        @click="$emit('install')"
        :disabled="stationCount === 0 || selectedStationCount === 0"
        title="安装选中的电台到欧卡2"
      >
        📥 安装选中电台
      </button>
    </div>
  </div>

  <div v-if="!ffmpegStatus" class="ffmpeg-warning">
    ⚠️ FFmpeg 未检测到，请确保已安装 FFmpeg 并添加到系统 PATH
  </div>
</template>

<style scoped>
.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.8rem 1.5rem;
  background: rgba(0, 0, 0, 0.3);
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.status-left {
  display: flex;
  align-items: center;
  gap: 2rem;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.status-text {
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
}

.port-badge {
  background: rgba(255, 255, 255, 0.1);
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  color: rgba(255, 255, 255, 0.7);
}

.status-info {
  display: flex;
  gap: 1rem;
}

.info-item {
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.9rem;
}

.info-item.active {
  color: #4ade80;
}

.info-item.selected {
  color: #7dd3fc;
}

.status-right {
  display: flex;
  gap: 0.8rem;
}

.btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.9rem;
  font-weight: 500;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 0.3rem;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-start {
  background: linear-gradient(135deg, #4ade80, #22d3ee);
  color: #000;
}

.btn-start:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(74, 222, 128, 0.4);
}

.btn-stop {
  background: linear-gradient(135deg, #f87171, #fb923c);
  color: #000;
}

.btn-stop:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(248, 113, 113, 0.4);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.9);
}

.btn-secondary:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.2);
}

.btn-primary {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.ffmpeg-warning {
  background: rgba(251, 146, 60, 0.2);
  border: 1px solid rgba(251, 146, 60, 0.5);
  color: #fb923c;
  padding: 0.8rem 1.5rem;
  font-size: 0.9rem;
}
</style>

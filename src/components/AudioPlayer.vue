<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import type { Station } from '../types'

const props = defineProps<{
  station: Station | null
  streamUrl: string
}>()

const emit = defineEmits<{
  close: []
}>()

const audioRef = ref<HTMLAudioElement | null>(null)
const isPlaying = ref(false)
const volume = ref(80)
const isLoading = ref(false)
const error = ref<string | null>(null)

// 组件挂载后自动播放
onMounted(() => {
  if (props.station && props.streamUrl && audioRef.value) {
    startPlayback()
  }
})

// 组件卸载时停止播放
onUnmounted(() => {
  if (audioRef.value) {
    audioRef.value.pause()
    audioRef.value.src = ''
  }
})

const startPlayback = async () => {
  if (!audioRef.value || !props.streamUrl) return
  
  isLoading.value = true
  error.value = null
  
  audioRef.value.src = props.streamUrl
  audioRef.value.volume = volume.value / 100
  
  try {
    await audioRef.value.play()
  } catch (e: any) {
    error.value = '播放失败: ' + e.message
    isLoading.value = false
  }
}

const togglePlay = () => {
  if (!audioRef.value) return

  if (isPlaying.value) {
    audioRef.value.pause()
  } else {
    audioRef.value.play().catch((e) => {
      error.value = '播放失败: ' + e.message
    })
  }
}

const handlePlay = () => {
  isPlaying.value = true
  isLoading.value = false
  error.value = null
}

const handlePause = () => {
  isPlaying.value = false
}

const handleError = () => {
  isLoading.value = false
  if (props.streamUrl) {
    error.value = '无法加载音频流'
  }
}

const handleVolumeChange = () => {
  if (audioRef.value) {
    audioRef.value.volume = volume.value / 100
  }
}

const handleClose = () => {
  // 停止播放
  if (audioRef.value) {
    audioRef.value.pause()
    audioRef.value.src = ''
  }
  emit('close')
}

// 获取类型图标
const getTypeIcon = (name: string) => {
  if (!name) return '📻'
  if (name.includes('新闻') || name.includes('之声')) return '📰'
  if (name.includes('音乐') || name.includes('Music')) return '🎵'
  if (name.includes('交通') || name.includes('高速')) return '🚗'
  return '📻'
}
</script>

<template>
  <div v-if="station" class="player">
    <audio
      ref="audioRef"
      @play="handlePlay"
      @pause="handlePause"
      @error="handleError"
    />
    
    <div class="player-info">
      <span class="player-icon">{{ getTypeIcon(station.name) }}</span>
      <div class="player-text">
        <span class="player-name">{{ station.name }}</span>
        <span class="player-province">{{ station.province }}</span>
      </div>
    </div>

    <div class="player-controls">
      <div v-if="isLoading" class="loading-indicator">
        <span class="spinner"></span>
        <span>加载中...</span>
      </div>
      
      <div v-else-if="error" class="error-message">
        {{ error }}
      </div>
      
      <button v-else class="btn-control" @click="togglePlay">
        <span v-if="isPlaying">⏸</span>
        <span v-else>▶</span>
      </button>

      <div class="volume-control">
        <span class="volume-icon">🔊</span>
        <input
          type="range"
          v-model="volume"
          min="0"
          max="100"
          class="volume-slider"
          @input="handleVolumeChange"
        />
      </div>

      <button class="btn-close" @click="handleClose" title="关闭">
        ✕
      </button>
    </div>
  </div>
</template>

<style scoped>
.player {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.8rem 1.5rem;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.2), rgba(118, 75, 162, 0.2));
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
}

.player-info {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.player-icon {
  font-size: 1.5rem;
}

.player-text {
  display: flex;
  flex-direction: column;
}

.player-name {
  font-weight: 600;
  color: #fff;
  font-size: 0.95rem;
}

.player-province {
  font-size: 0.8rem;
  color: rgba(255, 255, 255, 0.6);
}

.player-controls {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.btn-control {
  width: 45px;
  height: 45px;
  border: none;
  border-radius: 50%;
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  font-size: 1.2rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.btn-control:hover {
  transform: scale(1.1);
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.5);
}

.volume-control {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.volume-icon {
  font-size: 1rem;
}

.volume-slider {
  width: 100px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
  outline: none;
}

.volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: linear-gradient(135deg, #4facfe, #00f2fe);
  cursor: pointer;
}

.btn-close {
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.btn-close:hover {
  background: rgba(255, 100, 100, 0.3);
  color: #ff6b6b;
}

.loading-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.9rem;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #4facfe;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.error-message {
  color: #ff6b6b;
  font-size: 0.85rem;
}
</style>

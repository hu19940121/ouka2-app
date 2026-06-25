<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { ChevronDown, ChevronUp, Pause, Play, SkipBack, SkipForward, Volume2, X } from 'lucide-vue-next'
import type { Station } from '../types'

const props = defineProps<{
  station: Station | null
  streamUrl: string
  stations: Station[]
  currentStationId: string | null
  collapsed: boolean
}>()

const emit = defineEmits<{
  close: []
  previous: []
  next: []
  'toggle-collapse': []
}>()

const audioRef = ref<HTMLAudioElement | null>(null)
const isPlaying = ref(false)
const volume = ref(80)
const isLoading = ref(false)
const error = ref<string | null>(null)
const currentTime = ref(0)
const duration = ref(0)

const stationInitials = computed(() => props.station?.name.trim().slice(0, 2) || '电台')
const canNavigate = computed(() => {
  return Boolean(props.currentStationId) && props.stations.length > 1
})
const progressValue = computed(() => {
  if (!Number.isFinite(duration.value) || duration.value <= 0) {
    return isPlaying.value ? 56 : 0
  }
  return Math.min(100, (currentTime.value / duration.value) * 100)
})

const formatTime = (seconds: number) => {
  if (!Number.isFinite(seconds) || seconds <= 0) return '00:00'
  const minutes = Math.floor(seconds / 60)
  const rest = Math.floor(seconds % 60)
  return `${String(minutes).padStart(2, '0')}:${String(rest).padStart(2, '0')}`
}

const elapsedText = computed(() => formatTime(currentTime.value))
const durationText = computed(() => {
  if (!Number.isFinite(duration.value) || duration.value <= 0) return '直播'
  return formatTime(duration.value)
})
const volumeTrackStyle = computed(() => ({
  background: `linear-gradient(90deg, var(--accent) 0%, var(--accent) ${volume.value}%, var(--border) ${volume.value}%, var(--border) 100%)`,
}))

const resetAudioElement = () => {
  if (!audioRef.value) return

  audioRef.value.pause()
  audioRef.value.removeAttribute('src')
  audioRef.value.load()
  isPlaying.value = false
  currentTime.value = 0
  duration.value = 0
}

const getPlaybackUrl = () => {
  const separator = props.streamUrl.includes('?') ? '&' : '?'
  return `${props.streamUrl}${separator}t=${Date.now()}`
}

onMounted(() => {
  if (props.station && props.streamUrl && audioRef.value) {
    startPlayback()
  }
})

onUnmounted(() => {
  resetAudioElement()
})

const startPlayback = async () => {
  if (!audioRef.value || !props.streamUrl) return

  isLoading.value = true
  error.value = null

  resetAudioElement()
  audioRef.value.src = getPlaybackUrl()
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
  if (props.streamUrl && audioRef.value?.currentSrc) {
    error.value = '无法加载音频流'
  }
}

const handleLoadedMetadata = () => {
  if (!audioRef.value) return
  duration.value = Number.isFinite(audioRef.value.duration) ? audioRef.value.duration : 0
}

const handleTimeUpdate = () => {
  if (!audioRef.value) return
  currentTime.value = audioRef.value.currentTime
}

const handleVolumeChange = () => {
  if (audioRef.value) {
    audioRef.value.volume = volume.value / 100
  }
}

const handleClose = () => {
  resetAudioElement()
  emit('close')
}
</script>

<template>
  <div v-if="station" :class="['player', { 'player-collapsed': collapsed }]">
    <audio
      ref="audioRef"
      preload="none"
      @play="handlePlay"
      @pause="handlePause"
      @error="handleError"
      @loadedmetadata="handleLoadedMetadata"
      @timeupdate="handleTimeUpdate"
    />

    <div class="player-info">
      <div class="station-cover">
        <img v-if="station.image" :src="station.image" :alt="station.name" />
        <span v-else>{{ stationInitials }}</span>
      </div>
      <div class="player-text">
        <div class="player-title-line">
          <span class="player-name">{{ station.name }}</span>
          <span class="live-badge">正在播放</span>
        </div>
        <div class="player-meta">
          <span>{{ station.province }}</span>
          <span>·</span>
          <span>国语</span>
          <span class="signal-mini"><i></i><i></i><i></i></span>
          <span>良好</span>
        </div>
      </div>
    </div>

    <div class="player-center">
      <div class="transport-controls">
        <button
          class="transport-button"
          type="button"
          title="上一个"
          :disabled="!canNavigate"
          @click="emit('previous')"
        >
          <SkipBack :size="22" :fill="'currentColor'" />
        </button>
        <button class="play-button" type="button" @click="togglePlay">
          <span v-if="isLoading" class="small-spinner"></span>
          <Pause v-else-if="isPlaying" :size="30" :fill="'currentColor'" />
          <Play v-else :size="28" :fill="'currentColor'" />
        </button>
        <button
          class="transport-button"
          type="button"
          title="下一个"
          :disabled="!canNavigate"
          @click="emit('next')"
        >
          <SkipForward :size="22" :fill="'currentColor'" />
        </button>
      </div>

      <div class="progress-row">
        <span>{{ elapsedText }}</span>
        <div class="progress-track">
          <span class="progress-fill" :style="{ width: `${progressValue}%` }"></span>
          <span class="progress-thumb" :style="{ left: `${progressValue}%` }"></span>
        </div>
        <span>{{ durationText }}</span>
      </div>

      <div v-if="error" class="error-message">{{ error }}</div>
    </div>

    <div class="player-actions">
      <Volume2 :size="20" />
      <input
        type="range"
        v-model.number="volume"
        min="0"
        max="100"
        class="volume-slider"
        :style="volumeTrackStyle"
        @input="handleVolumeChange"
      />
      <span class="volume-value">{{ volume }}%</span>
      <button
        class="plain-button"
        type="button"
        :title="collapsed ? '展开' : '收起'"
        @click="emit('toggle-collapse')"
      >
        <ChevronDown v-if="collapsed" :size="22" />
        <ChevronUp v-else :size="22" />
      </button>
      <button class="plain-button" type="button" title="关闭" @click="handleClose">
        <X :size="20" />
      </button>
    </div>

    <div class="mini-player-actions">
      <button class="mini-play-button" type="button" @click="togglePlay">
        <span v-if="isLoading" class="small-spinner"></span>
        <Pause v-else-if="isPlaying" :size="20" :fill="'currentColor'" />
        <Play v-else :size="19" :fill="'currentColor'" />
      </button>
      <button class="plain-button" type="button" title="展开" @click="emit('toggle-collapse')">
        <ChevronDown :size="21" />
      </button>
      <button class="plain-button" type="button" title="关闭" @click="handleClose">
        <X :size="19" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.player {
  min-height: 96px;
  display: grid;
  grid-column: 1 / -1;
  grid-template-columns: minmax(260px, 360px) minmax(360px, 1fr) minmax(240px, 320px);
  align-items: center;
  gap: 24px;
  padding: 12px 28px;
  background: var(--surface);
  border-top: 1px solid var(--border);
}

.player-collapsed {
  min-height: 58px;
  grid-template-columns: minmax(220px, 1fr) auto;
  gap: 16px;
  padding: 8px 24px;
}

.player-collapsed .station-cover {
  width: 42px;
  height: 42px;
  font-size: 0.78rem;
}

.player-collapsed .player-center,
.player-collapsed .player-actions {
  display: none;
}

.player-collapsed .player-meta {
  margin-top: 4px;
  font-size: 0.78rem;
}

.player-collapsed .player-name {
  font-size: 0.95rem;
}

.player-info {
  display: flex;
  align-items: center;
  gap: 16px;
  min-width: 0;
}

.station-cover {
  width: 70px;
  height: 70px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-muted);
  color: var(--danger);
  display: grid;
  place-items: center;
  flex: 0 0 auto;
  overflow: hidden;
  font-size: 1rem;
  font-weight: 850;
}

.station-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.player-text {
  min-width: 0;
}

.player-title-line {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.player-name {
  min-width: 0;
  color: var(--text-primary);
  font-size: 1.1rem;
  font-weight: 850;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.live-badge {
  flex: 0 0 auto;
  padding: 3px 8px;
  border-radius: 5px;
  background: var(--accent-soft);
  color: var(--accent-strong);
  font-size: 0.76rem;
  font-weight: 760;
}

.player-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.signal-mini {
  display: inline-flex;
  align-items: flex-end;
  gap: 2px;
  height: 16px;
  margin-left: 8px;
}

.signal-mini i {
  width: 4px;
  border-radius: 2px;
  background: var(--accent);
}

.signal-mini i:nth-child(1) {
  height: 7px;
}

.signal-mini i:nth-child(2) {
  height: 11px;
}

.signal-mini i:nth-child(3) {
  height: 16px;
}

.player-center {
  min-width: 0;
}

.transport-controls {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 34px;
}

.transport-button,
.play-button,
.plain-button {
  border: 0;
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.transport-button:disabled {
  cursor: not-allowed;
  opacity: 0.35;
}

.play-button {
  width: 56px;
  height: 56px;
  border: 1px solid var(--border);
  border-radius: 50%;
  background: var(--surface-raised);
  box-shadow: 0 3px 8px var(--shadow-soft);
}

.mini-player-actions {
  display: none;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
}

.player-collapsed .mini-player-actions {
  display: flex;
}

.mini-play-button {
  width: 34px;
  height: 34px;
  border: 1px solid var(--border);
  border-radius: 50%;
  background: var(--surface-raised);
  color: var(--text-primary);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.progress-row {
  display: grid;
  grid-template-columns: 48px minmax(180px, 1fr) 48px;
  align-items: center;
  gap: 14px;
  margin-top: 8px;
  color: var(--text-secondary);
  font-size: 0.84rem;
}

.progress-track {
  position: relative;
  height: 5px;
  border-radius: 999px;
  background: var(--border);
}

.progress-fill {
  position: absolute;
  inset: 0 auto 0 0;
  border-radius: inherit;
  background: var(--accent);
}

.progress-thumb {
  position: absolute;
  top: 50%;
  width: 18px;
  height: 18px;
  border: 2px solid var(--text-muted);
  border-radius: 50%;
  background: var(--surface-raised);
  transform: translate(-50%, -50%);
}

.player-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 14px;
  color: var(--text-secondary);
}

.volume-slider {
  width: 130px;
  height: 5px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--border);
  border-radius: 999px;
  outline: none;
}

.volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 18px;
  height: 18px;
  border: 2px solid var(--text-muted);
  border-radius: 50%;
  background: var(--surface-raised);
  cursor: pointer;
}

.volume-value {
  min-width: 42px;
  color: var(--text-secondary);
  font-size: 0.86rem;
}

.small-spinner {
  width: 22px;
  height: 22px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.error-message {
  margin-top: 6px;
  text-align: center;
  color: var(--danger);
  font-size: 0.82rem;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 1100px) {
  .player {
    grid-template-columns: minmax(240px, 1fr) minmax(300px, 1.2fr);
    row-gap: 10px;
  }

  .player-actions {
    display: none;
  }

  .player-collapsed {
    grid-template-columns: 1fr auto;
  }
}

@media (max-width: 940px) {
  .player {
    grid-template-columns: 1fr;
    gap: 14px;
    padding: 12px 18px;
  }

  .transport-controls {
    gap: 22px;
  }

  .player-collapsed {
    grid-template-columns: minmax(0, 1fr) auto;
  }
}
</style>

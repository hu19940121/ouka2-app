<script setup lang="ts">
import { computed } from 'vue'
import { Copy, MoreVertical, Play, RadioTower, Trash2 } from 'lucide-vue-next'
import type { Station } from '../types'

const props = defineProps<{
  station: Station
  serverPort: number
  selected: boolean
  serverRunning: boolean
  isCurrent: boolean
}>()

const emit = defineEmits<{
  play: [station: Station]
  copy: [url: string]
  delete: [station: Station]
  'toggle-selection': [station: Station]
}>()

const streamUrl = computed(() => {
  return `http://127.0.0.1:${props.serverPort}/stream/${props.station.id}`
})

const signalText = computed(() => {
  if (props.station.is_custom) return '自定义'
  if (props.station.subtitle) return props.station.subtitle
  return '可用'
})

const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(streamUrl.value)
    emit('copy', streamUrl.value)
  } catch (e) {
    console.error('复制失败:', e)
  }
}
</script>

<template>
  <div :class="['station-row', { 'is-selected': selected, 'is-current': isCurrent }]">
    <label class="station-check" :title="selected ? '从安装队列移除' : '加入安装队列'">
      <input
        type="checkbox"
        :checked="selected"
        @change="emit('toggle-selection', station)"
      />
      <span class="check-visual"></span>
    </label>

    <div class="station-main">
      <div class="station-title-row">
        <h3 class="station-name">{{ station.name }}</h3>
        <span v-if="isCurrent" class="playing-badge">
          <RadioTower :size="14" />
          正在播放
        </span>
      </div>
      <p class="station-subtitle">{{ signalText }}</p>
    </div>

    <span :class="['province-tag', { 'is-custom': station.is_custom }]">
      {{ station.province }}
    </span>

    <span class="language-cell">国语</span>

    <div class="station-quality" aria-label="信号状态">
      <span class="signal-bars">
        <i></i>
        <i></i>
        <i></i>
      </span>
      <span>{{ station.is_custom ? '手动源' : '良好' }}</span>
    </div>

    <div class="station-actions">
      <button
        class="icon-button primary"
        type="button"
        :disabled="!serverRunning"
        :title="serverRunning ? '播放' : '请先启动服务器'"
        @click="emit('play', station)"
      >
        <Play :size="16" :fill="'currentColor'" />
      </button>
      <button class="icon-button" type="button" title="复制地址" @click="handleCopy">
        <Copy :size="17" />
      </button>
      <button
        v-if="station.is_custom"
        class="icon-button danger"
        type="button"
        title="删除自定义电台"
        @click="emit('delete', station)"
      >
        <Trash2 :size="16" />
      </button>
      <button v-else class="icon-button ghost" type="button" title="更多">
        <MoreVertical :size="17" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.station-row {
  display: grid;
  grid-template-columns: var(--station-table-columns);
  align-items: center;
  gap: var(--station-table-gap);
  min-width: var(--station-table-min-width);
  min-height: 58px;
  padding: 0 18px;
  border-bottom: 1px solid #e6e9ee;
  background: #ffffff;
  color: #151923;
  transition: background 0.18s ease;
}

.station-row:hover {
  background: #f8faf8;
}

.station-row.is-selected {
  background: #fbfdfb;
}

.station-row.is-current {
  background: #f3faf4;
}

.station-check {
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.station-check input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.check-visual {
  width: 18px;
  height: 18px;
  border: 1px solid #c8ced8;
  border-radius: 4px;
  background: #fff;
  transition: all 0.16s ease;
}

.station-check input:checked + .check-visual {
  border-color: #2f9e55;
  background: #2f9e55;
  box-shadow: inset 0 0 0 4px #2f9e55;
}

.station-check input:checked + .check-visual::after {
  content: '';
  display: block;
  width: 8px;
  height: 5px;
  margin: 4px 0 0 4px;
  border-left: 2px solid white;
  border-bottom: 2px solid white;
  transform: rotate(-45deg);
}

.station-main {
  min-width: 0;
}

.station-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.station-name {
  min-width: 0;
  margin: 0;
  overflow: hidden;
  color: #161a22;
  font-size: 0.96rem;
  font-weight: 760;
  letter-spacing: 0;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.province-tag {
  display: inline-flex;
  align-items: center;
  width: fit-content;
  max-width: 108px;
  padding: 0;
  overflow: hidden;
  border: 0;
  border-radius: 0;
  background: transparent;
  color: #2f3642;
  flex-shrink: 0;
  font-size: 0.9rem;
  font-weight: 560;
  line-height: 1.35;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.province-tag.is-custom {
  color: #315f9d;
}

.playing-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border-radius: 5px;
  background: #e4f3e8;
  color: #2b874c;
  flex: 0 0 auto;
  font-size: 0.76rem;
  font-weight: 700;
}

.station-subtitle {
  margin: 4px 0 0;
  overflow: hidden;
  color: #7b8492;
  font-size: 0.8rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.language-cell {
  color: #2f3642;
  font-size: 0.9rem;
}

.station-quality {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 8px;
  color: #333b48;
  font-size: 0.88rem;
  white-space: nowrap;
}

.signal-bars {
  display: inline-flex;
  align-items: flex-end;
  gap: 2px;
  height: 16px;
}

.signal-bars i {
  display: block;
  width: 4px;
  border-radius: 2px;
  background: #31a354;
}

.signal-bars i:nth-child(1) {
  height: 6px;
}

.signal-bars i:nth-child(2) {
  height: 10px;
}

.signal-bars i:nth-child(3) {
  height: 15px;
}

.station-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
}

.icon-button {
  width: 32px;
  height: 32px;
  border: 1px solid #dce1e8;
  border-radius: 50%;
  background: #fff;
  color: #202633;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  transition: all 0.16s ease;
}

.icon-button:hover:not(:disabled) {
  border-color: #b9c2cf;
  background: #f5f7fa;
  transform: translateY(-1px);
}

.icon-button:disabled {
  opacity: 0.42;
  cursor: not-allowed;
}

.icon-button.primary {
  border-color: #cfd7e3;
}

.icon-button.ghost {
  border-color: transparent;
  background: transparent;
}

.icon-button.danger {
  color: #c33f3f;
}

.icon-button.danger:hover {
  border-color: #f0c6c6;
  background: #fff5f5;
}

@media (max-width: 1040px) {
  .station-row {
    grid-template-columns: var(--station-table-columns);
  }

}
</style>

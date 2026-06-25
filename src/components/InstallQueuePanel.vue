<script setup lang="ts">
import { computed } from 'vue'
import { NButton, NEmpty, NScrollbar, NTag, NTooltip } from 'naive-ui'
import { ChevronRight, Copy, Download, GripVertical, Square, X } from 'lucide-vue-next'
import type { ServerStatus, Station } from '../types'

const props = defineProps<{
  status: ServerStatus
  stationCount: number
  selectedStations: Station[]
  selectedStationCount: number
  ffmpegStatus: string | null
  logCount: number
  errorLogCount: number
  isCrawling: boolean
  isInstalling: boolean
}>()

const emit = defineEmits<{
  start: []
  stop: []
  crawl: []
  install: []
  logs: []
  manage: []
  clear: []
  remove: [stationId: string]
}>()

const visibleStations = computed(() => props.selectedStations)
const serverType = computed(() => (props.status.running ? 'success' : 'error'))
const serverText = computed(() => (props.status.running ? '运行中' : '已停止'))
</script>

<template>
  <aside class="queue-panel">
    <section class="panel-section">
      <div class="panel-block server-section">
        <div class="section-heading">
          <h2>本地服务器</h2>
          <NTag :type="serverType" round size="small">{{ serverText }}</NTag>
        </div>

        <div class="server-rows">
          <div class="server-row">
            <span>端口</span>
            <strong>{{ status.port }}</strong>
          </div>
          <div class="server-row">
            <span>访问地址</span>
            <strong>http://localhost:{{ status.port }}</strong>
            <Copy :size="18" class="row-icon" />
          </div>
          <div class="server-row">
            <span>启动时间</span>
            <strong>{{ status.running ? '当前会话' : '--' }}</strong>
          </div>
        </div>

        <NButton
          v-if="!status.running"
          type="primary"
          secondary
          block
          strong
          class="server-primary-action"
          @click="emit('start')"
        >
          启动服务器
        </NButton>
        <NButton
          v-else
          type="error"
          secondary
          block
          strong
          class="server-primary-action"
          @click="emit('stop')"
        >
          <template #icon>
            <Square :size="16" :fill="'currentColor'" />
          </template>
          停止服务器
        </NButton>

        <p v-if="!ffmpegStatus" class="ffmpeg-warning">
          FFmpeg 未检测到，请确认运行环境已包含 FFmpeg。
        </p>
      </div>

      <div class="panel-block connections-section">
        <div class="section-heading compact">
          <h2>活动连接 ({{ status.active_streams }})</h2>
          <button class="text-link" type="button" @click="emit('logs')">
            更多
            <ChevronRight :size="16" />
          </button>
        </div>

        <div class="connection-list">
          <div class="connection-row">
            <span class="dot"></span>
            <span>ETS2</span>
            <strong>127.0.0.1:56012</strong>
          </div>
          <div class="connection-row">
            <span class="dot"></span>
            <span>浏览器</span>
            <strong>127.0.0.1:56013</strong>
          </div>
        </div>
      </div>

      <div class="panel-block install-section">
        <div class="section-heading compact">
          <h2>安装队列 ({{ selectedStationCount }})</h2>
          <NTooltip trigger="hover">
            <template #trigger>
              <NButton text type="error" :disabled="selectedStationCount === 0" @click="emit('clear')">
                清空
              </NButton>
            </template>
            清空当前安装队列
          </NTooltip>
        </div>

        <NScrollbar class="queue-scroll">
          <div v-if="visibleStations.length > 0" class="queue-list">
            <div v-for="station in visibleStations" :key="station.id" class="queue-item">
              <GripVertical :size="17" class="drag-icon" />
              <div class="queue-item-main">
                <strong>{{ station.name }}</strong>
                <span>{{ station.province }} · {{ station.is_custom ? '自定义' : 'MP3 128kbps' }}</span>
              </div>
              <button class="remove-button" type="button" title="移出队列" @click="emit('remove', station.id)">
                <X :size="18" />
              </button>
            </div>
          </div>
          <NEmpty v-else description="还没有选择电台" size="small" class="queue-empty" />
        </NScrollbar>

        <NButton
          type="primary"
          block
          strong
          class="install-button"
          :loading="isInstalling"
          :disabled="stationCount === 0 || selectedStationCount === 0"
          @click="emit('install')"
        >
          <template #icon>
            <Download :size="20" />
          </template>
          安装选中电台（{{ selectedStationCount }}）
        </NButton>
      </div>
    </section>
  </aside>
</template>

<style scoped>
.queue-panel {
  min-width: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.panel-section {
  height: 100%;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-block {
  padding: 14px 16px;
  border-bottom: 1px solid var(--border);
}

.panel-block:last-child {
  border-bottom: 0;
}

.section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.section-heading.compact {
  align-items: center;
}

.section-heading h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: 0.98rem;
  font-weight: 800;
  letter-spacing: 0;
}

.server-rows {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin: 14px 0;
}

.server-row {
  display: grid;
  grid-template-columns: 64px minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  color: var(--text-strong);
}

.server-row span {
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.server-row strong {
  min-width: 0;
  color: var(--text-strong);
  font-size: 0.9rem;
  font-weight: 520;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-icon {
  color: var(--text-secondary);
}

.server-primary-action {
  margin-top: 10px;
}

.ffmpeg-warning {
  margin: 12px 0 0;
  padding: 10px 12px;
  border: 1px solid var(--warning-border);
  border-radius: 6px;
  background: var(--warning-soft);
  color: var(--warning);
  font-size: 0.78rem;
  line-height: 1.45;
}

.text-link {
  border: 0;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 0;
  font-size: 0.86rem;
}

.connection-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 14px;
}

.connection-row {
  display: grid;
  grid-template-columns: 12px 1fr auto;
  align-items: center;
  gap: 12px;
  color: var(--text-secondary);
  font-size: 0.92rem;
}

.connection-row strong {
  color: var(--text-secondary);
  font-weight: 520;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent);
}

.install-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.install-section .section-heading {
  flex: 0 0 auto;
  min-height: 30px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-soft);
}

.queue-scroll {
  min-height: 0;
  flex: 1;
  margin-top: 10px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-soft);
  overflow: hidden;
}

.queue-scroll :deep(.n-scrollbar-container) {
  border-radius: inherit;
}

.queue-scroll :deep(.n-scrollbar-content) {
  border-radius: inherit;
}

.queue-list {
  display: flex;
  flex-direction: column;
}

.queue-item {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 52px;
  padding: 8px;
  border-top: 1px solid var(--border);
  background: var(--surface-soft);
}

.queue-item:first-child {
  border-top: 0;
}

.drag-icon {
  color: var(--text-muted);
  flex: 0 0 auto;
}

.queue-item-main {
  min-width: 0;
  flex: 1;
}

.queue-item-main strong,
.queue-item-main span {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.queue-item-main strong {
  color: var(--text-primary);
  font-size: 0.86rem;
}

.queue-item-main span {
  margin-top: 3px;
  color: var(--text-muted);
  font-size: 0.75rem;
}

.remove-button {
  width: 26px;
  height: 26px;
  border: 0;
  border-radius: 50%;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  flex: 0 0 auto;
}

.remove-button:hover {
  background: var(--danger-soft);
  color: var(--danger);
}

.queue-empty {
  padding-top: 30px;
}

.install-button {
  flex: 0 0 auto;
  margin-top: 14px;
  min-height: 42px;
}

@media (max-width: 1280px) {
  .queue-panel {
    height: auto;
    overflow: visible;
  }

  .panel-section {
    height: auto;
  }

  .install-section {
    min-height: 280px;
  }

  .queue-scroll {
    max-height: 320px;
  }
}
</style>

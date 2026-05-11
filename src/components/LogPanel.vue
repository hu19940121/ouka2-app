<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { DiagnosticLogEntry } from '../types'

const props = defineProps<{
  visible: boolean
  logs: DiagnosticLogEntry[]
}>()

const emit = defineEmits<{
  close: []
  clear: []
}>()

const levelFilter = ref<'all' | DiagnosticLogEntry['level']>('all')
const moduleFilter = ref('all')
const copied = ref(false)
const logBodyRef = ref<HTMLElement | null>(null)

const modules = computed(() => {
  return Array.from(new Set(props.logs.map(log => log.module))).sort()
})

const filteredLogs = computed(() => {
  return props.logs.filter((log) => {
    const matchesLevel = levelFilter.value === 'all' || log.level === levelFilter.value
    const matchesModule = moduleFilter.value === 'all' || log.module === moduleFilter.value
    return matchesLevel && matchesModule
  })
})

const latestLevel = computed(() => props.logs[props.logs.length - 1]?.level ?? 'info')

const formatLog = (log: DiagnosticLogEntry) => {
  const station = log.stationName ? ` ${log.stationName}` : ''
  const detail = log.detail ? `\n    ${log.detail}` : ''
  return `[${log.time}] [${log.level.toUpperCase()}] [${log.module}]${station} ${log.message}${detail}`
}

const copyLogs = async () => {
  await navigator.clipboard.writeText(filteredLogs.value.map(formatLog).join('\n'))
  copied.value = true
  setTimeout(() => {
    copied.value = false
  }, 1500)
}

watch(
  () => props.logs.length,
  () => {
    requestAnimationFrame(() => {
      if (logBodyRef.value) {
        logBodyRef.value.scrollTop = logBodyRef.value.scrollHeight
      }
    })
  }
)
</script>

<template>
  <Transition name="log-panel">
    <aside v-if="visible" class="log-panel">
      <div class="log-header">
        <div>
          <p class="eyebrow">实时诊断</p>
          <h2>运行日志</h2>
        </div>
        <button class="icon-button" @click="emit('close')" title="关闭日志">×</button>
      </div>

      <div class="log-summary">
        <div :class="['summary-pulse', `summary-${latestLevel}`]"></div>
        <span>{{ logs.length }} 条记录</span>
        <span>{{ filteredLogs.length }} 条匹配</span>
      </div>

      <div class="log-tools">
        <select v-model="levelFilter" class="tool-select">
          <option value="all">全部级别</option>
          <option value="error">错误</option>
          <option value="warn">警告</option>
          <option value="info">信息</option>
          <option value="debug">调试</option>
        </select>

        <select v-model="moduleFilter" class="tool-select">
          <option value="all">全部模块</option>
          <option v-for="module in modules" :key="module" :value="module">
            {{ module }}
          </option>
        </select>

        <button class="tool-button" @click="copyLogs">
          {{ copied ? '已复制' : '复制' }}
        </button>
        <button class="tool-button danger" @click="emit('clear')">清空</button>
      </div>

      <div ref="logBodyRef" class="log-body">
        <div v-if="filteredLogs.length === 0" class="empty-log">
          暂无匹配日志
        </div>

        <article
          v-for="(log, index) in filteredLogs"
          :key="`${log.time}-${index}-${log.message}`"
          :class="['log-row', `log-${log.level}`]"
        >
          <div class="log-meta">
            <span class="log-time">{{ log.time }}</span>
            <span class="log-level">{{ log.level }}</span>
            <span class="log-module">{{ log.module }}</span>
          </div>
          <div class="log-message">
            {{ log.message }}
            <span v-if="log.stationName" class="station-name">{{ log.stationName }}</span>
          </div>
          <pre v-if="log.detail" class="log-detail">{{ log.detail }}</pre>
        </article>
      </div>
    </aside>
  </Transition>
</template>

<style scoped>
.log-panel {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  width: min(620px, 92vw);
  z-index: 180;
  display: flex;
  flex-direction: column;
  background: rgba(11, 13, 30, 0.96);
  border-left: 1px solid rgba(125, 211, 252, 0.22);
  box-shadow: -24px 0 60px rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(18px);
}

.log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.35rem 1rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.eyebrow {
  margin: 0 0 0.25rem;
  color: #7dd3fc;
  font-size: 0.75rem;
  letter-spacing: 0;
}

.log-header h2 {
  margin: 0;
  color: #fff;
  font-size: 1.25rem;
}

.icon-button {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.82);
  cursor: pointer;
  font-size: 1.35rem;
  line-height: 1;
}

.icon-button:hover {
  background: rgba(255, 255, 255, 0.16);
}

.log-summary {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.85rem 1.35rem;
  color: rgba(255, 255, 255, 0.68);
  font-size: 0.86rem;
}

.summary-pulse {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: #7dd3fc;
  box-shadow: 0 0 18px currentColor;
}

.summary-error {
  background: #f87171;
}

.summary-warn {
  background: #fbbf24;
}

.summary-info,
.summary-debug {
  background: #7dd3fc;
}

.log-tools {
  display: grid;
  grid-template-columns: 1fr 1fr auto auto;
  gap: 0.6rem;
  padding: 0 1.35rem 1rem;
}

.tool-select,
.tool-button {
  height: 36px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.07);
  color: rgba(255, 255, 255, 0.9);
  outline: none;
}

.tool-select {
  padding: 0 0.65rem;
}

.tool-select option {
  background: #14172f;
  color: #fff;
}

.tool-button {
  padding: 0 0.85rem;
  cursor: pointer;
}

.tool-button:hover {
  background: rgba(255, 255, 255, 0.14);
}

.tool-button.danger {
  color: #fecaca;
  border-color: rgba(248, 113, 113, 0.3);
}

.log-body {
  flex: 1;
  overflow-y: auto;
  padding: 0.25rem 1.35rem 1.35rem;
}

.log-row {
  padding: 0.85rem 0;
  border-top: 1px solid rgba(255, 255, 255, 0.07);
}

.log-row:first-child {
  border-top: none;
}

.log-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.35rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 0.72rem;
  color: rgba(255, 255, 255, 0.48);
}

.log-level,
.log-module {
  padding: 0.08rem 0.38rem;
  border-radius: 5px;
  background: rgba(255, 255, 255, 0.08);
}

.log-error .log-level {
  color: #fecaca;
  background: rgba(248, 113, 113, 0.2);
}

.log-warn .log-level {
  color: #fde68a;
  background: rgba(251, 191, 36, 0.18);
}

.log-message {
  color: rgba(255, 255, 255, 0.88);
  font-size: 0.92rem;
  line-height: 1.45;
}

.station-name {
  display: inline-block;
  margin-left: 0.5rem;
  color: #7dd3fc;
}

.log-detail {
  margin-top: 0.55rem;
  padding: 0.65rem;
  white-space: pre-wrap;
  word-break: break-word;
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.26);
  color: rgba(255, 255, 255, 0.66);
  font-size: 0.76rem;
  line-height: 1.45;
}

.empty-log {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 180px;
  color: rgba(255, 255, 255, 0.45);
}

.log-panel-enter-active,
.log-panel-leave-active {
  transition: transform 0.25s ease, opacity 0.25s ease;
}

.log-panel-enter-from,
.log-panel-leave-to {
  opacity: 0;
  transform: translateX(24px);
}

@media (max-width: 720px) {
  .log-tools {
    grid-template-columns: 1fr 1fr;
  }
}
</style>

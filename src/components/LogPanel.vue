<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { NButton, NEmpty, NSelect, NTag, type SelectOption } from 'naive-ui'
import { Copy, Trash2, X } from 'lucide-vue-next'
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
const levelOptions: SelectOption[] = [
  { label: '全部级别', value: 'all' },
  { label: '错误', value: 'error' },
  { label: '警告', value: 'warn' },
  { label: '信息', value: 'info' },
  { label: '调试', value: 'debug' },
]
const moduleOptions = computed<SelectOption[]>(() => [
  { label: '全部模块', value: 'all' },
  ...modules.value.map((module) => ({ label: module, value: module })),
])

const filteredLogs = computed(() => {
  return props.logs.filter((log) => {
    const matchesLevel = levelFilter.value === 'all' || log.level === levelFilter.value
    const matchesModule = moduleFilter.value === 'all' || log.module === moduleFilter.value
    return matchesLevel && matchesModule
  })
})

const latestLevel = computed(() => props.logs[props.logs.length - 1]?.level ?? 'info')
const latestType = computed(() => {
  if (latestLevel.value === 'error') return 'error'
  if (latestLevel.value === 'warn') return 'warning'
  return 'success'
})

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
          <h2>运行日志</h2>
          <p>查看服务器、抓取和安装过程中的诊断信息</p>
        </div>
        <NButton quaternary circle @click="emit('close')" title="关闭日志">
          <template #icon>
            <X :size="18" />
          </template>
        </NButton>
      </div>

      <div class="log-summary">
        <NTag :type="latestType" round size="small">{{ latestLevel.toUpperCase() }}</NTag>
        <span>{{ logs.length }} 条记录</span>
        <span>{{ filteredLogs.length }} 条匹配</span>
      </div>

      <div class="log-tools">
        <NSelect v-model:value="levelFilter" :options="levelOptions" />
        <NSelect v-model:value="moduleFilter" :options="moduleOptions" />
        <NButton secondary @click="copyLogs">
          <template #icon>
            <Copy :size="16" />
          </template>
          {{ copied ? '已复制' : '复制' }}
        </NButton>
        <NButton secondary type="error" @click="emit('clear')">
          <template #icon>
            <Trash2 :size="16" />
          </template>
          清空
        </NButton>
      </div>

      <div ref="logBodyRef" class="log-body">
        <NEmpty v-if="filteredLogs.length === 0" description="暂无匹配日志" class="empty-log" />

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
  background: var(--surface);
  border-left: 1px solid var(--border);
  box-shadow: -18px 0 48px var(--shadow-panel);
}

.log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 18px 20px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-soft);
}

.log-header h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: 1.08rem;
  font-weight: 800;
}

.log-header p {
  margin: 4px 0 0;
  color: var(--text-secondary);
  font-size: 0.82rem;
}

.log-summary {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 20px;
  color: var(--text-secondary);
  font-size: 0.86rem;
}

.log-tools {
  display: grid;
  grid-template-columns: 1fr 1fr auto auto;
  gap: 8px;
  padding: 0 20px 14px;
  border-bottom: 1px solid var(--border-soft);
}

.log-body {
  flex: 1;
  overflow-y: auto;
  padding: 0 20px 20px;
  scrollbar-width: thin;
  scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
}

.log-row {
  padding: 12px 0;
  border-top: 1px solid var(--border-soft);
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
  color: var(--text-muted);
}

.log-level,
.log-module {
  padding: 2px 6px;
  border-radius: 5px;
  background: var(--surface-muted);
  color: var(--text-secondary);
  font-weight: 700;
}

.log-error .log-level {
  color: var(--danger);
  background: var(--danger-soft);
}

.log-warn .log-level {
  color: var(--warning);
  background: var(--warning-soft);
}

.log-message {
  color: var(--text-strong);
  font-size: 0.9rem;
  line-height: 1.45;
}

.station-name {
  display: inline-block;
  margin-left: 0.5rem;
  color: var(--accent-strong);
  font-weight: 700;
}

.log-detail {
  margin-top: 0.55rem;
  padding: 0.65rem;
  white-space: pre-wrap;
  word-break: break-word;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-muted);
  color: var(--text-secondary);
  font-size: 0.76rem;
  line-height: 1.45;
}

.empty-log {
  padding-top: 80px;
  height: 180px;
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

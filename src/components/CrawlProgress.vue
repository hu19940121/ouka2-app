<script setup lang="ts">
import { computed } from 'vue'
import { Radio, RefreshCw } from 'lucide-vue-next'
import type { CrawlProgress } from '../types'

const props = defineProps<{
  progress: CrawlProgress
}>()

const percentage = computed(() => {
  return Math.round((props.progress.current / Math.max(props.progress.total, 1)) * 100)
})
</script>

<template>
  <div class="crawl-overlay">
    <div class="crawl-modal">
      <div class="crawl-header">
        <span class="crawl-icon">
          <RefreshCw :size="22" />
        </span>
        <div>
          <h2>正在爬取电台数据</h2>
          <p>请保持应用打开，数据会自动写入当前列表。</p>
        </div>
      </div>

      <div class="crawl-content">
        <div class="progress-info">
          <span>正在获取: {{ progress.province }}</span>
          <span>{{ progress.current }} / {{ progress.total }}</span>
        </div>

        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: percentage + '%' }"></div>
        </div>

        <div class="stats">
          <span class="stat-icon">
            <Radio :size="18" />
          </span>
          <span>
            已发现 <strong>{{ progress.stations_found }}</strong> 个电台
          </span>
        </div>
      </div>

      <p class="crawl-tip">请稍候，这可能需要几分钟...</p>
    </div>
  </div>
</template>

<style scoped>
.crawl-overlay {
  position: fixed;
  inset: 0;
  background: var(--overlay);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 180;
  padding: 24px;
}

.crawl-modal {
  width: min(440px, 100%);
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
  box-shadow: 0 24px 70px var(--shadow-modal);
  padding: 22px;
}

.crawl-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 18px;
}

.crawl-icon {
  width: 42px;
  height: 42px;
  border-radius: 8px;
  background: var(--accent-softer);
  color: var(--accent);
  display: grid;
  place-items: center;
  flex: 0 0 auto;
  animation: spin 2s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.crawl-header h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: 1.08rem;
  font-weight: 800;
}

.crawl-header p {
  margin: 4px 0 0;
  color: var(--text-secondary);
  font-size: 0.82rem;
  line-height: 1.45;
}

.crawl-content {
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface-soft);
  margin-bottom: 14px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
  color: var(--text-secondary);
  font-size: 0.86rem;
  font-weight: 700;
}

.progress-bar {
  height: 8px;
  background: var(--border);
  border-radius: 999px;
  overflow: hidden;
  margin-bottom: 14px;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: inherit;
  transition: width 0.3s ease;
}

.stats {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.stat-icon {
  color: var(--accent);
  display: inline-flex;
}

.stats strong {
  color: var(--accent);
  font-size: 1.08rem;
}

.crawl-tip {
  text-align: center;
  color: var(--text-muted);
  font-size: 0.82rem;
  margin: 0;
}
</style>

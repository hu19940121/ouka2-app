<script setup lang="ts">
import type { CrawlProgress } from '../types'

const props = defineProps<{
  progress: CrawlProgress
}>()

const percentage = Math.round((props.progress.current / Math.max(props.progress.total, 1)) * 100)
</script>

<template>
  <div class="crawl-overlay">
    <div class="crawl-modal">
      <div class="crawl-header">
        <span class="crawl-icon">🔄</span>
        <h2>正在爬取电台数据</h2>
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
          <span class="stat-item">
            📻 已发现 <strong>{{ progress.stations_found }}</strong> 个电台
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
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.crawl-modal {
  background: linear-gradient(135deg, #1a1a2e, #16213e);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 20px;
  padding: 2rem;
  width: 400px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

.crawl-header {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  margin-bottom: 1.5rem;
}

.crawl-icon {
  font-size: 2rem;
  animation: spin 2s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.crawl-header h2 {
  margin: 0;
  font-size: 1.3rem;
  color: #fff;
}

.crawl-content {
  margin-bottom: 1rem;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 0.5rem;
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.8);
}

.progress-bar {
  height: 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 1rem;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #4facfe, #00f2fe);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.stats {
  text-align: center;
}

.stat-item {
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.95rem;
}

.stat-item strong {
  color: #4facfe;
  font-size: 1.2rem;
}

.crawl-tip {
  text-align: center;
  color: rgba(255, 255, 255, 0.5);
  font-size: 0.85rem;
  margin: 0;
}
</style>

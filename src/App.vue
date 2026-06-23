<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import {
  NButton,
  NCheckbox,
  NConfigProvider,
  NDataTable,
  NInput,
  NSelect,
  NSwitch,
  NTag,
  NTooltip,
  zhCN,
  type DataTableColumns,
  type GlobalThemeOverrides,
} from 'naive-ui'
import {
  Copy,
  FileText,
  Info,
  ListMusic,
  Plus,
  Play,
  RadioTower,
  RefreshCw,
  Search,
  Settings,
  Trash2,
  Upload,
} from 'lucide-vue-next'
import { CUSTOM_STATION_FILTER, useRadioStore } from './stores/radio'
import AudioPlayer from './components/AudioPlayer.vue'
import CrawlProgress from './components/CrawlProgress.vue'
import InstallStationsDialog from './components/InstallStationsDialog.vue'
import InstallQueuePanel from './components/InstallQueuePanel.vue'
import LogPanel from './components/LogPanel.vue'
import appLogo from './assets/app-logo-ui.png'
import type { Station } from './types'

const store = useRadioStore()
const appVersion = __APP_VERSION__
const DEBUG_MODE_STORAGE_KEY = 'ouka2-debug-mode'

const currentStation = ref<Station | null>(null)
const playerKey = ref(0)
const toast = ref<{ message: string; type: 'success' | 'error' | 'info' } | null>(null)

const showCustomDialog = ref(false)
const customName = ref('')
const customUrl = ref('')
const isAddingCustom = ref(false)
const showInstallDialog = ref(false)
const isInstalling = ref(false)
const showLogPanel = ref(false)
const showSettingsDialog = ref(false)
const showAboutDialog = ref(false)
const isPlayerCollapsed = ref(false)
const isDebugMode = ref(localStorage.getItem(DEBUG_MODE_STORAGE_KEY) === 'true')

const naiveThemeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#2f9e55',
    primaryColorHover: '#268646',
    primaryColorPressed: '#1f6f3a',
    primaryColorSuppl: '#2f9e55',
    borderRadius: '8px',
    borderRadiusSmall: '6px',
  },
  Button: {
    heightLarge: '40px',
    heightMedium: '34px',
    borderRadiusLarge: '6px',
    borderRadiusMedium: '6px',
    textColor: '#202633',
    textColorHover: '#1f6f3a',
    border: '1px solid #e1e5eb',
    borderHover: '1px solid #d4dbe4',
  },
  Input: {
    heightLarge: '42px',
    borderRadius: '6px',
    border: '1px solid #e1e5eb',
    borderHover: '1px solid #d4dbe4',
    borderFocus: '1px solid #c8d2df',
    boxShadowFocus: '0 0 0 3px rgba(47, 158, 85, 0.08)',
    color: '#ffffff',
    placeholderColor: '#a0a7b3',
  },
  Select: {
    peers: {
      InternalSelection: {
        heightLarge: '42px',
        borderRadius: '6px',
        border: '1px solid #e1e5eb',
        borderHover: '1px solid #d4dbe4',
        borderFocus: '1px solid #c8d2df',
        boxShadowFocus: '0 0 0 3px rgba(47, 158, 85, 0.08)',
      },
    },
  },
  Tag: {
    borderRadius: '999px',
  },
  DataTable: {
    thColor: '#fbfcfd',
    thTextColor: '#4f5968',
    tdColor: '#ffffff',
    tdColorHover: '#f7faf8',
    borderColor: '#e4e8ef',
    thFontWeight: '700',
    thPaddingMedium: '9px 12px',
    tdPaddingMedium: '8px 12px',
  },
}

const provinceOptions = computed(() => [
  { label: '全部地区', value: '' },
  { label: '自定义电台', value: CUSTOM_STATION_FILTER },
  ...store.provinces.map((province) => ({ label: province, value: province })),
])

const selectedStationIdSet = computed(() => new Set(store.selectedStationIds))
const filteredStationIds = computed(() => store.filteredStations.map((station) => station.id))
const allFilteredSelected = computed(() =>
  filteredStationIds.value.length > 0 &&
  filteredStationIds.value.every((id) => selectedStationIdSet.value.has(id))
)
const someFilteredSelected = computed(() =>
  filteredStationIds.value.some((id) => selectedStationIdSet.value.has(id))
)

const currentStreamUrl = computed(() => {
  if (!currentStation.value) return ''
  return store.getStreamUrl(currentStation.value.id)
})

const serverStatusType = computed(() => (store.serverStatus.running ? 'success' : 'error'))
const serverStatusText = computed(() => (store.serverStatus.running ? '运行中' : '已停止'))

const showToast = (message: string, type: 'success' | 'error' | 'info' = 'info') => {
  toast.value = { message, type }
  setTimeout(() => {
    toast.value = null
  }, 3000)
}

const isStationSelected = (station: Station) => selectedStationIdSet.value.has(station.id)

const stationTableColumns = computed<DataTableColumns<Station>>(() => [
  {
    title: () => h(NCheckbox, {
      checked: allFilteredSelected.value,
      indeterminate: someFilteredSelected.value && !allFilteredSelected.value,
      disabled: store.filteredStations.length === 0,
      'onUpdate:checked': () => handleToggleAllFiltered(),
    }),
    key: 'selection',
    width: 48,
    fixed: 'left',
    render: (station) => h(NCheckbox, {
      checked: isStationSelected(station),
      'onUpdate:checked': () => handleToggleStationSelection(station),
    }),
  },
  {
    title: '电台名称',
    key: 'name',
    minWidth: 300,
    render: (station) => h('div', { class: 'station-name-cell' }, [
      h('div', { class: 'station-title-line' }, [
        h('span', { class: 'station-title' }, station.name),
        currentStation.value?.id === station.id
          ? h('span', { class: 'playing-badge' }, [
              h(RadioTower, { size: 14 }),
              '正在播放',
            ])
          : null,
      ]),
      h('div', { class: 'station-subtitle' }, station.subtitle || (station.is_custom ? '自定义电台' : '可用')),
    ]),
  },
  {
    title: '地区',
    key: 'province',
    width: 96,
    ellipsis: { tooltip: true },
  },
  {
    title: '操作',
    key: 'actions',
    width: 92,
    align: 'right',
    render: (station) => h('div', { class: 'station-actions-cell' }, [
      h('button', {
        class: 'table-icon-button primary',
        type: 'button',
        disabled: !store.serverStatus.running,
        title: store.serverStatus.running ? '播放' : '请先启动服务器',
        onClick: () => handlePlay(station),
      }, h(Play, { size: 16, fill: 'currentColor' })),
      h('button', {
        class: 'table-icon-button',
        type: 'button',
        title: '复制地址',
        onClick: () => handleCopyStationUrl(station),
      }, h(Copy, { size: 17 })),
      station.is_custom
        ? h('button', {
            class: 'table-icon-button danger',
            type: 'button',
            title: '删除自定义电台',
            onClick: () => handleDeleteStation(station),
          }, h(Trash2, { size: 16 }))
        : null,
    ]),
  },
])

const handleToggleStationSelection = async (station: Station) => {
  await store.toggleStationSelection(station.id)
}

const handleToggleAllFiltered = async () => {
  const filteredIds = filteredStationIds.value
  if (filteredIds.length === 0) return

  if (allFilteredSelected.value) {
    const filteredIdSet = new Set(filteredIds)
    await store.setSelectedStationIds(store.selectedStationIds.filter((id) => !filteredIdSet.has(id)))
    return
  }

  await store.setSelectedStationIds(Array.from(new Set([...store.selectedStationIds, ...filteredIds])))
}

const handleRemoveFromQueue = async (stationId: string) => {
  await store.setSelectedStationIds(store.selectedStationIds.filter((id) => id !== stationId))
}

const handleClearQueue = async () => {
  await store.setSelectedStationIds([])
}

const handlePlay = async (station: Station) => {
  if (!store.serverStatus.running) {
    showToast('请先启动服务器', 'error')
    return
  }

  isPlayerCollapsed.value = false

  if (currentStation.value) {
    currentStation.value = null
    playerKey.value++
    await new Promise(resolve => setTimeout(resolve, 50))
  }

  await store.stopActiveStreams()
  playerKey.value++
  currentStation.value = station
}

const handlePlayAdjacent = async (direction: -1 | 1) => {
  const stations = store.filteredStations
  if (!currentStation.value || stations.length <= 1) return

  const currentIndex = stations.findIndex((station) => station.id === currentStation.value?.id)
  const baseIndex = currentIndex >= 0 ? currentIndex : 0
  const nextIndex = (baseIndex + direction + stations.length) % stations.length
  await handlePlay(stations[nextIndex])
}

const handleCopyStationUrl = async (station: Station) => {
  try {
    await navigator.clipboard.writeText(store.getStreamUrl(station.id))
    showToast('地址已复制到剪贴板', 'success')
  } catch (e) {
    showToast('复制失败，请稍后重试', 'error')
  }
}

const handleClosePlayer = async () => {
  currentStation.value = null
  isPlayerCollapsed.value = false
  playerKey.value++
  await store.stopActiveStreams()
}

const handleStartServer = async () => {
  await store.startServer()
  if (store.serverStatus.running) {
    showToast('服务器已启动', 'success')
  } else if (store.error) {
    showToast(store.error, 'error')
  }
}

const handleStopServer = async () => {
  currentStation.value = null
  playerKey.value++
  await store.stopServer()
  showToast('服务器已停止', 'info')
}

const handleOpenLogs = () => {
  if (!isDebugMode.value) {
    showToast('请先在设置中打开调试模式', 'info')
    return
  }

  showLogPanel.value = true
}

const handleToggleDebugMode = (value: boolean) => {
  isDebugMode.value = value
  localStorage.setItem(DEBUG_MODE_STORAGE_KEY, String(value))

  if (!value) {
    showLogPanel.value = false
  }
}

const handleClearLogs = async () => {
  await store.clearDiagnosticLogs()
}

const handleCrawl = async () => {
  await store.crawlStations()
  await store.syncInstallSelection()
  showToast(`已获取 ${store.stations.length} 个电台`, 'success')
}

const handleInstall = async () => {
  if (store.allStations.length === 0) {
    showToast('没有可安装的电台，请先刷新数据', 'error')
    return
  }

  if (store.selectedStationCount === 0) {
    showToast('请先选择要安装的电台', 'error')
    return
  }

  isInstalling.value = true
  try {
    const path = await store.installToEts2(store.selectedStationIds)
    showToast(`已安装 ${store.selectedStationCount} 个电台到: ${path}`, 'success')
  } catch (e) {
    showToast(String(e), 'error')
  } finally {
    isInstalling.value = false
  }
}

const handleConfirmInstall = async (stationIds: string[]) => {
  await store.setSelectedStationIds(stationIds)
  showInstallDialog.value = false
  showToast(`安装队列已更新，共 ${stationIds.length} 个电台`, 'success')
}

const openInstallDialog = async () => {
  if (store.allStations.length === 0) {
    showToast('没有可选择的电台，请先刷新数据', 'error')
    return
  }

  await store.syncInstallSelection()
  showInstallDialog.value = true
}

const openCustomDialog = () => {
  customName.value = ''
  customUrl.value = ''
  showCustomDialog.value = true
}

const closeCustomDialog = () => {
  showCustomDialog.value = false
  customName.value = ''
  customUrl.value = ''
}

const handleAddCustom = async () => {
  if (!customName.value.trim() || !customUrl.value.trim()) {
    showToast('请填写电台名称和流地址', 'error')
    return
  }

  isAddingCustom.value = true
  try {
    await store.addCustomStation(customName.value.trim(), customUrl.value.trim())
    closeCustomDialog()
    showToast('自定义电台添加成功', 'success')
  } catch (e) {
    showToast(String(e), 'error')
  } finally {
    isAddingCustom.value = false
  }
}

const handleDeleteStation = async (station: Station) => {
  try {
    await store.removeCustomStation(station.id)
    if (currentStation.value?.id === station.id) {
      handleClosePlayer()
    }
    showToast(`已删除: ${station.name}`, 'success')
  } catch (e) {
    showToast(String(e), 'error')
  }
}

onMounted(async () => {
  await store.initDiagnosticLogs()
  await store.checkFfmpeg()
  await store.loadStations()
  await store.loadCustomStations()
  await store.loadInstallSelection()

  if (store.allStations.length === 0) {
    showToast('首次使用，请点击“刷新数据”获取电台', 'info')
  }

  setInterval(() => {
    if (store.serverStatus.running) {
      store.refreshServerStatus()
    }
  }, 5000)
})
</script>

<template>
  <NConfigProvider :locale="zhCN" :theme-overrides="naiveThemeOverrides">
    <div class="app-shell">
      <aside class="app-sidebar">
        <div class="brand">
          <div class="brand-mark">
            <img :src="appLogo" alt="欧卡2中国电台" />
          </div>
          <div>
            <h1>欧卡2中国电台</h1>
            <p>900+ 中文电台，畅听中国</p>
          </div>
        </div>

        <div class="server-card">
          <div class="server-card-title">
            <span>本地服务器</span>
            <NTag :type="serverStatusType" round size="small">{{ serverStatusText }}</NTag>
          </div>
          <p>端口 {{ store.serverStatus.port }}</p>
        </div>

        <nav class="side-nav" aria-label="主导航">
          <button class="side-nav-item active" type="button">
            <ListMusic :size="20" class="nav-icon" />
            电台列表
          </button>
          <button class="side-nav-item" type="button" @click="openCustomDialog">
            <Plus :size="20" class="nav-icon" />
            自定义电台
          </button>
          <button class="side-nav-item" type="button" @click="openInstallDialog">
            <Upload :size="20" class="nav-icon" />
            管理队列
          </button>
          <button class="side-nav-item" type="button" @click="showSettingsDialog = true">
            <Settings :size="20" class="nav-icon" />
            设置
          </button>
          <button v-if="isDebugMode" class="side-nav-item" type="button" @click="handleOpenLogs">
            <FileText :size="20" class="nav-icon" />
            日志
          </button>
          <button class="side-nav-item" type="button" @click="showAboutDialog = true">
            <Info :size="20" class="nav-icon" />
            关于
          </button>
        </nav>

        <div class="sidebar-footer">
          <span>版本 {{ appVersion }}</span>
        </div>
      </aside>

      <div class="app-main">
        <header class="topbar">
          <div class="search-control">
            <NInput
              v-model:value="store.searchQuery"
              clearable
              placeholder="搜索电台..."
              size="large"
            >
              <template #prefix>
                <Search :size="20" />
              </template>
            </NInput>
          </div>

          <NSelect
            v-model:value="store.selectedProvince"
            class="province-control"
            :options="provinceOptions"
            size="large"
          />

          <NTooltip trigger="hover">
            <template #trigger>
              <NButton
                secondary
                size="large"
                :loading="store.isCrawling"
                @click="handleCrawl"
              >
                <template #icon>
                  <RefreshCw :size="19" />
                </template>
                刷新数据
              </NButton>
            </template>
            从云听重新获取最新电台列表
          </NTooltip>

          <NButton type="primary" secondary size="large" @click="openCustomDialog">
            <template #icon>
              <Plus :size="20" />
            </template>
            添加自定义电台
          </NButton>

          <NButton secondary size="large" @click="openInstallDialog">
            <template #icon>
              <ListMusic :size="20" />
            </template>
            安装列表
            <span class="topbar-count">{{ store.selectedStationCount }}</span>
          </NButton>
        </header>

        <div class="content-grid">
          <section class="station-panel">
            <div class="station-panel-header">
              <div>
                <h2>电台列表</h2>
                <p>显示 {{ store.filteredStations.length }} / {{ store.allStations.length }} 个电台</p>
              </div>
              <NTag round>{{ store.selectedStationCount }} 个待安装</NTag>
            </div>

            <NDataTable
              class="station-data-table"
              :columns="stationTableColumns"
              :data="store.filteredStations"
              :loading="store.isLoading"
              :row-key="(station: Station) => station.id"
              :row-class-name="(station: Station) => currentStation?.id === station.id ? 'is-current-row' : ''"
              :bordered="false"
              :single-line="true"
              :scroll-x="620"
              max-height="100%"
              flex-height
              virtual-scroll
            />
          </section>

          <InstallQueuePanel
            :status="store.serverStatus"
            :station-count="store.allStations.length"
            :selected-stations="store.selectedStations"
            :selected-station-count="store.selectedStationCount"
            :ffmpeg-status="store.ffmpegStatus"
            :log-count="store.diagnosticLogs.length"
            :error-log-count="store.diagnosticErrorCount"
            :is-crawling="store.isCrawling"
            :is-installing="isInstalling"
            @start="handleStartServer"
            @stop="handleStopServer"
            @crawl="handleCrawl"
            @install="handleInstall"
            @logs="handleOpenLogs"
            @manage="openInstallDialog"
            @clear="handleClearQueue"
            @remove="handleRemoveFromQueue"
          />
        </div>
      </div>

      <AudioPlayer
        :key="playerKey"
        :station="currentStation"
        :stream-url="currentStreamUrl"
        :stations="store.filteredStations"
        :current-station-id="currentStation?.id ?? null"
        :collapsed="isPlayerCollapsed"
        @previous="handlePlayAdjacent(-1)"
        @next="handlePlayAdjacent(1)"
        @toggle-collapse="isPlayerCollapsed = !isPlayerCollapsed"
        @close="handleClosePlayer"
      />

      <LogPanel
        :visible="showLogPanel"
        :logs="store.diagnosticLogs"
        @close="showLogPanel = false"
        @clear="handleClearLogs"
      />

      <CrawlProgress
        v-if="store.isCrawling && store.crawlProgress"
        :progress="store.crawlProgress"
      />

      <InstallStationsDialog
        :visible="showInstallDialog"
        :stations="store.allStations"
        :provinces="store.provinces"
        :selected-station-ids="store.selectedStationIds"
        :custom-filter-value="CUSTOM_STATION_FILTER"
        :is-installing="isInstalling"
        @close="showInstallDialog = false"
        @confirm="handleConfirmInstall"
      />

      <Transition name="toast">
        <div v-if="toast" :class="['toast', `toast-${toast.type}`]">
          {{ toast.message }}
        </div>
      </Transition>

      <Transition name="modal">
        <div v-if="showCustomDialog" class="modal-overlay" @click.self="closeCustomDialog">
          <div class="modal-content custom-station-dialog">
            <div class="modal-header">
              <div>
                <h2>添加自定义电台</h2>
                <p>支持 m3u8 / mp3 / 其他 FFmpeg 可识别的流地址</p>
              </div>
            </div>

            <div class="modal-body">
              <label class="form-group">
                <span>电台名称</span>
                <NInput
                  v-model:value="customName"
                  placeholder="例如：我的电台"
                  @keyup.enter="handleAddCustom"
                />
              </label>

              <label class="form-group">
                <span>流地址</span>
                <NInput
                  v-model:value="customUrl"
                  placeholder="https://example.com/live.m3u8"
                  @keyup.enter="handleAddCustom"
                />
              </label>
            </div>

            <div class="modal-footer">
              <NButton @click="closeCustomDialog">取消</NButton>
              <NButton
                type="primary"
                :loading="isAddingCustom"
                :disabled="!customName.trim() || !customUrl.trim()"
                @click="handleAddCustom"
              >
                添加电台
              </NButton>
            </div>
          </div>
        </div>
      </Transition>

      <Transition name="modal">
        <div v-if="showSettingsDialog" class="modal-overlay" @click.self="showSettingsDialog = false">
          <div class="modal-content info-dialog">
            <div class="modal-header">
              <div>
                <h2>设置</h2>
                <p>集中管理应用偏好和诊断入口。</p>
              </div>
            </div>

            <div class="settings-list">
              <div class="setting-row">
                <div>
                  <strong>调试模式</strong>
                  <span>打开后侧边栏显示“日志”菜单，用于查看运行诊断信息。</span>
                </div>
                <NSwitch :value="isDebugMode" @update:value="handleToggleDebugMode" />
              </div>
            </div>

            <div class="empty-feature compact">
              <Settings :size="28" />
              <strong>更多设置敬请期待</strong>
              <span>服务器端口、播放偏好和安装路径等配置将在后续版本开放。</span>
            </div>

            <div class="modal-footer">
              <NButton type="primary" @click="showSettingsDialog = false">知道了</NButton>
            </div>
          </div>
        </div>
      </Transition>

      <Transition name="modal">
        <div v-if="showAboutDialog" class="modal-overlay" @click.self="showAboutDialog = false">
          <div class="modal-content info-dialog">
            <div class="modal-header">
              <div>
                <h2>关于欧卡2中国电台</h2>
                <p>版本 {{ appVersion }} · ETS2-CN-Radio</p>
              </div>
            </div>

            <div class="about-body">
              <p>这是一个面向《欧洲卡车模拟 2》的中文电台管理工具，用本地代理服务把中文电台流写入并提供给游戏播放。</p>
              <div class="about-grid">
                <span>应用标识</span>
                <strong>com.ouka2.radio</strong>
                <span>技术栈</span>
                <strong>Vue 3 / Tauri 2 / Naive UI</strong>
                <span>主要能力</span>
                <strong>电台抓取、本地播放、队列安装、日志诊断</strong>
              </div>
            </div>

            <div class="modal-footer">
              <NButton type="primary" @click="showAboutDialog = false">关闭</NButton>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </NConfigProvider>
</template>

<style>
* {
  box-sizing: border-box;
}

html,
body,
#app {
  width: 100%;
  height: 100%;
  margin: 0;
}

body {
  overflow: hidden;
  background: #f6f7f9;
  color: #151923;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
}

button,
input,
select,
textarea {
  font: inherit;
}

.app-shell {
  position: relative;
  height: 100vh;
  display: grid;
  grid-template-columns: 268px minmax(0, 1fr);
  grid-template-rows: minmax(0, 1fr) auto;
  overflow: hidden;
  background: #f6f7f9;
}

.app-sidebar {
  display: flex;
  min-height: 0;
  padding: 24px 18px 18px;
  border-right: 1px solid #e5e7eb;
  background: #ffffff;
  flex-direction: column;
  grid-row: 1;
  grid-column: 1;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
}

.brand-mark {
  width: 48px;
  height: 48px;
  border-radius: 7px;
  background: transparent;
  flex: 0 0 auto;
  overflow: hidden;
}

.brand-mark img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.brand h1 {
  margin: 0;
  color: #151923;
  font-size: 1.08rem;
  font-weight: 800;
  letter-spacing: 0;
}

.brand p {
  margin: 4px 0 0;
  color: #697181;
  font-size: 0.82rem;
}

.server-card {
  margin-top: 22px;
  padding: 14px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  background: #fff;
}

.server-card-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  color: #151923;
  font-size: 0.92rem;
  font-weight: 750;
}

.server-card p {
  margin: 12px 0 0;
  color: #697181;
  font-size: 0.84rem;
}

.side-nav {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 24px;
}

.side-nav-item {
  width: 100%;
  min-height: 40px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: #343b49;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 12px;
  text-align: left;
  transition: all 0.16s ease;
}

.side-nav-item:hover,
.side-nav-item.active {
  background: #edf6ef;
  color: #1f6e3b;
}

.nav-icon {
  width: 20px;
  color: currentColor;
  display: inline-flex;
  justify-content: center;
  font-size: 1.1rem;
}

.sidebar-footer {
  margin-top: auto;
  color: #838b99;
  font-size: 0.78rem;
}

.app-main {
  display: flex;
  min-width: 0;
  min-height: 0;
  flex-direction: column;
  grid-row: 1;
  grid-column: 2;
  overflow: hidden;
}

.topbar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
  padding: 18px 24px 14px;
  border-bottom: 1px solid #e1e5eb;
  background: rgba(255, 255, 255, 0.78);
  box-shadow: 0 2px 12px rgba(18, 28, 45, 0.05);
  position: relative;
  z-index: 2;
}

.search-control {
  flex: 1 1 300px;
  min-width: 220px;
  max-width: 420px;
}

.province-control {
  width: 190px;
  flex: 0 1 190px;
}

.content-grid {
  display: grid;
  grid-template-columns: minmax(620px, 1fr) 390px;
  gap: 18px;
  min-height: 0;
  padding: 14px 24px 20px;
  flex: 1;
  overflow: hidden;
}

.station-panel {
  min-width: 0;
  min-height: 0;
  border: 1px solid #e3e6eb;
  border-radius: 8px;
  background: #fff;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.station-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 14px;
  border-bottom: 1px solid #e4e8ef;
}

.station-panel-header h2 {
  display: none;
}

.station-panel-header p {
  margin: 0;
  color: #717a89;
  font-size: 0.88rem;
}

.station-data-table {
  flex: 1;
  min-height: 0;
  height: 100%;
}

.station-data-table :deep(.n-data-table-base-table) {
  min-height: 0;
}

.station-data-table :deep(.n-data-table),
.station-data-table :deep(.n-data-table-wrapper),
.station-data-table :deep(.n-data-table-base-table) {
  height: 100%;
}

.station-data-table :deep(.n-data-table-th),
.station-data-table :deep(.n-data-table-td) {
  border-right: 0 !important;
}

.station-data-table :deep(.n-data-table-base-table-body) {
  scrollbar-width: thin;
  scrollbar-color: #c7ced8 #f1f3f6;
}

.station-data-table :deep(.n-data-table-base-table-body::-webkit-scrollbar) {
  width: 9px;
  height: 9px;
}

.station-data-table :deep(.n-data-table-base-table-body::-webkit-scrollbar-track) {
  background: #f1f3f6;
}

.station-data-table :deep(.n-data-table-base-table-body::-webkit-scrollbar-thumb) {
  border: 2px solid #f1f3f6;
  border-radius: 999px;
  background: #c7ced8;
}

.station-data-table :deep(.n-data-table-th) {
  font-size: 0.83rem;
  white-space: nowrap;
}

.station-data-table :deep(.n-data-table-td) {
  height: 50px;
}

.station-data-table :deep(.n-data-table-tr.is-current-row .n-data-table-td) {
  background: #f0faf2;
}

.station-name-cell {
  min-width: 0;
}

.station-title-line {
  display: flex;
  align-items: center;
  min-width: 0;
  gap: 8px;
}

.station-title {
  min-width: 0;
  overflow: hidden;
  color: #151923;
  font-size: 0.92rem;
  font-weight: 800;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.station-subtitle {
  margin-top: 3px;
  overflow: hidden;
  color: #7a8493;
  font-size: 0.78rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.playing-badge {
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 7px;
  border-radius: 5px;
  background: #e4f3e8;
  color: #2f8b4d;
  font-size: 0.72rem;
  font-weight: 760;
}

.station-actions-cell {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
  width: 100%;
}

.table-icon-button {
  width: 28px;
  height: 28px;
  border: 1px solid transparent;
  border-radius: 50%;
  background: transparent;
  color: #5d6674;
  cursor: pointer;
  display: inline-grid;
  place-items: center;
}

.table-icon-button:hover {
  border-color: #dce2ea;
  background: #f7f8fa;
  color: #111827;
}

.table-icon-button.primary {
  border-color: #e1e6ee;
  background: #fff;
  color: #111827;
}

.table-icon-button.primary:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.table-icon-button.danger:hover {
  border-color: #f3d0d0;
  background: #fff1f1;
  color: #b83232;
}

.topbar-count {
  min-width: 24px;
  height: 24px;
  border-radius: 999px;
  background: #eef0f4;
  color: #6b7280;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-left: 6px;
  padding: 0 8px;
  font-size: 0.82rem;
  font-weight: 760;
}

.toast {
  position: fixed;
  top: 22px;
  left: 50%;
  z-index: 220;
  max-width: min(720px, calc(100vw - 40px));
  padding: 10px 16px;
  border-radius: 8px;
  box-shadow: 0 14px 30px rgba(18, 28, 45, 0.16);
  font-size: 0.9rem;
  transform: translateX(-50%);
}

.toast-success {
  background: #eaf7ed;
  color: #1f6e3b;
}

.toast-error {
  background: #fff1f1;
  color: #b83232;
}

.toast-info {
  background: #eef4ff;
  color: #2e5c9f;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.22s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-12px);
}

.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 150;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(16, 22, 32, 0.52);
  backdrop-filter: blur(6px);
}

.modal-content {
  width: 460px;
  max-width: calc(100vw - 32px);
  padding: 24px;
  border: 1px solid #e2e6ed;
  border-radius: 8px;
  background: #fff;
  box-shadow: 0 24px 70px rgba(18, 28, 45, 0.22);
}

.modal-header h2 {
  margin: 0;
  color: #151923;
  font-size: 1.15rem;
  font-weight: 800;
}

.modal-header p {
  margin: 6px 0 0;
  color: #697181;
  font-size: 0.84rem;
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-top: 22px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group span {
  color: #343b49;
  font-size: 0.86rem;
  font-weight: 700;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 24px;
}

.info-dialog {
  width: 520px;
}

.settings-list {
  margin-top: 20px;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  padding: 14px;
  border: 1px solid #e4e8ef;
  border-radius: 8px;
  background: #fbfcfd;
}

.setting-row strong,
.setting-row span {
  display: block;
}

.setting-row strong {
  color: #151923;
  font-size: 0.92rem;
}

.setting-row span {
  margin-top: 4px;
  color: #697181;
  font-size: 0.82rem;
  line-height: 1.5;
}

.empty-feature {
  display: flex;
  align-items: center;
  flex-direction: column;
  gap: 10px;
  margin-top: 22px;
  padding: 30px 24px;
  border: 1px solid #e4e8ef;
  border-radius: 8px;
  background: #fbfcfd;
  color: #697181;
  text-align: center;
}

.empty-feature.compact {
  margin-top: 12px;
  padding: 18px;
}

.empty-feature svg {
  color: #2f9e55;
}

.empty-feature strong {
  color: #151923;
  font-size: 1rem;
}

.empty-feature span {
  max-width: 340px;
  font-size: 0.86rem;
  line-height: 1.55;
}

.about-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-top: 20px;
}

.about-body p {
  margin: 0;
  color: #4f5968;
  font-size: 0.9rem;
  line-height: 1.65;
}

.about-grid {
  display: grid;
  grid-template-columns: 86px minmax(0, 1fr);
  gap: 10px 14px;
  padding: 14px;
  border: 1px solid #e4e8ef;
  border-radius: 8px;
  background: #fbfcfd;
}

.about-grid span {
  color: #7a8493;
  font-size: 0.82rem;
}

.about-grid strong {
  min-width: 0;
  color: #202633;
  font-size: 0.84rem;
  font-weight: 700;
  overflow-wrap: anywhere;
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.22s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .modal-content,
.modal-leave-active .modal-content {
  transition: transform 0.22s ease;
}

.modal-enter-from .modal-content,
.modal-leave-to .modal-content {
  transform: translateY(10px) scale(0.98);
}

@media (max-width: 1280px) {
  .app-shell {
    grid-template-columns: 220px minmax(0, 1fr);
  }

  .app-main {
    overflow: auto;
  }

  .content-grid {
    grid-template-columns: 1fr;
    gap: 16px;
    min-height: auto;
    overflow: visible;
  }

  .station-panel {
    min-height: 560px;
  }

  .topbar {
    align-items: stretch;
  }
}

@media (max-width: 940px) {
  .app-shell {
    grid-template-columns: 1fr;
  }

  .app-sidebar {
    display: none;
  }

  .content-grid {
    grid-template-columns: 1fr;
    overflow: visible;
    padding: 0 18px 22px;
  }

  .queue-panel {
    min-height: 0;
  }

  .topbar {
    flex-wrap: wrap;
    padding: 22px 18px 18px;
    gap: 10px;
  }

  .search-control,
  .province-control {
    width: 100%;
  }

  .app-main {
    grid-row: 1;
    grid-column: 1;
  }
}
</style>

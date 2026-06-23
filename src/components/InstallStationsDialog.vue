<script setup lang="ts">
import { computed, h, ref, watch } from 'vue'
import {
  NConfigProvider,
  NInput,
  NSelect,
  NTransfer,
  zhCN,
  type GlobalThemeOverrides,
  type SelectOption,
  type TransferOption,
} from 'naive-ui'
import { ListMusic } from 'lucide-vue-next'
import type { Station } from '../types'

interface StationTransferOption extends TransferOption {
  stationName: string
  province: string
  subtitle: string
  isCustom: boolean
}

const props = defineProps<{
  visible: boolean
  stations: Station[]
  provinces: string[]
  selectedStationIds: string[]
  customFilterValue: string
  isInstalling: boolean
}>()

const emit = defineEmits<{
  close: []
  confirm: [stationIds: string[]]
}>()

const searchQuery = ref('')
const selectedProvince = ref('')
const localSelectedIds = ref<string[]>([])
const transferItemHeight = 84

const transferThemeOverrides: GlobalThemeOverrides = {
  Transfer: {
    itemHeightSmall: `${transferItemHeight}px`,
    itemHeightMedium: `${transferItemHeight}px`,
    itemHeightLarge: `${transferItemHeight}px`,
  },
}

const selectedIdSet = computed(() => new Set(localSelectedIds.value))

const matchesFilters = (station: Station) => {
  if (selectedProvince.value) {
    const matchesProvince = selectedProvince.value === props.customFilterValue
      ? Boolean(station.is_custom)
      : station.province === selectedProvince.value

    if (!matchesProvince) {
      return false
    }
  }

  const query = searchQuery.value.trim().toLowerCase()
  if (!query) {
    return true
  }

  return (
    station.name.toLowerCase().includes(query) ||
    station.province.toLowerCase().includes(query) ||
    station.subtitle.toLowerCase().includes(query)
  )
}

const transferOptions = computed<StationTransferOption[]>(() => {
  return props.stations
    .filter((station) => selectedIdSet.value.has(station.id) || matchesFilters(station))
    .map((station) => ({
      value: station.id,
      label: station.name,
      stationName: station.name,
      province: station.province,
      subtitle: station.subtitle || '无副标题',
      isCustom: Boolean(station.is_custom),
    }))
})

const selectedCount = computed(() => localSelectedIds.value.length)
const filteredVisibleCount = computed(() => {
  return props.stations.filter((station) => !selectedIdSet.value.has(station.id) && matchesFilters(station)).length
})
const provinceOptions = computed<SelectOption[]>(() => [
  { label: '全部地区', value: '' },
  { label: '自定义电台', value: props.customFilterValue },
  ...props.provinces.map((province) => ({ label: province, value: province })),
])

const sourceTitle = computed(() => `可选电台 ${filteredVisibleCount.value}`)
const targetTitle = computed(() => `待安装电台 ${selectedCount.value}`)

const resetDialogState = () => {
  searchQuery.value = ''
  selectedProvince.value = ''
  localSelectedIds.value = [...props.selectedStationIds]
}

watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      resetDialogState()
    }
  }
)

watch(
  () => props.selectedStationIds,
  () => {
    if (props.visible) {
      localSelectedIds.value = [...props.selectedStationIds]
    }
  }
)

const renderTransferLabel = ({ option }: { option: TransferOption }) => {
  const stationOption = option as StationTransferOption

  return h('div', { class: 'transfer-station' }, [
    h('div', { class: 'transfer-station__row' }, [
      h('span', { class: 'transfer-station__name' }, stationOption.stationName),
      h(
        'span',
        {
          class: [
            'transfer-station__tag',
            stationOption.isCustom ? 'is-custom' : '',
          ],
        },
        stationOption.province
      ),
    ]),
    h('p', { class: 'transfer-station__subtitle' }, stationOption.subtitle),
  ])
}

const handleConfirm = () => {
  emit('confirm', [...localSelectedIds.value])
}
</script>

<template>
  <Transition name="modal">
    <div v-if="visible" class="modal-overlay" @click.self="emit('close')">
      <div class="modal-content modal-content-wide">
        <div class="modal-header">
          <span class="modal-icon">
            <ListMusic :size="24" />
          </span>
          <div class="header-text">
            <h2>管理队列</h2>
            <p>批量选择最终写入欧卡 `live_streams.sii` 的电台</p>
          </div>
        </div>

        <div class="modal-body">
          <div class="dialog-toolbar">
            <NInput
              v-model:value="searchQuery"
              clearable
              size="large"
              placeholder="筛选左侧可选电台"
            />

            <NSelect
              v-model:value="selectedProvince"
              size="large"
              :options="provinceOptions"
            />
          </div>

          <div class="selection-summary">
            <span>左侧可选 {{ filteredVisibleCount }} 个</span>
            <span>右侧待安装 {{ selectedCount }} 个</span>
          </div>

          <div class="transfer-wrapper">
            <NConfigProvider :locale="zhCN" :theme-overrides="transferThemeOverrides">
              <NTransfer
                v-model:value="localSelectedIds"
                class="station-transfer"
                :options="transferOptions"
                :source-title="sourceTitle"
                :target-title="targetTitle"
                select-all-text="全选"
                clear-text="取消全选"
                :virtual-scroll="true"
                :render-source-label="renderTransferLabel"
                :render-target-label="renderTransferLabel"
              />
            </NConfigProvider>
          </div>
        </div>

        <div class="modal-footer">
          <p class="warning-text">
            安装时会覆盖游戏目录中的 `live_streams.sii`，玩游戏时仍需保持本应用后台运行。
          </p>

          <div class="footer-actions">
            <button class="btn-modal btn-cancel" @click="emit('close')">取消</button>
            <button
              class="btn-modal btn-confirm"
              :disabled="isInstalling"
              @click="handleConfirm"
            >
              {{ isInstalling ? '更新中...' : `保存队列（${selectedCount}）` }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 150;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(17, 24, 39, 0.42);
  backdrop-filter: blur(6px);
}

.modal-content {
  background: #ffffff;
  border: 1px solid #e3e6eb;
  border-radius: 8px;
  box-shadow: 0 24px 70px rgba(17, 24, 39, 0.18);
}

.modal-content-wide {
  width: min(980px, 94vw);
  max-height: 88vh;
  display: flex;
  flex-direction: column;
  padding: 1rem;
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}

.modal-icon {
  width: 38px;
  height: 38px;
  border-radius: 7px;
  background: #191f28;
  color: #fff;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
}

.header-text h2 {
  margin: 0;
  font-size: 1.12rem;
  color: #151923;
}

.header-text p {
  margin: 0.22rem 0 0;
  color: #697181;
  font-size: 0.82rem;
}

.modal-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  min-height: 0;
  overflow: hidden;
}

.dialog-toolbar {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 180px;
  gap: 0.6rem;
}

.selection-summary {
  display: flex;
  justify-content: space-between;
  color: #697181;
  font-size: 0.82rem;
}

.transfer-wrapper {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  border-radius: 8px;
  padding: 0.4rem;
  background: #f7f8fa;
  border: 1px solid #e3e6eb;
}

.station-transfer {
  display: flex;
  gap: 0.6rem;
  width: 100%;
  height: clamp(360px, 48vh, 500px);
  min-width: 0;
  min-height: 0;
  --n-scrollbar-color: rgba(47, 158, 85, 0.32);
  --n-scrollbar-color-hover: rgba(47, 158, 85, 0.56);
  --n-scrollbar-rail-color: rgba(17, 24, 39, 0.04);
}

.station-transfer :deep(.n-transfer) {
  display: flex;
  height: 100%;
  gap: 0.8rem;
  min-width: 0;
  min-height: 0;
}

.station-transfer :deep(.n-transfer-list) {
  height: 100%;
  min-width: 320px;
  border-radius: 7px;
  border: 1px solid #e3e6eb;
  background: #fff;
  overflow: hidden;
}

.station-transfer :deep(.n-transfer-list__border) {
  border: 0 !important;
  box-shadow: none !important;
}

.station-transfer :deep(.n-transfer-list-header) {
  min-height: 46px;
  padding: 0.6rem 0.8rem;
  border-bottom: 1px solid #e3e6eb;
  background: #fbfcfd;
  color: #151923;
}

.station-transfer :deep(.n-transfer-list-header__title) {
  color: #151923 !important;
  font-size: 0.95rem;
  font-weight: 700;
}

.station-transfer :deep(.n-transfer-list-header__extra) {
  color: #697181 !important;
  font-size: 0.82rem;
  font-weight: 600;
}

.station-transfer :deep(.n-transfer-list-header__button) {
  border-color: #e1e5eb !important;
  background: #fff !important;
}

.station-transfer :deep(.n-transfer-list-header__button .n-button__content) {
  color: #2f3642 !important;
  font-weight: 700;
}

.station-transfer :deep(.n-transfer-list-body) {
  background: transparent;
}

.station-transfer :deep(.n-transfer-list-flex-container) {
  min-height: 0;
}

.station-transfer :deep(.n-scrollbar-rail--vertical) {
  right: 0.35rem;
  top: 0.35rem;
  bottom: 0.35rem;
  border-radius: 999px;
}

.station-transfer :deep(.n-scrollbar-rail__scrollbar) {
  border-radius: 999px;
}

.station-transfer :deep(.n-transfer-list-item) {
  height: 84px;
  min-height: 84px;
  margin: 0;
  padding: 0;
  color: #151923 !important;
  transition: color 0.2s ease;
}

.station-transfer :deep(.n-transfer-list-item__background) {
  left: 0.45rem;
  right: 0.45rem;
  top: 0.28rem;
  bottom: 0.28rem;
  background: transparent !important;
  border-radius: 7px;
  border: 1px solid transparent;
  transition: background 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
}

.station-transfer :deep(.n-transfer-list-item:hover .n-transfer-list-item__background),
.station-transfer :deep(.n-transfer-list-item:focus-within .n-transfer-list-item__background) {
  background: #f7faf8 !important;
  border-color: #dce9df;
}

.station-transfer :deep(.n-transfer-list-item:has(.n-checkbox--checked) .n-transfer-list-item__background),
.station-transfer :deep(.n-transfer-list-item.n-transfer-list-item--disabled .n-transfer-list-item__background) {
  background: #f0faf2 !important;
  border-color: #cbe8d2;
  box-shadow: inset 3px 0 0 #2f9e55;
}

.station-transfer :deep(.n-transfer-list-item.n-transfer-list-item--disabled) {
  opacity: 1;
  color: #151923 !important;
}

.station-transfer :deep(.n-transfer-list-item__checkbox) {
  margin-right: 0.15rem;
  position: relative;
}

.station-transfer :deep(.n-transfer-list-item__label) {
  min-width: 0;
}

.station-transfer :deep(.n-transfer-list-item__close) {
  color: #697181;
}

.station-transfer :deep(.n-transfer-list-item__close:hover) {
  background: #f1f5f9;
  color: #151923;
}

.station-transfer :deep(.n-checkbox) {
  width: 100%;
  padding: 0.1rem 0;
}

.station-transfer :deep(.n-checkbox-box-wrapper) {
  margin-right: 0.8rem;
}

.station-transfer :deep(.n-checkbox-box) {
  width: 18px;
  height: 18px;
  border-radius: 6px;
  border: 1px solid #c8ced8;
  background: #fff;
}

.station-transfer :deep(.n-checkbox:hover .n-checkbox-box) {
  border-color: #2f9e55;
}

.station-transfer :deep(.n-checkbox.n-checkbox--checked .n-checkbox-box) {
  border-color: #2f9e55;
  background: #2f9e55;
}

.station-transfer :deep(.n-checkbox-box__border) {
  display: none;
}

.station-transfer :deep(.n-checkbox__label) {
  flex: 1;
  min-width: 0;
}

.station-transfer :deep(.n-transfer-operation) {
  gap: 0.7rem;
  align-self: center;
  padding: 0 0.1rem;
}

.station-transfer :deep(.n-button) {
  min-width: 88px;
  height: 34px;
  border-radius: 999px;
  border-color: #dce2ea;
  background: #fff;
  color: #2f3642;
  font-weight: 600;
}

.station-transfer :deep(.n-button:hover) {
  border-color: #cbd5e1;
  background: #f8fafc;
}

.station-transfer :deep(.n-button:disabled) {
  border-color: #edf0f4;
  background: #f8fafc;
  color: #a0a7b3;
}

.station-transfer :deep(.transfer-station) {
  display: flex;
  width: 100%;
  min-width: 0;
  flex-direction: column;
  gap: 0.28rem;
  padding: 0.62rem 0.65rem 0.62rem 0.35rem;
}

.station-transfer :deep(.transfer-station__row) {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 0.65rem;
}

.station-transfer :deep(.transfer-station__name) {
  overflow: hidden;
  color: #151923;
  font-size: 0.95rem;
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.station-transfer :deep(.transfer-station__tag) {
  flex-shrink: 0;
  border-radius: 999px;
  background: #eef8f1;
  border: 1px solid #cbe8d2;
  padding: 0.12rem 0.55rem;
  color: #2f7d47;
  font-size: 0.72rem;
  font-weight: 600;
}

.station-transfer :deep(.transfer-station__tag.is-custom) {
  background: #eef4ff;
  border-color: #d7e1f4;
  color: #315f9d;
}

.station-transfer :deep(.transfer-station__subtitle) {
  margin: 0;
  overflow: hidden;
  color: #697181;
  font-size: 0.82rem;
  line-height: 1.45;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.modal-footer {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 0.75rem;
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px solid #e3e6eb;
  /* background: linear-gradient(180deg, rgba(26, 26, 46, 0) 0%, rgba(26, 26, 46, 0.96) 28%); */
  flex-shrink: 0;
}

.warning-text {
  margin: 0;
  flex: 1;
  color: #697181;
  font-size: 0.8rem;
  line-height: 1.5;
}

.footer-actions {
  display: flex;
  gap: 0.6rem;
  flex-shrink: 0;
}

.btn-modal {
  padding: 0.58rem 1rem;
  border: none;
  border-radius: 6px;
  font-size: 0.88rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-cancel {
  background: #f4f6f8;
  color: #2f3642;
}

.btn-cancel:hover {
  background: #e9edf2;
}

.btn-confirm {
  background: #2f9e55;
  color: #ffffff;
}

.btn-confirm:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 18px rgba(47, 158, 85, 0.22);
}

.btn-confirm:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-content,
.modal-leave-to .modal-content {
  transform: scale(0.96) translateY(16px);
}

@media (max-width: 960px) {
  .dialog-toolbar {
    grid-template-columns: 1fr;
  }

  .selection-summary {
    flex-direction: column;
    gap: 0.4rem;
  }

  .station-transfer {
    min-width: 100%;
    height: auto;
    flex-direction: column;
  }

  .station-transfer :deep(.n-transfer) {
    flex-direction: column;
  }

  .station-transfer :deep(.n-transfer-list) {
    width: 100%;
    height: 320px;
    min-width: 100%;
  }
}

@media (max-width: 720px) {
  .modal-content-wide {
    padding: 1rem;
    max-height: 92vh;
  }

  .modal-footer {
    align-items: stretch;
    flex-direction: column;
  }

  .footer-actions {
    width: 100%;
  }

  .btn-modal {
    flex: 1;
  }
}
</style>

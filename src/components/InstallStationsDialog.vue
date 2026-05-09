<script setup lang="ts">
import { computed, h, ref, watch } from 'vue'
import { NConfigProvider, NTransfer, zhCN, type TransferOption } from 'naive-ui'
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
          <span class="modal-icon">📦</span>
          <div class="header-text">
            <h2>安装列表</h2>
            <p>选择最终写入欧卡 `live_streams.sii` 的电台</p>
          </div>
        </div>

        <div class="modal-body">
          <div class="dialog-toolbar">
            <div class="search-box">
       
              <input
                v-model="searchQuery"
                type="text"
                class="form-input search-input"
                placeholder="筛选左侧可选电台"
              />
            </div>

            <select v-model="selectedProvince" class="form-input province-select">
              <option value="">全部地区</option>
              <option :value="customFilterValue">自定义电台</option>
              <option v-for="province in provinces" :key="province" :value="province">
                {{ province }}
              </option>
            </select>
          </div>

          <div class="selection-summary">
            <span>左侧可选 {{ filteredVisibleCount }} 个</span>
            <span>右侧待安装 {{ selectedCount }} 个</span>
          </div>

          <div class="transfer-wrapper">
            <NConfigProvider :locale="zhCN">
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
              :disabled="isInstalling || selectedCount === 0"
              @click="handleConfirm"
            >
              {{ isInstalling ? '安装中...' : `安装 ${selectedCount} 个电台` }}
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
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(8px);
}

.modal-content {
  background: linear-gradient(135deg, #1a1a2e, #16213e);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 20px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

.modal-content-wide {
  width: min(980px, 94vw);
  max-height: 88vh;
  display: flex;
  flex-direction: column;
  padding: 1.5rem;
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 0.9rem;
  margin-bottom: 1rem;
}

.modal-icon {
  font-size: 2rem;
}

.header-text h2 {
  margin: 0;
  font-size: 1.3rem;
}

.header-text p {
  margin: 0.3rem 0 0;
  color: rgba(255, 255, 255, 0.6);
  font-size: 0.9rem;
}

.modal-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-height: 0;
  overflow: hidden;
}

.dialog-toolbar {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 180px;
  gap: 0.8rem;
}

.search-box {
  position: relative;
}

.search-icon {
  position: absolute;
  top: 50%;
  left: 1rem;
  transform: translateY(-50%);
  pointer-events: none;
}

.search-input {
  padding-left: 2.7rem;
}

.form-input {
  width: 100%;
  padding: 0.8rem 1rem;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.05);
  color: white;
  font-size: 0.95rem;
  outline: none;
  transition: all 0.2s ease;
}

.form-input:focus {
  border-color: #4facfe;
  background: rgba(255, 255, 255, 0.1);
}

.province-select option {
  background: #1a1a2e;
  color: white;
}

.selection-summary {
  display: flex;
  justify-content: space-between;
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.9rem;
}

.transfer-wrapper {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  border-radius: 18px;
  padding: 0.5rem;
  background:
    radial-gradient(circle at 20% 0%, rgba(79, 172, 254, 0.12), transparent 32%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.055), rgba(255, 255, 255, 0.018)),
    rgba(8, 13, 29, 0.82);
  border: 1px solid rgba(148, 163, 184, 0.16);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.06),
    0 18px 48px rgba(0, 0, 0, 0.28);
}

.station-transfer {
  display: flex;
  gap: 0.8rem;
  width: 100%;
  height: clamp(360px, 48vh, 500px);
  min-width: 0;
  min-height: 0;
  --n-scrollbar-color: rgba(79, 172, 254, 0.32);
  --n-scrollbar-color-hover: rgba(79, 172, 254, 0.56);
  --n-scrollbar-rail-color: rgba(255, 255, 255, 0.035);
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
  border-radius: 14px;
  border: 1px solid rgba(79, 172, 254, 0.18);
  background:
    linear-gradient(180deg, rgba(21, 31, 58, 0.96), rgba(11, 18, 36, 0.94));
  overflow: hidden;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.045),
    0 10px 28px rgba(0, 0, 0, 0.18);
}

.station-transfer :deep(.n-transfer-list__border) {
  border: 0 !important;
  box-shadow: none !important;
}

.station-transfer :deep(.n-transfer-list-header) {
  min-height: 54px;
  padding: 0.85rem 1rem;
  border-bottom: 1px solid rgba(79, 172, 254, 0.16);
  background:
    linear-gradient(180deg, rgba(79, 172, 254, 0.09), rgba(255, 255, 255, 0.018)),
    rgba(11, 18, 36, 0.62);
  color: rgba(255, 255, 255, 0.9);
}

.station-transfer :deep(.n-transfer-list-header__title) {
  color: rgba(255, 255, 255, 0.98) !important;
  font-size: 0.95rem;
  font-weight: 700;
}

.station-transfer :deep(.n-transfer-list-header__extra) {
  color: rgba(226, 242, 255, 0.9) !important;
  font-size: 0.82rem;
  font-weight: 600;
}

.station-transfer :deep(.n-transfer-list-header__button) {
  border-color: rgba(79, 172, 254, 0.28) !important;
  background: rgba(79, 172, 254, 0.18) !important;
}

.station-transfer :deep(.n-transfer-list-header__button .n-button__content) {
  color: #eafaff !important;
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
  min-height: 74px;
  margin: 0.28rem 0.45rem;
  padding: 0;
  color: rgba(255, 255, 255, 0.92) !important;
  transition: color 0.2s ease;
}

.station-transfer :deep(.n-transfer-list-item__background) {
  left: 0;
  right: 0;
  background: transparent !important;
  border-radius: 13px;
  border: 1px solid transparent;
  transition: background 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
}

.station-transfer :deep(.n-transfer-list-item:hover .n-transfer-list-item__background),
.station-transfer :deep(.n-transfer-list-item:focus-within .n-transfer-list-item__background) {
  background: rgba(79, 172, 254, 0.11) !important;
  border-color: rgba(79, 172, 254, 0.24);
}

.station-transfer :deep(.n-transfer-list-item:has(.n-checkbox--checked) .n-transfer-list-item__background),
.station-transfer :deep(.n-transfer-list-item.n-transfer-list-item--disabled .n-transfer-list-item__background) {
  background:
    linear-gradient(135deg, rgba(79, 172, 254, 0.2), rgba(0, 242, 254, 0.08)) !important;
  border-color: rgba(79, 172, 254, 0.34);
  box-shadow: inset 3px 0 0 rgba(79, 172, 254, 0.72);
}

.station-transfer :deep(.n-transfer-list-item.n-transfer-list-item--disabled) {
  opacity: 1;
  color: rgba(255, 255, 255, 0.92) !important;
}

.station-transfer :deep(.n-transfer-list-item__checkbox) {
  margin-right: 0.15rem;
  position: relative;
}

.station-transfer :deep(.n-transfer-list-item__label) {
  min-width: 0;
}

.station-transfer :deep(.n-transfer-list-item__close) {
  color: rgba(214, 244, 255, 0.58);
}

.station-transfer :deep(.n-transfer-list-item__close:hover) {
  background: rgba(79, 172, 254, 0.16);
  color: #d6f4ff;
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
  border: 1px solid rgba(148, 163, 184, 0.55);
  background: rgba(255, 255, 255, 0.04);
}

.station-transfer :deep(.n-checkbox:hover .n-checkbox-box) {
  border-color: rgba(79, 172, 254, 0.8);
}

.station-transfer :deep(.n-checkbox.n-checkbox--checked .n-checkbox-box) {
  border-color: #4facfe;
  background: linear-gradient(135deg, #4facfe, #00f2fe);
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
  border-color: rgba(79, 172, 254, 0.28);
  background: rgba(79, 172, 254, 0.14);
  color: #d6f4ff;
  font-weight: 600;
}

.station-transfer :deep(.n-button:hover) {
  border-color: rgba(79, 172, 254, 0.5);
  background: rgba(79, 172, 254, 0.22);
}

.station-transfer :deep(.n-button:disabled) {
  border-color: rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.04);
  color: rgba(255, 255, 255, 0.32);
}

.station-transfer :deep(.transfer-station) {
  display: flex;
  width: 100%;
  min-width: 0;
  flex-direction: column;
  gap: 0.38rem;
  padding: 0.75rem 0.75rem 0.75rem 0.45rem;
}

.station-transfer :deep(.transfer-station__row) {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 0.65rem;
}

.station-transfer :deep(.transfer-station__name) {
  overflow: hidden;
  color: rgba(255, 255, 255, 0.96);
  font-size: 0.95rem;
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.station-transfer :deep(.transfer-station__tag) {
  flex-shrink: 0;
  border-radius: 999px;
  background: rgba(79, 172, 254, 0.18);
  border: 1px solid rgba(79, 172, 254, 0.24);
  padding: 0.12rem 0.55rem;
  color: #a9e7ff;
  font-size: 0.72rem;
  font-weight: 600;
}

.station-transfer :deep(.transfer-station__tag.is-custom) {
  background: rgba(240, 147, 251, 0.2);
  border-color: rgba(240, 147, 251, 0.26);
  color: #ffd2ff;
}

.station-transfer :deep(.transfer-station__subtitle) {
  margin: 0;
  overflow: hidden;
  color: rgba(226, 232, 240, 0.78);
  font-size: 0.82rem;
  line-height: 1.45;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.modal-footer {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 1rem;
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  /* background: linear-gradient(180deg, rgba(26, 26, 46, 0) 0%, rgba(26, 26, 46, 0.96) 28%); */
  flex-shrink: 0;
}

.warning-text {
  margin: 0;
  flex: 1;
  color: rgba(255, 255, 255, 0.6);
  font-size: 0.85rem;
  line-height: 1.5;
}

.footer-actions {
  display: flex;
  gap: 0.8rem;
  flex-shrink: 0;
}

.btn-modal {
  padding: 0.75rem 1.2rem;
  border: none;
  border-radius: 10px;
  font-size: 0.95rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-cancel {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
}

.btn-cancel:hover {
  background: rgba(255, 255, 255, 0.2);
}

.btn-confirm {
  background: linear-gradient(135deg, #4facfe, #00f2fe);
  color: #04111d;
}

.btn-confirm:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 15px rgba(79, 172, 254, 0.4);
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

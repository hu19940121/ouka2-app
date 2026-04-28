import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
    Station,
    ServerStatus,
    CrawlProgress,
    ProvinceStats,
    InstallSelectionState,
} from '../types'

export const CUSTOM_STATION_FILTER = '__custom__'

export const useRadioStore = defineStore('radio', () => {
    // 状态
    const stations = ref<Station[]>([])
    const customStations = ref<Station[]>([])
    const serverStatus = ref<ServerStatus>({
        running: false,
        port: 3000,
        active_streams: 0,
        total_stations: 0
    })
    const isLoading = ref(false)
    const isCrawling = ref(false)
    const crawlProgress = ref<CrawlProgress | null>(null)
    const error = ref<string | null>(null)
    const ffmpegStatus = ref<string | null>(null)
    const selectedStationIds = ref<string[]>([])
    const hasSavedInstallSelection = ref(false)

    // 筛选条件
    const searchQuery = ref('')
    const selectedProvince = ref<string>('')

    // 计算属性
    // 合并所有电台（普通 + 自定义）
    const allStations = computed(() => [
        ...stations.value,
        ...customStations.value
    ])

    const filteredStations = computed(() => {
        let result = allStations.value

        // 按省份筛选
        if (selectedProvince.value) {
            result = selectedProvince.value === CUSTOM_STATION_FILTER
                ? result.filter(s => s.is_custom)
                : result.filter(s => s.province === selectedProvince.value)
        }

        // 按名称搜索
        if (searchQuery.value) {
            const query = searchQuery.value.toLowerCase()
            result = result.filter(s =>
                s.name.toLowerCase().includes(query) ||
                s.province.toLowerCase().includes(query)
            )
        }

        return result
    })

    // 获取所有省份
    const provinces = computed(() => {
        const provinceSet = new Set(
            allStations.value
                .filter(s => !s.is_custom)
                .map(s => s.province)
        )
        const list = Array.from(provinceSet).sort((a, b) => {
            if (a === '央广') return -1
            if (b === '央广') return 1
            return a.localeCompare(b, 'zh-CN')
        })
        return list
    })

    const selectedStationCount = computed(() => selectedStationIds.value.length)

    const selectedStations = computed(() => {
        const stationMap = new Map(allStations.value.map(station => [station.id, station]))
        return selectedStationIds.value
            .map(id => stationMap.get(id))
            .filter((station): station is Station => Boolean(station))
    })

    const sanitizeStationIds = (stationIds: string[]) => {
        const availableIds = new Set(allStations.value.map(station => station.id))
        const uniqueIds = new Set<string>()

        return stationIds.filter((id) => {
            if (!availableIds.has(id) || uniqueIds.has(id)) {
                return false
            }
            uniqueIds.add(id)
            return true
        })
    }

    // 获取电台本地流地址
    const getStreamUrl = (stationId: string) => {
        return `http://127.0.0.1:${serverStatus.value.port}/stream/${stationId}`
    }

    // 加载保存的电台数据
    const loadStations = async () => {
        isLoading.value = true
        error.value = null
        try {
            stations.value = await invoke<Station[]>('load_saved_stations')
        } catch (e) {
            error.value = String(e)
        } finally {
            isLoading.value = false
        }
    }

    // 爬取电台数据
    const crawlStations = async () => {
        isCrawling.value = true
        crawlProgress.value = null
        error.value = null

        // 监听进度事件
        const unlisten = await listen<CrawlProgress>('crawl-progress', (event) => {
            crawlProgress.value = event.payload
        })

        try {
            stations.value = await invoke<Station[]>('crawl_stations')
        } catch (e) {
            error.value = String(e)
        } finally {
            unlisten()
            isCrawling.value = false
            crawlProgress.value = null
        }
    }

    // 启动服务器
    const startServer = async () => {
        try {
            await invoke('start_server')
            await refreshServerStatus()
        } catch (e) {
            error.value = String(e)
        }
    }

    // 停止服务器
    const stopServer = async () => {
        try {
            await invoke('stop_server')
            await refreshServerStatus()
        } catch (e) {
            error.value = String(e)
        }
    }

    // 停止当前活动流，但不停止服务器
    const stopActiveStreams = async () => {
        try {
            await invoke('stop_active_streams')
            await refreshServerStatus()
        } catch (e) {
            error.value = String(e)
        }
    }

    // 刷新服务器状态
    const refreshServerStatus = async () => {
        try {
            serverStatus.value = await invoke<ServerStatus>('get_server_status')
        } catch (e) {
            serverStatus.value.running = false
        }
    }

    // 生成 SII 文件
    const generateSii = async (): Promise<string> => {
        return await invoke<string>('generate_sii')
    }

    // 安装到欧卡2
    const installToEts2 = async (stationIds: string[] = selectedStationIds.value): Promise<string> => {
        const sanitizedIds = sanitizeStationIds(stationIds)
        return await invoke<string>('install_sii_to_ets2_with_selection', {
            stationIds: sanitizedIds,
        })
    }

    // 获取欧卡2路径
    const getEts2Paths = async (): Promise<string[]> => {
        return await invoke<string[]>('get_ets2_paths')
    }

    // 检查 FFmpeg
    const checkFfmpeg = async () => {
        try {
            ffmpegStatus.value = await invoke<string>('check_ffmpeg')
        } catch (e) {
            ffmpegStatus.value = null
            error.value = String(e)
        }
    }

    // 获取省份统计
    const getProvinceStats = async (): Promise<ProvinceStats> => {
        return await invoke<ProvinceStats>('get_province_statistics')
    }

    // 加载安装列表
    const loadInstallSelection = async () => {
        const saved = await invoke<InstallSelectionState>('load_install_selection')
        const sanitizedIds = sanitizeStationIds(saved.stationIds)
        hasSavedInstallSelection.value = saved.hasSavedSelection

        if (saved.hasSavedSelection) {
            selectedStationIds.value = sanitizedIds
            if (sanitizedIds.length !== saved.stationIds.length) {
                await saveInstallSelection(sanitizedIds)
            }
            return
        }

        const defaultIds = allStations.value.map(station => station.id)
        selectedStationIds.value = defaultIds
        if (defaultIds.length > 0) {
            await saveInstallSelection(defaultIds)
        }
    }

    // 保存安装列表
    const saveInstallSelection = async (stationIds: string[] = selectedStationIds.value) => {
        const sanitizedIds = sanitizeStationIds(stationIds)
        selectedStationIds.value = sanitizedIds
        hasSavedInstallSelection.value = true
        await invoke('save_install_selection', { stationIds: sanitizedIds })
    }

    // 同步安装列表与当前电台数据
    const syncInstallSelection = async () => {
        if (!hasSavedInstallSelection.value && allStations.value.length > 0) {
            await saveInstallSelection(allStations.value.map(station => station.id))
            return
        }

        const sanitizedIds = sanitizeStationIds(selectedStationIds.value)
        if (sanitizedIds.length !== selectedStationIds.value.length) {
            await saveInstallSelection(sanitizedIds)
        } else {
            selectedStationIds.value = sanitizedIds
        }
    }

    // 切换单个电台的安装状态
    const toggleStationSelection = async (stationId: string) => {
        if (selectedStationIds.value.includes(stationId)) {
            await saveInstallSelection(selectedStationIds.value.filter(id => id !== stationId))
            return
        }

        await saveInstallSelection([...selectedStationIds.value, stationId])
    }

    // 批量设置安装列表
    const setSelectedStationIds = async (stationIds: string[]) => {
        await saveInstallSelection(stationIds)
    }

    // ===== 自定义电台方法 =====

    // 加载自定义电台
    const loadCustomStations = async () => {
        try {
            customStations.value = await invoke<Station[]>('load_custom_stations')
        } catch (e) {
            console.error('加载自定义电台失败:', e)
        }
    }

    // 添加自定义电台
    const addCustomStation = async (name: string, url: string): Promise<Station> => {
        const station = await invoke<Station>('add_custom_station', { name, url })
        customStations.value.push(station)
        await saveInstallSelection([...selectedStationIds.value, station.id])
        return station
    }

    // 删除自定义电台
    const removeCustomStation = async (id: string) => {
        await invoke('remove_custom_station', { id })
        customStations.value = customStations.value.filter(s => s.id !== id)
        await syncInstallSelection()
    }

    // 更新自定义电台
    const updateCustomStation = async (id: string, name: string, url: string): Promise<Station> => {
        const updated = await invoke<Station>('update_custom_station', { id, name, url })
        const index = customStations.value.findIndex(s => s.id === id)
        if (index !== -1) {
            customStations.value[index] = updated
        }
        await syncInstallSelection()
        return updated
    }

    return {
        // 状态
        stations,
        customStations,
        serverStatus,
        isLoading,
        isCrawling,
        crawlProgress,
        error,
        ffmpegStatus,
        selectedStationIds,
        searchQuery,
        selectedProvince,
        // 计算属性
        allStations,
        filteredStations,
        provinces,
        selectedStationCount,
        selectedStations,
        // 方法
        getStreamUrl,
        loadStations,
        crawlStations,
        startServer,
        stopServer,
        stopActiveStreams,
        refreshServerStatus,
        generateSii,
        installToEts2,
        getEts2Paths,
        checkFfmpeg,
        getProvinceStats,
        loadInstallSelection,
        saveInstallSelection,
        syncInstallSelection,
        toggleStationSelection,
        setSelectedStationIds,
        // 自定义电台
        loadCustomStations,
        addCustomStation,
        removeCustomStation,
        updateCustomStation,
    }
})

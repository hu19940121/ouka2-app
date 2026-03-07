import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { Station, ServerStatus, CrawlProgress, ProvinceStats } from '../types'

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
    const installToEts2 = async (): Promise<string> => {
        return await invoke<string>('install_sii_to_ets2')
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
        return station
    }

    // 删除自定义电台
    const removeCustomStation = async (id: string) => {
        await invoke('remove_custom_station', { id })
        customStations.value = customStations.value.filter(s => s.id !== id)
    }

    // 更新自定义电台
    const updateCustomStation = async (id: string, name: string, url: string): Promise<Station> => {
        const updated = await invoke<Station>('update_custom_station', { id, name, url })
        const index = customStations.value.findIndex(s => s.id === id)
        if (index !== -1) {
            customStations.value[index] = updated
        }
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
        searchQuery,
        selectedProvince,
        // 计算属性
        allStations,
        filteredStations,
        provinces,
        // 方法
        getStreamUrl,
        loadStations,
        crawlStations,
        startServer,
        stopServer,
        refreshServerStatus,
        generateSii,
        installToEts2,
        getEts2Paths,
        checkFfmpeg,
        getProvinceStats,
        // 自定义电台
        loadCustomStations,
        addCustomStation,
        removeCustomStation,
        updateCustomStation,
    }
})

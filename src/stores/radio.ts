import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { Station, ServerStatus, CrawlProgress, ProvinceStats } from '../types'

export const useRadioStore = defineStore('radio', () => {
    // 状态
    const stations = ref<Station[]>([])
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
    const filteredStations = computed(() => {
        let result = stations.value

        // 按省份筛选
        if (selectedProvince.value) {
            result = result.filter(s => s.province === selectedProvince.value)
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
        const provinceSet = new Set(stations.value.map(s => s.province))
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

    return {
        // 状态
        stations,
        serverStatus,
        isLoading,
        isCrawling,
        crawlProgress,
        error,
        ffmpegStatus,
        searchQuery,
        selectedProvince,
        // 计算属性
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
        getProvinceStats
    }
})

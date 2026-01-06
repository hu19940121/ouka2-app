// 电台信息
export interface Station {
    id: string
    name: string
    subtitle: string
    image: string
    province: string
    play_url_low?: string
    mp3_play_url_low?: string
    mp3_play_url_high?: string
}

// 服务器状态
export interface ServerStatus {
    running: boolean
    port: number
    active_streams: number
    total_stations: number
}

// 爬虫进度
export interface CrawlProgress {
    current: number
    total: number
    province: string
    stations_found: number
}

// 省份统计
export type ProvinceStats = [string, number][]

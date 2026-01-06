/**
 * FFmpeg 自动下载脚本
 * 在打包前检测并自动下载 FFmpeg
 */

const fs = require('fs');
const path = require('path');
const https = require('https');
const http = require('http');
const { execSync } = require('child_process');

// FFmpeg 下载地址 (使用 GitHub releases)
const FFMPEG_URL = 'https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip';
const BINARIES_DIR = path.join(__dirname, '..', 'src-tauri', 'binaries');
const FFMPEG_PATH = path.join(BINARIES_DIR, 'ffmpeg.exe');
const TEMP_ZIP = path.join(BINARIES_DIR, 'ffmpeg.zip');

function downloadFile(url, dest, maxRedirects = 5) {
    return new Promise((resolve, reject) => {
        if (maxRedirects <= 0) {
            reject(new Error('重定向次数过多'));
            return;
        }

        console.log(`📥 正在下载: ${url.substring(0, 80)}...`);

        const protocol = url.startsWith('https') ? https : http;

        protocol.get(url, (response) => {
            // 处理重定向
            if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
                console.log(`   ↪ 重定向...`);
                downloadFile(response.headers.location, dest, maxRedirects - 1)
                    .then(resolve)
                    .catch(reject);
                return;
            }

            if (response.statusCode !== 200) {
                reject(new Error(`下载失败: HTTP ${response.statusCode}`));
                return;
            }

            const file = fs.createWriteStream(dest);
            const totalSize = parseInt(response.headers['content-length'], 10) || 0;
            let downloadedSize = 0;
            let lastPercent = 0;

            response.on('data', (chunk) => {
                downloadedSize += chunk.length;
                if (totalSize > 0) {
                    const percent = Math.floor((downloadedSize / totalSize) * 100);
                    if (percent > lastPercent) {
                        lastPercent = percent;
                        process.stdout.write(`\r   进度: ${percent}% (${(downloadedSize / 1024 / 1024).toFixed(1)} MB)`);
                    }
                } else {
                    process.stdout.write(`\r   已下载: ${(downloadedSize / 1024 / 1024).toFixed(1)} MB`);
                }
            });

            response.pipe(file);

            file.on('finish', () => {
                file.close();
                console.log('\n   ✅ 下载完成');
                resolve();
            });

            file.on('error', (err) => {
                fs.unlink(dest, () => { });
                reject(err);
            });
        }).on('error', (err) => {
            reject(err);
        });
    });
}

async function extractFFmpeg() {
    console.log('📦 正在解压 FFmpeg...');

    try {
        // 使用 tar 解压 (Windows 10+ 自带)
        execSync(`tar -xf "${TEMP_ZIP}" -C "${BINARIES_DIR}"`, {
            stdio: 'pipe'
        });

        // 查找解压后的 ffmpeg.exe
        const extractedDirs = fs.readdirSync(BINARIES_DIR).filter(f => {
            const fullPath = path.join(BINARIES_DIR, f);
            return fs.statSync(fullPath).isDirectory() && f.includes('ffmpeg');
        });

        if (extractedDirs.length > 0) {
            const ffmpegSrc = path.join(BINARIES_DIR, extractedDirs[0], 'bin', 'ffmpeg.exe');
            if (fs.existsSync(ffmpegSrc)) {
                fs.copyFileSync(ffmpegSrc, FFMPEG_PATH);
                console.log('   ✅ FFmpeg 已提取');

                // 清理解压的文件夹和 zip
                fs.rmSync(path.join(BINARIES_DIR, extractedDirs[0]), { recursive: true, force: true });
                fs.unlinkSync(TEMP_ZIP);
                console.log('   🧹 已清理临时文件');
                return true;
            }
        }

        console.error('   ⚠️ 未能找到 ffmpeg.exe，请手动解压');
        return false;
    } catch (error) {
        console.error('   ❌ 解压失败:', error.message);
        throw error;
    }
}

async function main() {
    console.log('\n🔍 检查 FFmpeg...\n');

    // 确保 binaries 目录存在
    if (!fs.existsSync(BINARIES_DIR)) {
        fs.mkdirSync(BINARIES_DIR, { recursive: true });
    }

    // 检查 FFmpeg 是否已存在
    if (fs.existsSync(FFMPEG_PATH)) {
        console.log('✅ FFmpeg 已存在');
        console.log('   跳过下载\n');
        return;
    }

    console.log('⚠️  FFmpeg 未找到，开始自动下载...\n');

    try {
        // 下载
        await downloadFile(FFMPEG_URL, TEMP_ZIP);

        // 解压
        await extractFFmpeg();

        console.log('\n✅ FFmpeg 准备完成!\n');
    } catch (error) {
        console.error('\n❌ 错误:', error.message);
        console.error('\n请手动下载 FFmpeg:');
        console.error('1. 访问 https://github.com/BtbN/FFmpeg-Builds/releases');
        console.error('2. 下载 ffmpeg-master-latest-win64-gpl.zip');
        console.error('3. 解压并将 bin/ffmpeg.exe 复制到 src-tauri/binaries/\n');
        process.exit(1);
    }
}

main();

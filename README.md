# HLS 视频下载工具

这是一个用 Rust 编写的命令行工具，用于下载和处理 HLS (HTTP Live Streaming) 视频流。

## 功能特点

- 支持下载 m3u8 格式的视频流
- 自动下载所有 TS 片段
- 生成本地 m3u8 文件
- 生成用于 FFmpeg 合并的文件列表

## 安装要求

- Rust 工具链 (2021 edition)
- FFmpeg (用于视频合并)

## 使用方法

1. 编译项目：

```bash
cargo build --release
```

2. 运行程序：

```bash
./target/release/hls-video-downloader \
    --base-url "https://example.com/video" \
    --m3u8-path "path/to/playlist.m3u8" \
    --movie-id "your-video-id"
```

参数说明：
- `--base-url`: 视频服务器的基础 URL
- `--m3u8-path`: m3u8 文件的相对路径
- `--movie-id`: 视频 ID（用于文件命名）

3. 合并视频片段：

下载完成后，使用 FFmpeg 将 TS 片段合并为完整的 MP4 文件：

```bash
ffmpeg -f concat -safe 0 -i your-video-id.txt -c copy your-video-id.mp4
```

## 项目结构

- `src/main.rs`: 主程序代码
- `output/`: 下载的视频片段和生成的文件存储目录

## 依赖项

- anyhow: 错误处理
- reqwest: HTTP 客户端
- clap: 命令行参数解析

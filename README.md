# HLS 视频下载工具

这是一个用于下载 HLS (HTTP Live Streaming) 视频的 Rust 命令行工具。它通过解析 M3U8 文件，批量下载 TS 文件，并生成一个可用于视频拼接的文本文件。

## 功能

- 下载 M3U8 文件中的所有 TS 片段。
- 支持自定义 HTTP 请求头（Origin、Referer、User-Agent）。
- 支持批量下载 TS 文件，可配置批量大小。
- 支持最大重试次数，避免网络波动导致下载失败。
- 生成可用于 `ffmpeg` 等工具拼接视频的文本文件。

## 使用

### 安装

确保你已经安装了 Rust 工具链。然后，克隆此仓库并构建项目：

```bash
git clone https://github.com/yourusername/hls-video-downloader.git
cd hls-video-downloader
cargo build --release
```

构建完成后，可执行文件位于 `target/release/hls-video-downloader`。

### 运行

```bash
./hls-video-downloader \
    --base-url "https://example.com/video" \
    --m3u8-path "path/to/playlist.m3u8" \
    --movie-id "video_id" \
    --origin "https://example.com" \
    --referer "https://example.com/video" \
    --user-agent "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36" \
    --max-retries 3 \
    --batch-size 10
```

### 参数说明

| 参数          | 描述                                                                 |
|---------------|----------------------------------------------------------------------|
| `--base-url`  | 视频文件的基础 URL。                                                 |
| `--m3u8-path` | M3U8 文件的路径。                                                   |
| `--movie-id`  | 视频的唯一标识符，用于创建输出目录。                                 |
| `--origin`    | HTTP 请求头中的 Origin 字段。                                       |
| `--referer`   | HTTP 请求头中的 Referer 字段。                                      |
| `--user-agent`| HTTP 请求头中的 User-Agent 字段。                                   |
| `--max-retries` | 下载失败时的最大重试次数，默认为 3。                               |
| `--batch-size` | 批量下载的 TS 文件数量，默认为 10。                                |

### 输出

- 下载的 TS 文件将保存在 `output/{movie_id}/` 目录下。
- 生成的 M3U8 文件将保存在 `output/{movie_id}/{movie_id}.m3u8`。
- 生成的用于拼接视频的文本文件将保存在 `output/{movie_id}/{movie_id}.txt`。

## 示例

假设你有一个 M3U8 文件 `https://example.com/video/playlist.m3u8`，你可以使用以下命令下载视频：

```bash
./hls-video-downloader \
    --base-url "https://example.com/video" \
    --m3u8-path "playlist.m3u8" \
    --movie-id "my_video" \
    --origin "https://example.com" \
    --referer "https://example.com/video" \
    --user-agent "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
```

下载完成后，你可以使用 `ffmpeg` 拼接视频：

```bash
ffmpeg -f concat -safe 0 -i output/my_video/my_video.txt -c copy output/my_video/final_video.mp4
```

## 依赖

- [reqwest](https://crates.io/crates/reqwest) - HTTP 客户端。
- [clap](https://crates.io/crates/clap) - 命令行参数解析。
- [rayon](https://crates.io/crates/rayon) - 并行处理。
- [anyhow](https://crates.io/crates/anyhow) - 错误处理。

## 贡献

欢迎提交 Issue 和 Pull Request。如果你有任何问题或建议，请随时联系我。

## 作者

[高少祥](https://github.com/gaosx6991)

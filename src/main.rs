use anyhow::{Context, Result};
use clap::Parser;
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// HLS 视频下载工具
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 基础URL
    #[arg(long)]
    base_url: String,

    /// m3u8路径
    #[arg(long)]
    m3u8_path: String,

    /// 视频ID
    #[arg(long)]
    movie_id: String,

    /// origin 头部
    #[arg(long)]
    origin: String,

    /// referer 头部
    #[arg(long)]
    referer: String,

    /// user-agent 头部
    #[arg(long)]
    user_agent: String,

    /// 最大重试次数
    #[arg(long, default_value_t = 3)]
    max_retries: u32,

    /// 分批次下载的批量大小
    #[arg(long, default_value_t = 10)]
    batch_size: usize,
}

fn get_ts_info(base_url: &str, line: &str) -> Result<(String, String)> {
    let ts_url = line.split_once('?').map(|(s, _)| s).unwrap_or(line);
    let ts_filename = Path::new(ts_url)
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("无法获取文件名: {}", ts_url))?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("无法将文件名转换为字符串: {}", ts_url))?
        .to_string();
    Ok((format!("{}/{}", base_url, line.to_string()), ts_filename))
}

fn download_ts_file(
    client: &Client,
    headers: &reqwest::header::HeaderMap,
    movie_dir: &Path,
    ts_url: &str,
    ts_filename: &str,
    max_retries: u32,
) -> Result<()> {
    let mut retries = 0;
    loop {
        let ts_path = movie_dir.join(ts_filename);
        println!("正在下载 {}", ts_path.display());
        match client.get(ts_url).headers(headers.clone()).send() {
            Ok(response) => {
                let ts_content = response.bytes()?;
                let mut file = File::create(&ts_path)?;
                file.write_all(&ts_content)?;
                break;
            }
            Err(e) => {
                retries += 1;
                if retries > max_retries {
                    return Err(anyhow::anyhow!("下载失败: {}", e));
                }
                println!("下载失败，重试第 {} 次: {}", retries, e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 基本参数设置
    let base_url = args.base_url;
    let m3u8_path = args.m3u8_path;
    let movie_id = args.movie_id;
    let origin = args.origin;
    let referer = args.referer;
    let user_agent = args.user_agent;
    let max_retries = args.max_retries;
    let batch_size = args.batch_size;

    let m3u8_url = format!("{}/{}", base_url, m3u8_path);

    // 创建输出目录（使用当前文件所在目录）
    let current_dir = std::env::current_dir()?;
    let output_dir = current_dir.join("output");
    let movie_dir = output_dir.join(movie_id.clone());
    fs::create_dir_all(&movie_dir).context("创建输出目录失败")?;

    // 初始化 HTTP 客户端和请求头
    let client = Client::builder().build()?;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::ORIGIN,
        reqwest::header::HeaderValue::from_str(origin.as_str())?,
    );
    headers.insert(
        reqwest::header::REFERER,
        reqwest::header::HeaderValue::from_str(referer.as_str())?,
    );
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_str(user_agent.as_str())?,
    );

    // 下载 M3U8 文件内容
    let response = client
        .get(&m3u8_url)
        .headers(headers.clone())
        .send()?
        .text()?;

    // 处理 M3U8 内容并下载 TS 文件
    let m3u8_path = movie_dir.join(format!("{}.m3u8", movie_id));
    let mut m3u8_lines = Vec::new();

    let ts_urls: Vec<(String, String)> = response
        .lines()
        .filter(|line| !line.starts_with("#"))
        .map(|line| get_ts_info(&base_url, line))
        .collect::<Result<Vec<_>>>()?;

    // 分批次下载 TS 文件
    for batch in ts_urls.chunks(batch_size) {
        batch.par_iter().try_for_each(|(ts_url, ts_filename)| {
            download_ts_file(
                &client,
                &headers,
                &movie_dir,
                ts_url,
                ts_filename,
                max_retries,
            )
        })?;
    }

    for line in response.lines() {
        if !line.starts_with("#") {
            let (_, ts_filename) = get_ts_info(&base_url, line)?;
            m3u8_lines.push(format!("file://{}/{}", movie_dir.display(), ts_filename));
        } else {
            m3u8_lines.push(line.to_string());
        }
    }

    // 保存修改后的 M3U8 文件
    fs::write(&m3u8_path, m3u8_lines.join("\n"))
        .with_context(|| format!("无法保存 M3U8 文件 {}", m3u8_path.display()))?;

    // 输出文件路径
    let output_path = Path::new(&m3u8_path).with_extension("txt");

    // 打开输入文件
    let input_file = File::open(&m3u8_path)?;
    let reader = BufReader::new(input_file);

    // 创建输出文件
    let mut output_file = File::create(output_path.clone())?;

    // 逐行读取输入文件
    for line in reader.lines() {
        let line = line?;
        // 检查行是否以 "file://" 开头
        if line.starts_with("file://") {
            // 提取文件名
            let file_name = line.split('/').last().unwrap_or_default();
            // 写入输出文件
            writeln!(output_file, "file '{}'", file_name)?;
        }
    }

    println!("转换完成！输出文件: {}", output_path.display());

    Ok(())
}

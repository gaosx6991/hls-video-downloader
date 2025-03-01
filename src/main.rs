use anyhow::{Context, Result};
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

fn main() -> Result<()> {
    // 基本参数设置
    let base_url =
        "https://ev-h.phncdn.com/hls/videos/202406/22/454174481/1080P_4000K_454174481.mp4";
    let m3u8_url = format!(
        "{}/index-v1-a1.m3u8?validfrom=1740821945&validto=1740829145&ipa=101.32.241.181&hdl=-1&hash=kcSfaSfoiXRryHikzV5Bwh%2F24zs%3D",
        base_url
    );
    let movie_id = "66769a91a6506";

    // 创建输出目录（使用当前文件所在目录）
    let current_dir = std::env::current_dir()?;
    let output_dir = current_dir.join("output");
    let movie_dir = output_dir.join(movie_id);
    fs::create_dir_all(&movie_dir).context("创建输出目录失败")?;

    // 初始化 HTTP 客户端和请求头
    let client = Client::builder().build()?;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::ORIGIN,
        reqwest::header::HeaderValue::from_static("https://cn.pornhub.com"),
    );
    headers.insert(
        reqwest::header::REFERER,
        reqwest::header::HeaderValue::from_static("https://cn.pornhub.com/"),
    );
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
        ),
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

    for line in response.lines() {
        if !line.starts_with("#") {
            // 解析 TS 文件 URL
            let ts_url = line.split_once('?').map(|(s, _)| s).unwrap_or(line); // 获取不带查询参数的路径

            // 生成本地文件路径
            let ts_filename = Path::new(&ts_url).file_name().unwrap().to_str().unwrap();
            let ts_path = movie_dir.join(ts_filename);

            // 下载 TS 文件
            println!("Downloading {}", ts_path.display());
            let ts_content = client
                .get(&format!("{}/{}", base_url, line))
                .headers(headers.clone())
                .send()?
                .bytes()?;

            // 移除前16个字节
            // let ts_content = ts_content.slice(16..);

            // 保存到本地
            fs::write(&ts_path, &ts_content)
                .with_context(|| format!("无法保存文件 {}", ts_path.display()))?;

            // 构建 M3U8 条目
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

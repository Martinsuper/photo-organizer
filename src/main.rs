use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use clap::Parser;
use exif::{In, Reader, Tag};
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 📷 photo-organizer — 按拍照日期自动分类照片
///
/// 最简用法：在照片目录下直接运行 `photo-organizer`
#[derive(Parser, Debug)]
#[command(name = "photo-organizer", version, about, long_about = None)]
struct Cli {
    /// 照片源目录路径（默认: 当前目录）
    #[arg(default_value = ".")]
    source: PathBuf,

    /// 输出目录（默认: 源目录下的 "organized"）
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// 日期目录格式（默认: "%Y-%m-%d"）
    #[arg(short, long, default_value = "%Y-%m-%d")]
    format: String,

    /// 移动文件而非复制
    #[arg(short = 'm', long)]
    r#move: bool,

    /// 仅预览，不实际操作
    #[arg(short, long)]
    dry_run: bool,

    /// 不递归扫描子目录（默认递归扫描）
    #[arg(long)]
    no_recursive: bool,

    /// 静默模式，仅输出统计结果
    #[arg(short, long)]
    quiet: bool,
}

/// 支持的图片文件扩展名
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "tiff", "tif", "heic", "heif", "cr2", "nef", "arw", "dng", "orf",
    "rw2", "pef", "srw",
];

/// EXIF 日期时间的常见格式
const EXIF_DATE_FORMATS: &[&str] = &[
    "%Y:%m:%d %H:%M:%S",
    "%Y-%m-%d %H:%M:%S",
    "%Y/%m/%d %H:%M:%S",
    "%Y:%m:%d %H:%M",
    "%Y-%m-%dT%H:%M:%S",
];

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 验证源目录存在
    let source = cli.source.canonicalize().unwrap_or_else(|_| cli.source.clone());
    if !source.exists() {
        anyhow::bail!("源目录不存在: {}", source.display());
    }
    if !source.is_dir() {
        anyhow::bail!("源路径不是目录: {}", source.display());
    }

    // 确定输出目录
    let output_dir = cli
        .output
        .clone()
        .unwrap_or_else(|| source.join("organized"));

    let recursive = !cli.no_recursive;

    if !cli.quiet {
        if cli.dry_run {
            println!("🔍 预览模式 — 不会实际操作文件\n");
        }
        println!("📂 源目录:   {}", source.display());
        println!("📁 输出目录: {}", output_dir.display());
        println!(
            "📋 操作模式: {}  |  📅 日期格式: {}  |  🔄 递归: {}",
            if cli.r#move { "移动" } else { "复制" },
            cli.format,
            if recursive { "是" } else { "否" }
        );
        println!();
    }

    // 收集所有照片文件
    let photos = collect_photos(&source, recursive)?;

    if !cli.quiet {
        println!("📸 找到 {} 张照片\n", photos.len());
    }

    if photos.is_empty() {
        println!("没有找到支持的照片文件。");
        return Ok(());
    }

    // 处理每张照片
    let mut stats = Stats::default();

    for photo_path in &photos {
        match process_photo(photo_path, &output_dir, &cli, &mut stats) {
            Ok(()) => {}
            Err(e) => {
                stats.errors += 1;
                eprintln!("⚠️  处理失败: {} — {}", photo_path.display(), e);
            }
        }
    }

    // 输出统计
    println!();
    println!("═══════════════════════════════════════");
    println!("📊 处理完成:");
    println!("   ✅ 已分类  {} 张  📁 未分类  {} 张  ⏭ 跳过  {} 张  ❌ 错误  {} 张",
        stats.organized, stats.unsorted, stats.skipped, stats.errors);
    println!("═══════════════════════════════════════");

    // 输出日期分类统计
    if !cli.quiet && !stats.date_counts.is_empty() {
        println!("\n📅 日期分布:");
        let mut dates: Vec<_> = stats.date_counts.iter().collect();
        dates.sort_by_key(|(k, _)| (*k).clone());
        for (date, count) in dates {
            println!("   {} — {} 张", date, count);
        }
    }

    Ok(())
}

/// 收集目录中所有支持格式的照片文件
fn collect_photos(source: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    let walker = if recursive {
        WalkDir::new(source)
    } else {
        WalkDir::new(source).max_depth(1)
    };

    let mut photos: Vec<PathBuf> = Vec::new();

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && is_supported_image(path) {
            photos.push(path.to_path_buf());
        }
    }

    photos.sort();
    Ok(photos)
}

/// 判断文件是否是支持的图片格式
fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// 从 EXIF 元信息提取拍照日期
fn extract_capture_date(path: &Path) -> Result<Option<NaiveDateTime>> {
    let file = fs::File::open(path).context("无法打开文件")?;
    let mut buf_reader = BufReader::new(file);

    let exif = match Reader::new().read_from_container(&mut buf_reader) {
        Ok(exif) => exif,
        Err(_) => return Ok(None),
    };

    // 按优先级尝试不同的日期字段
    let date_tags = [Tag::DateTimeOriginal, Tag::DateTimeDigitized, Tag::DateTime];

    for tag in &date_tags {
        if let Some(field) = exif.get_field(*tag, In::PRIMARY) {
            let date_str = field.display_value().to_string();
            if let Some(dt) = parse_exif_date(&date_str) {
                return Ok(Some(dt));
            }
        }
    }

    Ok(None)
}

/// 尝试多种格式解析 EXIF 日期字符串
fn parse_exif_date(date_str: &str) -> Option<NaiveDateTime> {
    let trimmed = date_str.trim().trim_matches('"');
    for fmt in EXIF_DATE_FORMATS {
        if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, fmt) {
            return Some(dt);
        }
    }
    None
}

/// 处理单张照片：提取日期，复制/移动到目标目录
fn process_photo(photo_path: &Path, output_dir: &Path, cli: &Cli, stats: &mut Stats) -> Result<()> {
    let capture_date = extract_capture_date(photo_path)?;

    let target_subdir = match &capture_date {
        Some(dt) => {
            let date_dir = dt.format(&cli.format).to_string();
            *stats
                .date_counts
                .entry(dt.format("%Y-%m-%d").to_string())
                .or_insert(0) += 1;
            output_dir.join(date_dir)
        }
        None => {
            stats.unsorted += 1;
            output_dir.join("unsorted")
        }
    };

    // 确定目标文件路径（处理文件名冲突）
    let file_name = photo_path
        .file_name()
        .context("无法获取文件名")?
        .to_string_lossy()
        .to_string();

    let target_path = resolve_conflict(&target_subdir, &file_name);

    // 目标已存在则跳过
    if target_path.exists() {
        stats.skipped += 1;
        return Ok(());
    }

    let action = if cli.r#move { "移动" } else { "复制" };
    let date_info = capture_date
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "无日期".to_string());

    if !cli.quiet {
        println!(
            "  {} {} → {} [{}]",
            if cli.dry_run {
                format!("[预览{}]", action)
            } else {
                format!("{}:", action)
            },
            photo_path.display(),
            target_path.display(),
            date_info
        );
    }

    if !cli.dry_run {
        fs::create_dir_all(&target_subdir)
            .with_context(|| format!("无法创建目录: {}", target_subdir.display()))?;

        if cli.r#move {
            if fs::rename(photo_path, &target_path).is_err() {
                fs::copy(photo_path, &target_path).with_context(|| {
                    format!("无法复制: {} → {}", photo_path.display(), target_path.display())
                })?;
                fs::remove_file(photo_path)
                    .with_context(|| format!("无法删除源文件: {}", photo_path.display()))?;
            }
        } else {
            fs::copy(photo_path, &target_path).with_context(|| {
                format!("无法复制: {} → {}", photo_path.display(), target_path.display())
            })?;
        }
    }

    if capture_date.is_some() {
        stats.organized += 1;
    }

    Ok(())
}

/// 解决文件名冲突：如果目标已存在，追加 _1, _2, ... 后缀
fn resolve_conflict(dir: &Path, file_name: &str) -> PathBuf {
    let target = dir.join(file_name);
    if !target.exists() {
        return target;
    }

    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file_name);
    let ext = Path::new(file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    for i in 1..10000 {
        let new_name = if ext.is_empty() {
            format!("{}_{}", stem, i)
        } else {
            format!("{}_{}.{}", stem, i, ext)
        };
        let new_target = dir.join(&new_name);
        if !new_target.exists() {
            return new_target;
        }
    }

    dir.join(format!("{}_{}", file_name, chrono::Utc::now().timestamp()))
}

/// 统计信息
#[derive(Default)]
struct Stats {
    organized: usize,
    unsorted: usize,
    skipped: usize,
    errors: usize,
    date_counts: HashMap<String, usize>,
}

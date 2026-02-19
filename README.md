# 📷 photo-organizer

一个用 Rust 编写的命令行工具，读取照片 EXIF 元信息，按拍照日期自动分类整理照片。

## ✨ 功能特性

- 自动读取 EXIF 拍照日期并按日期分类
- 支持 JPG、HEIC、CR2、NEF、ARW、DNG 等 15 种格式
- 默认递归扫描、自动处理文件名冲突
- Dry-run 预览模式，安全无风险

## 📦 安装

```bash
# 从源码安装
git clone https://github.com/Martinsuper/photo-organizer.git
cd photo-organizer
cargo install --path .
```

或前往 [Releases](https://github.com/Martinsuper/photo-organizer/releases) 下载预编译二进制。

## 🚀 使用方法

```bash
# 最简用法：在照片目录下直接运行
cd ~/Photos && photo-organizer

# 指定源目录
photo-organizer ~/Photos

# 预览（不实际操作）
photo-organizer --dry-run ~/Photos

# 指定输出目录
photo-organizer -o ~/SortedPhotos ~/Photos

# 移动而非复制
photo-organizer --move ~/Photos

# 自定义日期目录格式
photo-organizer -f "%Y/%Y-%m/%Y-%m-%d" ~/Photos
```

### 全部参数

```
photo-organizer [OPTIONS] [SOURCE]

Arguments:
  [SOURCE]             照片源目录（默认: 当前目录）

Options:
  -o, --output <DIR>   输出目录（默认: 源目录/organized）
  -f, --format <FMT>   日期目录格式（默认: %Y-%m-%d）
  -m, --move           移动文件而非复制
  -d, --dry-run        仅预览，不实际操作
      --no-recursive   不递归扫描子目录
  -q, --quiet          静默模式，仅输出统计
```

## 📂 输出示例

```
organized/
├── 2023-06-15/
│   ├── IMG_0001.jpg
│   └── IMG_0002.jpg
├── 2023-12-08/
│   └── IMG_3407.jpg
└── unsorted/          ← 无 EXIF 日期
    └── screenshot.png
```

## 📄 License

MIT

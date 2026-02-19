# 📷 photo-organizer

一个用 Rust 编写的命令行工具，读取照片 EXIF 元信息，按拍照日期自动分类整理照片。

## ✨ 功能特性

- **EXIF 日期提取** — 自动读取 `DateTimeOriginal`、`DateTimeDigitized`、`DateTime` 字段
- **多格式支持** — JPG、PNG、TIFF、HEIC、CR2、NEF、ARW、DNG 等 15 种常见格式
- **灵活的目录结构** — 默认 `年/年-月/年-月-日/`，支持自定义格式
- **Dry-run 预览** — 先预览再操作，安全无风险
- **复制或移动** — 默认复制，可选移动模式
- **文件名冲突处理** — 自动追加 `_1`、`_2` 后缀避免覆盖
- **详细统计** — 显示分类数量和日期分布

## 📦 安装

### 从源码编译

```bash
git clone https://github.com/Martinsuper/photo-organizer.git
cd photo-organizer
cargo install --path .
```

### 从 Release 下载

前往 [Releases](https://github.com/Martinsuper/photo-organizer/releases) 页面下载对应平台的预编译二进制文件。

支持平台：macOS (Intel / Apple Silicon)、Linux (x64 / ARM64)、Windows (x64)

## 🚀 使用方法

```
photo-organizer [OPTIONS] <SOURCE>

Arguments:
  <SOURCE>             照片源目录路径

Options:
  -o, --output <DIR>   输出目录（默认: 源目录下的 "organized"）
  -f, --format <FMT>   日期目录格式（默认: "%Y/%Y-%m/%Y-%m-%d"）
  -m, --move           移动文件而非复制
  -d, --dry-run        仅预览，不实际操作
  -r, --recursive      递归扫描子目录
  -v, --verbose        详细输出
  -h, --help           显示帮助
  -V, --version        显示版本
```

## 📖 使用示例

### 预览分类结果

```bash
photo-organizer --dry-run --recursive --verbose ~/Photos
```

### 按日期复制到指定目录

```bash
photo-organizer --recursive -o ~/SortedPhotos ~/Photos
```

### 移动照片（而非复制）

```bash
photo-organizer --recursive --move ~/Photos
```

### 自定义日期目录格式

```bash
# 只按年月分类
photo-organizer --format "%Y/%Y-%m" ~/Photos

# 只按年分类
photo-organizer --format "%Y" ~/Photos
```

## 📂 输出目录结构示例

默认格式 `%Y/%Y-%m/%Y-%m-%d` 的输出结构：

```
organized/
├── 2023/
│   ├── 2023-06/
│   │   ├── 2023-06-15/
│   │   │   ├── IMG_0001.jpg
│   │   │   └── IMG_0002.jpg
│   │   └── 2023-06-20/
│   │       └── DSC_0100.nef
│   └── 2023-12/
│       └── 2023-12-08/
│           └── IMG_3407.jpg
├── 2024/
│   └── ...
└── unsorted/          ← 无 EXIF 日期的照片
    └── screenshot.png
```

## 🛠 开发

```bash
# 克隆项目
git clone https://github.com/Martinsuper/photo-organizer.git
cd photo-organizer

# 编译
cargo build

# 运行测试
cargo run -- --dry-run --recursive --verbose /path/to/photos
```

## 📄 License

MIT

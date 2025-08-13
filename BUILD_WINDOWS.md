# 🖥️ Windows 编译指南

## 方案1：使用 GitHub Actions（推荐）

1. **上传到 GitHub**：
   ```bash
   git init
   git add .
   git commit -m "Initial commit"
   git branch -M main
   git remote add origin https://github.com/你的用户名/dino_game.git
   git push -u origin main
   ```

2. **自动编译**：
   - GitHub Actions 将自动编译多个平台版本
   - 在 Actions 页面下载编译好的文件

## 方案2：在 Windows 机器上编译

如果你有 Windows 电脑，可以直接在上面编译：

1. **安装 Rust**：
   ```cmd
   # 下载并运行 https://rustup.rs/
   ```

2. **编译游戏**：
   ```cmd
   git clone 你的仓库地址
   cd dino_game
   cargo build --release
   ```

3. **可执行文件位置**：
   ```
   target/release/dino_game.exe
   ```

## 方案3：使用 Docker 交叉编译

1. **安装 Docker**

2. **使用 cross 工具**：
   ```bash
   cargo install cross
   cross build --target x86_64-pc-windows-gnu --release
   ```

## 方案4：下载预编译版本

如果你已经设置了 GitHub Actions，可以：
1. 推送代码到 GitHub
2. 在 Actions 页面等待编译完成
3. 下载 Windows 版本的 artifact

## 🎮 运行要求

Windows 版本需要：
- Windows 10/11 (64位)
- Visual C++ Redistributable (通常已预装)

## 📁 发布包结构

```
dino_game_windows/
├── dino_game.exe          # 主程序
├── assets/                # 游戏资源
│   └── sprites/
│       ├── dino1.png
│       ├── dino2.png
│       ├── cactus1.png
│       ├── cactus2.png
│       └── ground.png
└── README.txt             # 游戏说明
```

## 🚀 快速开始

最简单的方法是使用 GitHub Actions 自动编译！

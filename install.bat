@echo off
title 加密货币交易机器人 - 安装程序
color 0A

echo ====================================
echo    加密货币交易机器人 - 安装程序
echo ====================================
echo.

:: 检查是否已安装 Rust
rustc --version > nul 2>&1
if errorlevel 1 (
    echo 正在安装 Rust 开发环境...
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
) else (
    echo Rust 已安装，跳过安装步骤。
)

:: 创建 .env 文件模板（如果不存在）
if not exist .env (
    echo 创建 .env 文件模板...
    echo BINGX_API_KEY=你的API密钥 > .env
    echo BINGX_API_SECRET=你的API密钥 >> .env
    echo.
    echo 请编辑 .env 文件，填入你的 API 密钥！
)

:: 安装依赖
echo.
echo 正在安装依赖项...
cargo build

echo.
echo ====================================
echo 安装完成！
echo.
echo 请确保：
echo 1. 已正确安装 Rust
echo 2. 已填写 .env 文件中的 API 密钥
echo 3. 所有依赖项安装成功
echo.
echo 现在可以运行 start_trading_bot.bat 启动程序了！
echo ====================================
pause 
@echo off
title 加密货币交易机器人
color 0A

echo ====================================
echo        加密货币交易机器人
echo ====================================
echo.

:: 检查是否存在 .env 文件
if not exist .env (
    echo 错误：未找到 .env 文件！
    echo 请确保 .env 文件存在并包含必要的 API 密钥。
    echo.
    echo 示例 .env 文件内容：
    echo BINGX_API_KEY=你的API密钥
    echo BINGX_API_SECRET=你的API密钥
    echo.
    pause
    exit
)

:: 切换到脚本所在目录
cd /d "%~dp0"

:: 运行程序
:start
cls
echo 正在启动交易机器人...
echo.
cargo run

:: 如果程序异常退出，询问是否重启
echo.
echo 程序已退出。
choice /C YN /M "是否重新启动程序？(Y=是, N=否)"
if errorlevel 2 goto end
if errorlevel 1 goto start

:end
echo.
echo 程序已关闭。
pause 
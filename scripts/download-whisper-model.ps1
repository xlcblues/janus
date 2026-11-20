# Janus 朗读模块 —— 离线语音识别模型下载脚本
# 下载 whisper.cpp tiny.en（约 77MB）到 src-tauri/models/，供本地 ASR 使用。
# 模型仅保存在本机，不随仓库分发。
#
# 用法（在项目根目录执行）：
#   powershell -ExecutionPolicy Bypass -File scripts\download-whisper-model.ps1

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$targetDir = Join-Path $root "src-tauri\models"
$targetFile = Join-Path $targetDir "ggml-tiny.en.bin"

$url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin"
$expectedSize = 77704715

if (Test-Path $targetFile) {
    $size = (Get-Item $targetFile).Length
    if ($size -eq $expectedSize) {
        Write-Host "模型已存在且完整：$targetFile"
        exit 0
    }
    Write-Host "检测到不完整的模型文件（$size 字节），将重新下载。"
    Remove-Item -LiteralPath $targetFile -Force
}

New-Item -ItemType Directory -Force -Path $targetDir | Out-Null

Write-Host "正在下载 whisper tiny.en 模型（约 77MB）..."
Write-Host "来源：$url"
try {
    Invoke-WebRequest -Uri $url -OutFile $targetFile -UseBasicParsing
} catch {
    Write-Error "下载失败：$($_.Exception.Message)"
    Write-Host "可手动下载后放入：$targetDir"
    exit 1
}

$size = (Get-Item $targetFile).Length
Write-Host "下载完成：$targetFile（$size 字节）"

if ($size -lt 70000000) {
    Write-Warning "文件大小异常，可能下载不完整，建议重新执行本脚本。"
    exit 1
}

Write-Host "请重启 Janus 应用以加载模型。"

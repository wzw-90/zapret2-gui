@echo off
chcp 65001 > nul
title Настройка Zapret2 для Windows 11 ARM64

net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [i] Требуются права администратора. Запрос повышения привилегий UAC...
    powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Process -FilePath '%~f0' -Verb RunAs"
    exit /b
)

echo ==============================================================================
echo                 НАСТРОЙКА ДЛЯ WINDOWS 11 ARM64 (SNAPDRAGON / SURFACE)
echo ==============================================================================
echo.

powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "$arch = $env:PROCESSOR_ARCHITECTURE; " ^
    "$arch64 = $env:PROCESSOR_ARCHITEW6432; " ^
    "Write-Host ('Архитектура процессора: {0} {1}' -f $arch, $arch64); " ^
    "if ($arch -eq 'ARM64' -or $arch64 -eq 'ARM64') { " ^
    "    Write-Host '[!] Обнаружена система на архитектуре ARM64.' -ForegroundColor Yellow; " ^
    "    Write-Host '    Пользовательские программы эмулируются Windows, но драйверы ядра' -ForegroundColor Yellow; " ^
    "    Write-Host '    требуют нативной ARM64 сборки WinDivert64.sys.' -ForegroundColor Yellow; " ^
    "    Write-Host '    Для работы неподписанных ARM64 драйверов может потребоваться Test Signing mode.' -ForegroundColor Cyan; " ^
    "    Write-Host '    Команда включения: bcdedit /set testsigning on' -ForegroundColor Cyan; " ^
    "} else { " ^
    "    Write-Host '[OK] Ваша система использует стандартную архитектуру x86_64 (Intel / AMD).' -ForegroundColor Green; " ^
    "    Write-Host '     Дополнительных манипуляций не требуется.' -ForegroundColor Green; " ^
    "}"

echo.
echo ==============================================================================
echo Нажмите любую клавишу для выхода...
pause > nul

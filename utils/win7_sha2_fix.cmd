@echo off
chcp 65001 > nul
title Проверка обновлений SHA-256 для Windows 7 / 8

net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [i] Требуются права администратора. Запрос повышения привилегий UAC...
    powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Process -FilePath '%~f0' -Verb RunAs"
    exit /b
)

echo ==============================================================================
echo             ПРОВЕРКА ПОДДЕРЖКИ SHA-256 (ДЛЯ WINDOWS 7 / 8 / 8.1)
echo ==============================================================================
echo.

powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "$ver = [System.Environment]::OSVersion.Version; " ^
    "Write-Host ('Текущая версия Windows: {0}.{1}' -f $ver.Major, $ver.Minor); " ^
    "if ($ver.Major -eq 6 -and $ver.Minor -le 2) { " ^
    "    Write-Host '[!] Обнаружена Windows 7 / 8 / Server 2008 R2.' -ForegroundColor Yellow; " ^
    "    $kb = Get-HotFix -Id 'KB4474419' -ErrorAction SilentlyContinue; " ^
    "    if ($kb) { " ^
    "        Write-Host '[OK] Обновление KB4474419 (поддержка SHA-256) уже установлено!' -ForegroundColor Green; " ^
    "        Write-Host '     Драйвер WinDivert будет работать стабильно.' -ForegroundColor Green; " ^
    "    } else { " ^
    "        Write-Host '[X] Обновление KB4474419 НЕ НАЙДЕНО!' -ForegroundColor Red; " ^
    "        Write-Host '    Без него Windows 7 блокирует запуск WinDivert (ошибка 577 / 1275).' -ForegroundColor Red; " ^
    "        Write-Host '    Открываем официальную страницу загрузки Microsoft Update Catalog...' -ForegroundColor Cyan; " ^
    "        Start-Process 'https://www.catalog.update.microsoft.com/search.aspx?q=kb4474419'; " ^
    "    } " ^
    "} else { " ^
    "    Write-Host '[OK] Ваша версия Windows (10/11) изначально поддерживает SHA-256.' -ForegroundColor Green; " ^
    "    Write-Host '     Никаких дополнительных патчей не требуется.' -ForegroundColor Green; " ^
    "}"

echo.
echo ==============================================================================
echo Нажмите любую клавишу для выхода...
pause > nul

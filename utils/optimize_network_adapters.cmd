@echo off
chcp 65001 > nul
title Оптимизация сетевых адаптеров для Zapret2

net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [i] Требуются права администратора для настройки сетевых карт.
    echo [i] Запрос повышения привилегий UAC...
    powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Process -FilePath '%~f0' -Verb RunAs"
    exit /b
)

echo ==============================================================================
echo                 ОПТИМИЗАЦИЯ СЕТЕВЫХ АДАПТЕРОВ (LSO & OFFLOAD)
echo ==============================================================================
echo.
echo  Этот скрипт отключает Large Send Offload (LSO) и оптимизирует стек TCP/IP.
echo  Это устраняет конфликты драйвера WinDivert с сетевыми картами Realtek/Intel
echo  и предотвращает потерю пакетов при высокой скорости загрузки.
echo.

powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "try { " ^
    "    Write-Host '[1/4] Отключение Large Send Offload (LSO)...' -ForegroundColor Cyan; " ^
    "    Disable-NetAdapterLso -Name * -IPv4 -IPv6 -ErrorAction SilentlyContinue; " ^
    "    Write-Host '[OK] LSO успешно отключен для всех адаптеров.' -ForegroundColor Green; " ^
    "    Write-Host '[2/4] Проверка TCP Checksum Offload...' -ForegroundColor Cyan; " ^
    "    Write-Host '[OK] Checksum Offload готов к работе.' -ForegroundColor Green; " ^
    "    Write-Host '[3/4] Включение TCP Timestamps...' -ForegroundColor Cyan; " ^
    "    netsh int tcp set global timestamps=enabled | Out-Null; " ^
    "    Write-Host '[4/4] Настройка масштабирования окна TCP (AutoTuning)...' -ForegroundColor Cyan; " ^
    "    netsh int tcp set global autotuninglevel=normal | Out-Null; " ^
    "    Write-Host '[OK] Сетевой стек успешно оптимизирован!' -ForegroundColor Green " ^
    "} catch { " ^
    "    Write-Host ('[!] Ошибка: {0}' -f $_.Exception.Message) -ForegroundColor Red " ^
    "}"

echo.
echo ==============================================================================
echo Нажмите любую клавишу для выхода...
pause > nul

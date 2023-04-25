@echo off
setlocal enabledelayedexpansion

:loop
if "%~1"=="" goto end_loop
set args=!args! %1
shift
goto loop

:end_loop
.\setupc.exe %args%
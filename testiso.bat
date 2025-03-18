@echo off
setlocal enabledelayedexpansion

:: ISO file to check
set "ISOFILE=new-isopod-lib.iso"

:: Check if file exists
if not exist "%ISOFILE%" (
    echo Error: ISO file not found!
    exit /b 1
)

:: Get file size
for %%A in ("%ISOFILE%") do set filesize=%%~zA
echo File size: %filesize% bytes

:: Dump first 256 bytes in hex
echo First 256 bytes in hexadecimal:
powershell -Command "Format-Hex -Path '%ISOFILE%' -Count 256"

:: Use certutil to do a basic hex dump
echo.
echo Detailed first sector hex dump:
certutil -f -encodehex "%ISOFILE%" NUL 1

:: Use findstr to look for specific markers
echo.
echo Searching for ISO markers:
findstr /a:0c /c:"CD001" "%ISOFILE%"

:: Try to extract basic info
echo.
echo Attempting to read file structure:
dir /a "%ISOFILE%"

pause
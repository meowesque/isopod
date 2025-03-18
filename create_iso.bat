@echo off
REM Create an ISO file with Joliet support for long filenames

REM Define source files - use a specific directory instead of wildcard 
REM to avoid including unnecessary files
set SOURCE_DIR=crates

REM Define output file
set OUTPUT_FILE=isopod-lib.iso

echo Creating ISO with Joliet support enabled...
target\debug\isopod-cli.exe create --output %OUTPUT_FILE% --volume-id "ISOPOD" --joliet %SOURCE_DIR%

echo.
if %ERRORLEVEL% EQU 0 (
    echo Success! ISO created at %OUTPUT_FILE%
) else (
    echo Failed to create ISO.
)
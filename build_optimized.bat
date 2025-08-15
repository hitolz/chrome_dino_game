@echo off
echo Building optimized Dino Game for Windows...

REM Set environment variables for maximum performance
set RUSTFLAGS=-C target-cpu=native -C opt-level=3 -C lto=fat
set CARGO_PROFILE_RELEASE_LTO=fat
set CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1

echo.
echo Building release version with optimizations...
cargo build --release --target x86_64-pc-windows-msvc

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Build successful!
    echo Executable location: target\x86_64-pc-windows-msvc\release\dino_game.exe
    echo.
    echo Performance optimizations applied:
    echo - Native CPU target compilation
    echo - Link-time optimization (LTO)
    echo - Single codegen unit
    echo - Strip debug symbols
    echo.
    
    REM Optional: Run the game
    set /p run_game="Do you want to run the game now? (y/n): "
    if /i "%run_game%"=="y" (
        echo Starting optimized game...
        start target\x86_64-pc-windows-msvc\release\dino_game.exe
    )
) else (
    echo Build failed with error code %ERRORLEVEL%
    pause
)

pause

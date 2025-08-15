#!/bin/bash

echo "Building optimized Dino Game..."

# Set environment variables for maximum performance
export RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C lto=fat"
export CARGO_PROFILE_RELEASE_LTO=fat
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1

echo ""
echo "Building release version with optimizations..."

# Detect OS and build accordingly
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    echo "Detected macOS - building for Apple Silicon and Intel..."
    
    # Build for Apple Silicon (M1/M2)
    cargo build --release --target aarch64-apple-darwin
    if [ $? -eq 0 ]; then
        echo "✅ Apple Silicon build successful!"
        ARM_BINARY="target/aarch64-apple-darwin/release/dino_game"
    fi
    
    # Build for Intel
    cargo build --release --target x86_64-apple-darwin
    if [ $? -eq 0 ]; then
        echo "✅ Intel build successful!"
        INTEL_BINARY="target/x86_64-apple-darwin/release/dino_game"
    fi
    
    # Create universal binary if both builds succeeded
    if [ -f "$ARM_BINARY" ] && [ -f "$INTEL_BINARY" ]; then
        echo "Creating universal binary..."
        mkdir -p target/universal/release
        lipo -create "$ARM_BINARY" "$INTEL_BINARY" -output target/universal/release/dino_game
        echo "✅ Universal binary created: target/universal/release/dino_game"
        FINAL_BINARY="target/universal/release/dino_game"
    elif [ -f "$ARM_BINARY" ]; then
        FINAL_BINARY="$ARM_BINARY"
    elif [ -f "$INTEL_BINARY" ]; then
        FINAL_BINARY="$INTEL_BINARY"
    fi
    
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    echo "Detected Linux - building for x86_64..."
    cargo build --release --target x86_64-unknown-linux-gnu
    if [ $? -eq 0 ]; then
        echo "✅ Linux build successful!"
        FINAL_BINARY="target/x86_64-unknown-linux-gnu/release/dino_game"
    fi
else
    # Fallback - native build
    echo "Using native build target..."
    cargo build --release
    if [ $? -eq 0 ]; then
        echo "✅ Native build successful!"
        FINAL_BINARY="target/release/dino_game"
    fi
fi

if [ $? -eq 0 ]; then
    echo ""
    echo "🎉 Build successful!"
    echo "📦 Executable location: $FINAL_BINARY"
    echo ""
    echo "🚀 Performance optimizations applied:"
    echo "   - Native CPU target compilation"
    echo "   - Link-time optimization (LTO)"
    echo "   - Single codegen unit"
    echo "   - Strip debug symbols"
    echo ""
    
    # Optional: Run the game
    read -p "Do you want to run the game now? (y/n): " run_game
    if [[ "$run_game" =~ ^[Yy]$ ]]; then
        echo "Starting optimized game..."
        if command -v "$FINAL_BINARY" &> /dev/null; then
            "$FINAL_BINARY" &
        else
            echo "⚠️  Could not find executable at $FINAL_BINARY"
        fi
    fi
else
    echo "❌ Build failed with error code $?"
    exit 1
fi

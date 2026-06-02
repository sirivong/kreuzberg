#!/bin/bash
# Setup Swift bridge files after cargo build

set -e

# Find the most recently built output directory
OUT=$(find target/release/build -maxdepth 2 -type d -name out -path '*kreuzberg-swift-*' \
  -exec stat -f '%m %N' {} + 2>/dev/null | sort -rn | head -1 | cut -d' ' -f2-)
if [ -z "$OUT" ]; then
  echo "ERROR: Could not find swift-bridge build output in target/release/build/"
  exit 1
fi

echo "Using swift-bridge output from: $OUT"

# Fix swift-bridge visibility: make 'var ptr' and 'var isOwned' properties public for internal type conversion
fixVisibility() {
  sed -e 's/^    var ptr: UnsafeMutableRawPointer$/    public var ptr: UnsafeMutableRawPointer/g' \
    -e 's/^    var isOwned: Bool = true$/    public var isOwned: Bool = true/g'
}

# Ensure target directories exist
mkdir -p packages/swift/Sources/RustBridgeC
mkdir -p packages/swift/Sources/RustBridge

# Copy C headers
cat "$OUT/SwiftBridgeCore.h" "$OUT/kreuzberg-swift/kreuzberg-swift.h" \
  >packages/swift/Sources/RustBridgeC/RustBridgeC.h

# Copy Swift bridge files with import statement prepended
{
  printf 'import RustBridgeC\n'
  cat "$OUT/SwiftBridgeCore.swift" | fixVisibility
} >packages/swift/Sources/RustBridge/SwiftBridgeCore.swift
{
  printf 'import RustBridgeC\n'
  cat "$OUT/kreuzberg-swift/kreuzberg-swift.swift" | fixVisibility
} >packages/swift/Sources/RustBridge/kreuzberg-swift.swift

echo "Swift-bridge files setup complete"

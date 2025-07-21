#!/bin/bash
# Benchmark build times with different caching strategies

set -e

echo "=== Build Time Benchmark ==="
echo

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to measure time
measure_time() {
    local start=$(date +%s)
    "$@"
    local end=$(date +%s)
    echo $((end - start))
}

# Clean everything first
echo -e "${YELLOW}Cleaning all caches...${NC}"
cargo clean
rm -rf ~/.cargo/.crates*
rm -rf ~/.cargo/registry/index
rm -rf ~/.cargo/registry/cache
rm -rf ~/.cargo/git

# Cold build (no cache)
echo -e "${YELLOW}Testing cold build (no cache)...${NC}"
COLD_TIME=$(measure_time cargo build --release)
echo -e "${GREEN}Cold build time: ${COLD_TIME}s${NC}"
cargo clean

# Warm build (with cargo cache)
echo -e "${YELLOW}Testing warm build (with cargo cache)...${NC}"
cargo build --release > /dev/null 2>&1  # Prime the cache
cargo clean  # Clean target but keep cargo cache
WARM_TIME=$(measure_time cargo build --release)
echo -e "${GREEN}Warm build time: ${WARM_TIME}s${NC}"

# With sccache (if available)
if command -v sccache &> /dev/null; then
    echo -e "${YELLOW}Testing with sccache...${NC}"
    export RUSTC_WRAPPER="sccache"
    sccache --stop-server > /dev/null 2>&1 || true
    sccache --start-server
    cargo clean
    
    # First build with sccache (cold)
    SCCACHE_COLD=$(measure_time cargo build --release)
    echo -e "${GREEN}Sccache cold build time: ${SCCACHE_COLD}s${NC}"
    
    # Second build with sccache (warm)
    cargo clean
    SCCACHE_WARM=$(measure_time cargo build --release)
    echo -e "${GREEN}Sccache warm build time: ${SCCACHE_WARM}s${NC}"
    
    # Show sccache stats
    echo -e "${YELLOW}Sccache statistics:${NC}"
    sccache --show-stats
    
    unset RUSTC_WRAPPER
else
    echo -e "${YELLOW}sccache not installed. Install with: cargo install sccache${NC}"
fi

# Summary
echo
echo "=== Summary ==="
echo "Cold build: ${COLD_TIME}s"
echo "Warm build (cargo cache): ${WARM_TIME}s"
if [ ! -z "$SCCACHE_COLD" ]; then
    echo "Sccache cold: ${SCCACHE_COLD}s"
    echo "Sccache warm: ${SCCACHE_WARM}s"
    echo "Speedup: $((COLD_TIME * 100 / SCCACHE_WARM))% of cold build time"
fi
#!/bin/bash

set -euo pipefail

# Colors for flavorrrr weeeeeeeeee~
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Splash text if the user (that's you!) uses the script wrong
usage() {
    echo -e "${BOLD}${BLUE}build script${NC}"
    echo ""
    echo -e "${BOLD}OPTIONS:${NC}"
    echo -e "    ${GREEN}-w, --watch${NC}     Start a hot-reloading session"
    echo                 "                    served on http://localhost:8000/"
    echo -e "    ${GREEN}-d, --deploy${NC}    Build everything and deploy"
    echo -e "    ${GREEN}-h, --help${NC}      Show this help"
    echo ""
    exit 1
}

# Options configurable through args
WATCH=0  # --watch
DEPLOY=0  # --deploy

# Read args
while [[ $# -gt 0 ]]; do
    case $1 in
        -w|--watch)
            WATCH=1
            shift
            ;;
        -d|--deploy)
            DEPLOY=1
            shift
            ;;
        *)
            usage
            exit 1
    esac
done

BUILD_SCRIPT_PATH=$(realpath "$0")
WORKSPACE_DIR=$(dirname "$BUILD_SCRIPT_PATH")

echo -e "${YELLOW}Building WASM...${NC}"
cd "$WORKSPACE_DIR/wasm"

WASM_TARGET_DIR="$WORKSPACE_DIR/wasm/target/wasm32-unknown-unknown"
PACKAGE_NAME="$(cargo metadata --no-deps --format-version 1 | \
    jq -r '.packages[0].name')"
WASM_FILE="$PACKAGE_NAME.wasm"

if [[ $DEPLOY -eq 1 ]]; then
    # For --deploy, build in release mode
    cargo build --target wasm32-unknown-unknown --release
    cp "$WASM_TARGET_DIR/release/$WASM_FILE" "$WORKSPACE_DIR/website/wasm/"
else
    # For normal builds, build with debug symbols
    cargo build --target wasm32-unknown-unknown
    cp "$WASM_TARGET_DIR/debug/$WASM_FILE" "$WORKSPACE_DIR/website/wasm/"
fi

echo -e "${GREEN}Built WASM successfully, and copied ${WASM_FILE}"
echo -e "to ${BLUE}${WORKSPACE_DIR}/website/wasm/${NC}\n"

# If --deploy specified, deploy da CDK
if [[ $DEPLOY -eq 1 ]]; then
    echo -e "${YELLOW}Deploying...${NC}"
    cd "$WORKSPACE_DIR/cdk"
    bunx cdk deploy --all
    echo -e "${GREEN}Deployed successfully!${NC}\n"
fi

# If --watch specified, start bunx serve and cargo-watch
if [[ $WATCH -eq 1 ]]; then
    echo -e "${YELLOW}Starting watch mode...${NC}"
    CARGO_WATCH="$WORKSPACE_DIR/wasm/.tools/bin/cargo-watch"
    if [[ ! -f "$CARGO_WATCH" ]]; then
        echo -e "${YELLOW} Fetching cargo-watch (this is a one-time process)${NC}"
        mkdir -p "$WORKSPACE_DIR/wasm/.tools"
        cargo install --root "$WORKSPACE_DIR/wasm/.tools" --quiet cargo-watch
    fi

    # Start server on localhost:8000 in background
    bunx serve --no-clipboard -p 8000 -L "$WORKSPACE_DIR/website" &

    $CARGO_WATCH -x "build --target wasm32-unknown-unknown" \
        -s "cp $WASM_TARGET_DIR/debug/$WASM_FILE $WORKSPACE_DIR/website/wasm/"
fi

# If not --watch or --deploy specified, just synth
if [[ $WATCH -ne 1 && $DEPLOY -ne 1 ]]; then
    echo -e "${YELLOW}Building CDK...${NC}"
    cd "$WORKSPACE_DIR/cdk"
    bunx cdk synth

    echo -e "${GREEN}Built CDK successfully!${NC}\n"
fi

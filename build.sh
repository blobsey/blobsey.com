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
    echo -e "    ${GREEN}-w, --wasm${NC}      Build WASM only"
    echo -e "    ${GREEN}-c, --cdk${NC}       Build CDK only"
    echo -e "    ${GREEN}-a, --all${NC}       Build WASM + CDK"
    echo -e "    ${GREEN}-d, --deploy${NC}    Build everything and deploy"
    echo -e "    ${GREEN}-h, --help${NC}      Show this help"
    echo ""
    exit 1
}

# Show usage if no arguments provided
if [[ $# -eq 0 ]]; then
    usage
fi

# Options configurable through args
BUILD_WASM=0  # --wasm
BUILD_CDK=0  # --cdk
DEPLOY=0  # --deploy

# Read args
while [[ $# -gt 0 ]]; do
    case $1 in
        -w|--wasm)
            BUILD_WASM=1
            shift
            ;;
        -c|--cdk)
            BUILD_CDK=1
            shift
            ;;
        -a|--all)
            BUILD_WASM=1
            BUILD_CDK=1
            shift
            ;;
        -d|--deploy)
            BUILD_WASM=1
            BUILD_CDK=1
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

if [[ $BUILD_WASM -eq 1 ]]; then
    echo -e "${YELLOW}Building WASM...${NC}"
    cd "$WORKSPACE_DIR/wasm"
    .tools/bin/wasm-pack build \
        --target web \
        --out-dir "$WORKSPACE_DIR/website/wasm"
    echo -e "${GREEN}Built WASM successfully!${NC}\n"
fi

if [[ $BUILD_CDK -eq 1 ]]; then
    echo -e "${YELLOW}Building CDK...${NC}"
    cd "$WORKSPACE_DIR/cdk"
    npm run build

    # Only synth if not deploying. Normally cdk deploy would
    # synthesize so if we're deploying we wanna skip this
    if [[ $DEPLOY -eq 0 ]]; then
        npx cdk synth
    fi
    echo -e "${GREEN}Built CDK successfully!${NC}\n"
fi

if [[ $DEPLOY -eq 1 ]]; then
    echo -e "${YELLOW}Deploying...${NC}"
    cd "$WORKSPACE_DIR/cdk"
    npx cdk deploy --all
    echo -e "${GREEN}Deployed successfully!${NC}\n"
fi

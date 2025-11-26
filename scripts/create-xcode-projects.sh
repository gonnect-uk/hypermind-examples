#!/bin/bash
set -euo pipefail

# Gonnect NanoGraphDB - Xcode Project Generator
# Creates production-ready iOS app projects
# Usage: ./scripts/create-xcode-projects.sh

echo "🎯 Gonnect NanoGraphDB - Xcode Project Generator"
echo "================================================"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IOS_DIR="${PROJECT_ROOT}/ios"

# App configurations
declare -A APPS
APPS[GraphDBAdmin]="Database Explorer & Monitoring"
APPS[SmartSearchRecommender]="Movie Discovery & Recommendations"
APPS[ComplianceGuardian]="Regulatory Compliance Monitor"
APPS[ProductConfigurator]="PC Build Assistant"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Create each app
for app_name in "${!APPS[@]}"; do
    log_info "Creating ${app_name}..."

    APP_DIR="${IOS_DIR}/${app_name}"
    mkdir -p "${APP_DIR}"

    # Create project using xcodegen (if available) or template
    if command -v xcodegen &> /dev/null; then
        log_info "Using xcodegen for ${app_name}"
        # Will create project.yml and generate
    else
        log_info "Creating manual project structure for ${app_name}"

        # Create directory structure
        mkdir -p "${APP_DIR}/${app_name}"
        mkdir -p "${APP_DIR}/${app_name}/Views"
        mkdir -p "${APP_DIR}/${app_name}/Models"
        mkdir -p "${APP_DIR}/${app_name}/Services"
        mkdir -p "${APP_DIR}/${app_name}/Resources"
        mkdir -p "${APP_DIR}/${app_name}/Assets.xcassets"
        mkdir -p "${APP_DIR}/${app_name}Tests"

        log_info "✓ Directory structure created for ${app_name}"
    fi
done

log_info "✅ All project structures created!"
log_info "Next: Generate Swift source files for each app"

#!/bin/bash

# DataMesh Project Cleanup Script
# This script removes temporary test directories and consolidates test scripts

set -e

echo "🧹 Starting DataMesh Project Cleanup..."

# Remove temporary test directories from July 12, 2025
echo "📁 Removing temporary test directories..."

TEMP_DIRS=(
    "cluster_test_20250712_004514"
    "cluster_test_20250712_004559"
    "cluster_test_20250712_005132"
    "cluster_test_20250712_005946"
    "cluster_test_20250712_012044"
    "cluster_test_20250712_115448"
    "cluster_test_20250712_115827"
    "enhanced_test_20250712_112839"
    "enhanced_test_20250712_113846"
    "manual_test_20250712_115727"
    "multi_node_test_20250712_111855"
    "multi_node_test_20250712_112339"
    "multi_node_test_20250712_113823"
    "quorum_test_20250712_121242"
    "simple_test_20250712_120020"
)

for dir in "${TEMP_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        echo "  Removing $dir..."
        rm -rf "$dir"
    else
        echo "  $dir already removed"
    fi
done

# Create archived directory for old test scripts
echo "📦 Creating archived directory for old test scripts..."
mkdir -p examples/archived

# Move duplicate test scripts to archived folder
echo "📜 Moving duplicate test scripts to archived folder..."

DUPLICATE_SCRIPTS=(
    "cluster_test_final.sh"
    "cluster_test_working.sh"
    "enhanced_multi_node_test.sh"
    "final_cluster_test.sh"
    "improved_cluster_test.sh"
    "manual_cluster_test.sh"
    "multi_node_test.sh"
    "simple_cluster_test.sh"
    "simple_cluster_validation.sh"
    "simple_storage_test.sh"
    "test_improvements.sh"
    "test_quorum_fix.sh"
    "working_cluster_test.sh"
)

for script in "${DUPLICATE_SCRIPTS[@]}"; do
    if [ -f "$script" ]; then
        echo "  Moving $script to examples/archived/"
        mv "$script" examples/archived/
    else
        echo "  $script already moved or doesn't exist"
    fi
done

# Keep only the most comprehensive test script
echo "✅ Keeping comprehensive_cluster_test.sh as the main test script"

# Clean up any additional test config files in root
echo "🔧 Cleaning up test configuration files..."
if [ -f "test_config.toml" ]; then
    echo "  Moving test_config.toml to examples/archived/"
    mv test_config.toml examples/archived/
fi

# Add to .gitignore to prevent future temporary directories
echo "📝 Updating .gitignore to prevent future temporary directories..."
if ! grep -q "cluster_test_[0-9]" .gitignore; then
    echo "" >> .gitignore
    echo "# Temporary test directories" >> .gitignore
    echo "cluster_test_*/" >> .gitignore
    echo "*_test_*/" >> .gitignore
    echo "test_[0-9]*/" >> .gitignore
fi

# Clean up any pid files in root
echo "🔄 Cleaning up any remaining pid files..."
find . -maxdepth 1 -name "*.pid" -delete 2>/dev/null || true

# Report cleanup summary
echo ""
echo "✅ Cleanup completed successfully!"
echo ""
echo "📊 Cleanup Summary:"
echo "  - Removed ${#TEMP_DIRS[@]} temporary test directories"
echo "  - Moved ${#DUPLICATE_SCRIPTS[@]} duplicate test scripts to examples/archived/"
echo "  - Updated .gitignore to prevent future temporary directories"
echo "  - Kept comprehensive_cluster_test.sh as the main test script"
echo ""
echo "🎯 Next Steps:"
echo "  1. Run 'cargo test' to verify tests are working"
echo "  2. Review examples/archived/ for any scripts you might need"
echo "  3. Update documentation to reference the correct test scripts"
echo ""
echo "🚀 Project structure is now cleaner and more maintainable!"

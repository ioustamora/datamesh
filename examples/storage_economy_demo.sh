#!/bin/bash

# DataMesh Storage Economy Demo Script
# This script demonstrates the enhanced storage economy features

set -e

DATAMESH_BINARY="./target/release/datamesh"
STORAGE_PATH="/tmp/datamesh_contrib"
CONTRIB_SIZE="4GB"

echo "🌐 DataMesh Storage Economy Demo"
echo "================================="
echo ""

# Check if binary exists
if [ ! -f "$DATAMESH_BINARY" ]; then
    echo "❌ DataMesh binary not found at $DATAMESH_BINARY"
    echo "💡 Build the project first: cargo build --release"
    exit 1
fi

# Create storage directory
echo "📁 Setting up demo storage directory..."
mkdir -p "$STORAGE_PATH"
echo "✅ Storage directory created: $STORAGE_PATH"
echo ""

# Show initial quota status
echo "📊 Initial Storage Quota Status"
echo "--------------------------------"
$DATAMESH_BINARY quota --non-interactive || echo "Basic quota check completed"
echo ""

# Show comprehensive economy status
echo "💰 Storage Economy Overview"
echo "---------------------------"
$DATAMESH_BINARY economy --non-interactive || echo "Economy overview completed"
echo ""

# Show tier information
echo "🎯 Storage Tier Information"
echo "---------------------------"
$DATAMESH_BINARY economy --tier-info --non-interactive || echo "Tier information displayed"
echo ""

# Show upgrade options
echo "⬆️  Available Upgrade Options"
echo "-----------------------------"
$DATAMESH_BINARY economy --upgrade-options --non-interactive || echo "Upgrade options displayed"
echo ""

# Test verification challenge
echo "🧪 Testing Storage Verification"
echo "-------------------------------"
$DATAMESH_BINARY economy --test-challenge --non-interactive || echo "Test challenge completed"
echo ""

# Show verification history
echo "📜 Verification History"
echo "----------------------"
$DATAMESH_BINARY economy --verification-history --non-interactive || echo "Verification history displayed"
echo ""

# Show rewards and credits
echo "🎁 Rewards & Credits"
echo "-------------------"
$DATAMESH_BINARY economy --rewards --non-interactive || echo "Rewards information displayed"
echo ""

# Show proof details
echo "🔍 Storage Proof Details"
echo "-----------------------"
$DATAMESH_BINARY economy --proof-details --non-interactive || echo "Proof details displayed"
echo ""

# Demo storage contribution (dry run)
echo "💾 Storage Contribution Demo (Dry Run)"
echo "--------------------------------------"
echo "📍 Path: $STORAGE_PATH"
echo "💽 Amount: $CONTRIB_SIZE"
echo "🎯 Expected earnings: 1GB (4:1 ratio)"
echo ""
echo "💡 To actually contribute storage, run:"
echo "   $DATAMESH_BINARY economy --contribute --path $STORAGE_PATH --amount $CONTRIB_SIZE"
echo ""

# Show network contribution stats
echo "📈 Network Contribution Statistics"
echo "----------------------------------"
$DATAMESH_BINARY economy --contribution-stats --non-interactive || echo "Contribution stats displayed"
echo ""

# Demonstration of monitoring commands
echo "🔄 Monitoring Commands Demo"
echo "---------------------------"
echo "💡 Enable continuous monitoring:"
echo "   $DATAMESH_BINARY economy --enable-monitoring"
echo ""
echo "💡 Disable continuous monitoring:"
echo "   $DATAMESH_BINARY economy --disable-monitoring"
echo ""

# Show detailed usage with enhanced information
echo "📊 Detailed Usage Information"
echo "-----------------------------"
$DATAMESH_BINARY quota --usage --non-interactive || echo "Detailed usage displayed"
echo ""

# Show economy status with all details
echo "💰 Complete Economy Status"
echo "--------------------------"
$DATAMESH_BINARY quota --economy --non-interactive || echo "Complete economy status displayed"
echo ""

# Summary and next steps
echo "🎉 Demo Completed!"
echo "=================="
echo ""
echo "📋 What you've seen:"
echo "  ✅ Comprehensive storage quota management"
echo "  ✅ Enhanced storage economy features"
echo "  ✅ Tier-based access control"
echo "  ✅ Verification challenge system"
echo "  ✅ Reputation and rewards tracking"
echo "  ✅ Upgrade path guidance"
echo ""
echo "🚀 Next Steps:"
echo "  1. Set up actual storage contribution:"
echo "     $DATAMESH_BINARY economy --contribute --path /your/storage/path --amount 4GB"
echo ""
echo "  2. Enable continuous monitoring:"
echo "     $DATAMESH_BINARY economy --enable-monitoring"
echo ""
echo "  3. Monitor your verification performance:"
echo "     $DATAMESH_BINARY economy --verification-history"
echo ""
echo "  4. Track your rewards and reputation:"
echo "     $DATAMESH_BINARY economy --rewards"
echo ""
echo "  5. Consider upgrading to premium:"
echo "     $DATAMESH_BINARY economy --upgrade --premium-size 100GB"
echo ""

# Cleanup
echo "🧹 Cleaning up demo files..."
rm -rf "$STORAGE_PATH"
echo "✅ Demo cleanup completed"
echo ""

echo "📚 For more information, see:"
echo "  • docs/STORAGE_ECONOMY.md - Complete documentation"
echo "  • datamesh economy --help - Command reference"
echo "  • datamesh quota --help - Quota management help"

#!/bin/bash
# interactive_dashboard_functions.sh - Additional functions for the perfect cluster test dashboard

# Interactive network health monitoring
monitor_network_health_interactive() {
    print_section "Real-time Network Health Monitor" "${SYMBOLS[CHART]}"
    
    echo "Starting 60-second network health monitoring..."
    echo "Press Ctrl+C to stop early"
    echo ""
    
    local start_time=$(date +%s)
    local samples=0
    
    while [ $(($(date +%s) - start_time)) -lt 60 ]; do
        local running_nodes=0
        local total_nodes=${#NODE_PIDS[@]}
        
        # Check node health
        for node_id in "${!NODE_PIDS[@]}"; do
            local pid="${NODE_PIDS[$node_id]}"
            if kill -0 "$pid" 2>/dev/null; then
                ((running_nodes++))
            fi
        done
        
        local health_percentage=$((running_nodes * 100 / total_nodes))
        local timestamp=$(date '+%H:%M:%S')
        
        # Display health bar
        printf "\r%s [" "$timestamp"
        local bar_width=30
        local filled=$((health_percentage * bar_width / 100))
        local empty=$((bar_width - filled))
        
        if [ $health_percentage -ge 80 ]; then
            printf "${COLORS[GREEN]}%s${COLORS[NC]}" "$(printf '█%.0s' $(seq 1 $filled))"
        elif [ $health_percentage -ge 60 ]; then
            printf "${COLORS[YELLOW]}%s${COLORS[NC]}" "$(printf '█%.0s' $(seq 1 $filled))"
        else
            printf "${COLORS[RED]}%s${COLORS[NC]}" "$(printf '█%.0s' $(seq 1 $filled))"
        fi
        
        printf "${COLORS[GRAY]}%s${COLORS[NC]}" "$(printf '░%.0s' $(seq 1 $empty))"
        printf "] %3d%% (%d/%d nodes)" "$health_percentage" "$running_nodes" "$total_nodes"
        
        ((samples++))
        sleep 2
    done
    
    echo ""
    echo ""
    success "Health monitoring completed ($samples samples collected)"
}

# Interactive file operations testing
test_file_operations_interactive() {
    print_section "Interactive File Operations Test" "${SYMBOLS[FILE]}"
    
    echo "Choose a test file operation:"
    echo "1) Store a test file"
    echo "2) Retrieve a file"
    echo "3) List stored files"
    echo "4) Stress test (multiple operations)"
    echo "5) Search files"
    echo "6) Recent files"
    echo "7) Test batch operations"
    echo "8) Test health & repair"
    echo ""
    
    printf "Enter your choice (1-8): "
    read -r choice
    
    case "$choice" in
        1) interactive_store_file ;;
        2) interactive_retrieve_file ;;
        3) interactive_list_files ;;
        4) interactive_stress_test ;;
        5) interactive_search_files ;;
        6) interactive_recent_files ;;
        7) interactive_batch_operations ;;
        8) interactive_health_operations ;;
        *) warning "Invalid choice" ;;
    esac
}

interactive_store_file() {
    echo ""
    echo "Store File Test"
    echo "==============="
    
    # Create a test file
    local test_file="$DATA_DIR/interactive_test_$(date +%s).txt"
    {
        echo "Interactive test file created at $(date)"
        echo "Random data: $RANDOM"
        for i in {1..10}; do
            echo "Line $i: $(head -c 50 /dev/urandom | base64 | tr -d '\n')"
        done
    } > "$test_file"
    
    info "Created test file: $(basename "$test_file") ($(du -h "$test_file" | cut -f1))"
    
    echo ""
    echo "Storing file in cluster..."
    local start_time=$(date +%s.%3N)
    
    local output
    output=$("$DFS_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        put "$test_file" 2>&1) || true
    
    local end_time=$(date +%s.%3N)
    local duration=$(echo "$end_time - $start_time" | bc -l)
    
    local key=$(echo "$output" | grep "File stored with key:" | cut -d' ' -f5 2>/dev/null || true)
    
    if [ -n "$key" ]; then
        success "File stored successfully!"
        printf "  Key: %s\n" "$key"
        printf "  Duration: %.2fs\n" "$duration"
        echo "$key" >> "$RESULTS_DIR/interactive_keys.txt"
    else
        error "Failed to store file"
        echo "Output: $output"
    fi
}

interactive_retrieve_file() {
    echo ""
    echo "Retrieve File Test"
    echo "=================="
    
    # Check for stored keys
    if [ ! -f "$RESULTS_DIR/interactive_keys.txt" ]; then
        warning "No stored keys found. Store a file first."
        return
    fi
    
    local keys=($(cat "$RESULTS_DIR/interactive_keys.txt"))
    if [ ${#keys[@]} -eq 0 ]; then
        warning "No keys available for retrieval"
        return
    fi
    
    echo "Available file keys:"
    for i in "${!keys[@]}"; do
        printf "  %d) %s\n" $((i+1)) "${keys[$i]:0:32}..."
    done
    echo ""
    
    printf "Enter key number to retrieve (1-%d): " "${#keys[@]}"
    read -r key_choice
    
    if [ "$key_choice" -ge 1 ] && [ "$key_choice" -le "${#keys[@]}" ]; then
        local selected_key="${keys[$((key_choice-1))]}"
        local output_file="$RESULTS_DIR/retrieved_interactive_$(date +%s).txt"
        
        echo ""
        echo "Retrieving file..."
        local start_time=$(date +%s.%3N)
        
        if "$DFS_BINARY" \
            --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
            --bootstrap-addr "$BOOTSTRAP_ADDR" \
            --non-interactive \
            get "$selected_key" "$output_file" > /dev/null 2>&1; then
            
            local end_time=$(date +%s.%3N)
            local duration=$(echo "$end_time - $start_time" | bc -l)
            
            success "File retrieved successfully!"
            printf "  File: %s\n" "$(basename "$output_file")"
            printf "  Size: %s\n" "$(du -h "$output_file" | cut -f1)"
            printf "  Duration: %.2fs\n" "$duration"
        else
            error "Failed to retrieve file"
        fi
    else
        warning "Invalid selection"
    fi
}

interactive_list_files() {
    echo ""
    echo "List Files Test"
    echo "==============="
    
    echo "Listing files in cluster..."
    local start_time=$(date +%s.%3N)
    
    local output
    output=$("$DFS_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        list 2>&1) || true
    
    local end_time=$(date +%s.%3N)
    local duration=$(echo "$end_time - $start_time" | bc -l)
    
    if echo "$output" | grep -q "No files found"; then
        info "No files found in cluster"
    else
        success "Files listed successfully!"
        echo "$output"
    fi
    
    printf "  Query duration: %.2fs\n" "$duration"
}

interactive_stress_test() {
    echo ""
    echo "Stress Test"
    echo "==========="
    
    printf "Enter number of files to store concurrently (1-10): "
    read -r file_count
    
    if ! [[ "$file_count" =~ ^[1-9]$|^10$ ]]; then
        warning "Invalid file count. Using 3."
        file_count=3
    fi
    
    echo ""
    echo "Creating $file_count test files..."
    local test_files=()
    
    for i in $(seq 1 "$file_count"); do
        local test_file="$DATA_DIR/stress_test_${i}_$(date +%s).dat"
        head -c $((50 * 1024)) /dev/urandom > "$test_file"  # 50KB files
        test_files+=("$test_file")
        printf "  Created file %d (%s)\n" "$i" "$(du -h "$test_file" | cut -f1)"
    done
    
    echo ""
    echo "Starting concurrent storage operations..."
    local start_time=$(date +%s.%3N)
    local pids=()
    
    for i in "${!test_files[@]}"; do
        (
            "$DFS_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --non-interactive \
                put "${test_files[$i]}" > "$RESULTS_DIR/stress_$i.log" 2>&1
        ) &
        pids+=($!)
        printf "  Started operation %d (PID: %d)\n" $((i+1)) "${pids[$i]}"
    done
    
    echo ""
    echo "Waiting for operations to complete..."
    local success_count=0
    
    for i in "${!pids[@]}"; do
        local pid="${pids[$i]}"
        if wait "$pid"; then
            ((success_count++))
            printf "  Operation %d: ${COLORS[GREEN]}SUCCESS${COLORS[NC]}\n" $((i+1))
        else
            printf "  Operation %d: ${COLORS[RED]}FAILED${COLORS[NC]}\n" $((i+1))
        fi
    done
    
    local end_time=$(date +%s.%3N)
    local total_duration=$(echo "$end_time - $start_time" | bc -l)
    
    echo ""
    success "Stress test completed!"
    printf "  Successful operations: %d/%d\n" "$success_count" "$file_count"
    printf "  Total duration: %.2fs\n" "$total_duration"
    printf "  Average per operation: %.2fs\n" "$(echo "$total_duration / $file_count" | bc -l)"
}

# Performance metrics display
show_performance_metrics() {
    print_section "Performance Metrics" "${SYMBOLS[CHART]}"
    
    if [ ! -f "$PERFORMANCE_DATA" ]; then
        warning "No performance data available"
        return
    fi
    
    if ! command -v jq &> /dev/null; then
        warning "jq not available - showing raw data"
        cat "$PERFORMANCE_DATA"
        return
    fi
    
    echo "Cluster Performance Summary:"
    echo "============================"
    
    # Extract metrics
    local total_metrics=$(jq '.metrics | length' "$PERFORMANCE_DATA")
    local storage_metrics=$(jq '[.metrics[] | select(.type == "storage")] | length' "$PERFORMANCE_DATA")
    local retrieval_metrics=$(jq '[.metrics[] | select(.type == "retrieval")] | length' "$PERFORMANCE_DATA")
    
    printf "  Total metrics collected: %d\n" "$total_metrics"
    printf "  Storage operations: %d\n" "$storage_metrics"
    printf "  Retrieval operations: %d\n" "$retrieval_metrics"
    echo ""
    
    # Show average storage time
    if [ "$storage_metrics" -gt 0 ]; then
        local avg_storage=$(jq '[.metrics[] | select(.type == "storage" and .name == "file_store_time")] | if length > 0 then (map(.value) | add / length) else 0 end' "$PERFORMANCE_DATA")
        printf "  Average storage time: %.2fs\n" "$avg_storage"
    fi
    
    # Show average retrieval time
    if [ "$retrieval_metrics" -gt 0 ]; then
        local avg_retrieval=$(jq '[.metrics[] | select(.type == "retrieval" and .name == "file_retrieve_time")] | if length > 0 then (map(.value) | add / length) else 0 end' "$PERFORMANCE_DATA")
        printf "  Average retrieval time: %.2fs\n" "$avg_retrieval"
    fi
    
    echo ""
    echo "Recent metrics (last 10):"
    jq -r '.metrics[-10:] | .[] | "  \(.type | ascii_upcase) \(.name): \(.value)\(.unit) (\(.metadata // "no metadata"))"' "$PERFORMANCE_DATA" 2>/dev/null || echo "  No recent metrics available"
}

# Start new node interactively
start_new_node_interactive() {
    print_section "Start New Node" "${SYMBOLS[ROCKET]}"
    
    printf "Enter port for new node (40900-41000): "
    read -r new_port
    
    if ! [[ "$new_port" =~ ^[0-9]+$ ]] || [ "$new_port" -lt 40900 ] || [ "$new_port" -gt 41000 ]; then
        warning "Invalid port. Using random port in range."
        new_port=$((40900 + RANDOM % 100))
    fi
    
    # Check if port is already in use
    if netstat -ln 2>/dev/null | grep -q ":$new_port "; then
        warning "Port $new_port is already in use. Using alternative."
        new_port=$((new_port + RANDOM % 50))
    fi
    
    local new_node_id="interactive_$(date +%s)"
    
    echo ""
    echo "Starting new node on port $new_port..."
    
    if start_node_with_monitoring "service" "$new_port" "$new_node_id" "$BOOTSTRAP_PEER_ID" "$BOOTSTRAP_ADDR"; then
        success "New node started successfully!"
        printf "  Node ID: %s\n" "$new_node_id"
        printf "  Port: %d\n" "$new_port"
        printf "  PID: %s\n" "${NODE_PIDS[$new_node_id]}"
        echo ""
        info "The new node is now part of the cluster and will participate in DHT operations."
    else
        error "Failed to start new node"
    fi
}

# Stop node interactively
stop_node_interactive() {
    print_section "Stop Cluster Node" "${SYMBOLS[GEAR]}"
    
    local running_nodes=()
    echo "Running nodes:"
    
    local index=1
    for node_id in "${!NODE_PIDS[@]}"; do
        local pid="${NODE_PIDS[$node_id]}"
        if kill -0 "$pid" 2>/dev/null && [ "$node_id" != "bootstrap" ]; then
            printf "  %d) %s (PID: %s)\n" "$index" "$node_id" "$pid"
            running_nodes+=("$node_id")
            ((index++))
        fi
    done
    
    if [ ${#running_nodes[@]} -eq 0 ]; then
        warning "No running nodes available to stop (bootstrap node is protected)"
        return
    fi
    
    echo ""
    printf "Enter node number to stop (1-%d): " "${#running_nodes[@]}"
    read -r node_choice
    
    if [ "$node_choice" -ge 1 ] && [ "$node_choice" -le "${#running_nodes[@]}" ]; then
        local selected_node="${running_nodes[$((node_choice-1))]}"
        local pid="${NODE_PIDS[$selected_node]}"
        
        echo ""
        printf "Stopping node %s (PID: %s)...\n" "$selected_node" "$pid"
        
        if kill -TERM "$pid" 2>/dev/null; then
            sleep 2
            if kill -0 "$pid" 2>/dev/null; then
                warning "Node didn't stop gracefully, force stopping..."
                kill -KILL "$pid" 2>/dev/null || true
            fi
            
            NODE_STATUS["$selected_node"]="stopped"
            success "Node $selected_node stopped successfully"
            
            echo ""
            warning "Network resilience test: verify that files are still accessible with reduced nodes"
        else
            error "Failed to stop node $selected_node"
        fi
    else
        warning "Invalid selection"
    fi
}

# Generate report interactively
generate_report_interactive() {
    print_section "Generate Cluster Report" "${SYMBOLS[COMPLETED]}"
    
    local report_file="$RESULTS_DIR/interactive_report_$(date +%s).txt"
    
    echo "Generating comprehensive cluster report..."
    
    {
        echo "DataMesh Interactive Cluster Report"
        echo "============================="
        echo "Generated: $(date)"
        echo "Cluster ID: $TEST_DIR"
        echo ""
        
        echo "Cluster Configuration:"
        echo "- Bootstrap Node: $BOOTSTRAP_PEER_ID"
        echo "- Bootstrap Address: $BOOTSTRAP_ADDR"
        echo "- Total Nodes Started: ${#NODE_PIDS[@]}"
        echo "- Currently Running: $(get_active_node_count)"
        echo ""
        
        echo "Node Status:"
        for node_id in "${!NODE_PIDS[@]}"; do
            local pid="${NODE_PIDS[$node_id]}"
            local status="${NODE_STATUS[$node_id]}"
            echo "- $node_id: $status (PID: $pid)"
        done
        echo ""
        
        echo "Performance Summary:"
        if [ -f "$PERFORMANCE_DATA" ] && command -v jq &> /dev/null; then
            local total_ops=$(jq '.metrics | length' "$PERFORMANCE_DATA")
            echo "- Total operations recorded: $total_ops"
            
            local storage_count=$(jq '[.metrics[] | select(.type == "storage")] | length' "$PERFORMANCE_DATA")
            if [ "$storage_count" -gt 0 ]; then
                local avg_storage=$(jq '[.metrics[] | select(.type == "storage" and .name == "file_store_time")] | if length > 0 then (map(.value) | add / length) else 0 end' "$PERFORMANCE_DATA")
                echo "- Average storage time: ${avg_storage}s"
            fi
            
            local retrieval_count=$(jq '[.metrics[] | select(.type == "retrieval")] | length' "$PERFORMANCE_DATA")
            if [ "$retrieval_count" -gt 0 ]; then
                local avg_retrieval=$(jq '[.metrics[] | select(.type == "retrieval" and .name == "file_retrieve_time")] | if length > 0 then (map(.value) | add / length) else 0 end' "$PERFORMANCE_DATA")
                echo "- Average retrieval time: ${avg_retrieval}s"
            fi
        else
            echo "- Performance data not available"
        fi
        echo ""
        
        echo "Cluster Health:"
        echo "- Overall health: $(get_cluster_health_status)"
        echo "- Uptime: $(get_cluster_uptime)"
        echo ""
        
        echo "Files and Directories:"
        echo "- Test data: $DATA_DIR"
        echo "- Logs: $LOG_DIR"
        echo "- Results: $RESULTS_DIR"
        echo "- Monitoring: $MONITOR_DIR"
        
    } > "$report_file"
    
    success "Report generated: $(basename "$report_file")"
    echo ""
    echo "Report preview:"
    echo "==============="
    head -20 "$report_file"
    echo "... (see full report in file)"
}

# View logs interactively
view_logs_interactive() {
    print_section "View Cluster Logs" "${SYMBOLS[FILE]}"
    
    local log_files=($(find "$LOG_DIR" -name "*.log" -type f))
    
    if [ ${#log_files[@]} -eq 0 ]; then
        warning "No log files found"
        return
    fi
    
    echo "Available log files:"
    for i in "${!log_files[@]}"; do
        local file="${log_files[$i]}"
        local size=$(du -h "$file" | cut -f1)
        printf "  %d) %s (%s)\n" $((i+1)) "$(basename "$file")" "$size"
    done
    echo ""
    
    printf "Enter log number to view (1-%d): " "${#log_files[@]}"
    read -r log_choice
    
    if [ "$log_choice" -ge 1 ] && [ "$log_choice" -le "${#log_files[@]}" ]; then
        local selected_log="${log_files[$((log_choice-1))]}"
        
        echo ""
        echo "Viewing $(basename "$selected_log") (last 50 lines):"
        echo "=================================================="
        tail -50 "$selected_log"
        echo ""
        echo "=================================================="
        printf "Press 'f' for full log, 'r' for real-time tail, or Enter to return: "
        read -r view_choice
        
        case "$view_choice" in
            f|F)
                echo ""
                echo "Full log content:"
                echo "================="
                cat "$selected_log"
                ;;
            r|R)
                echo ""
                echo "Real-time log tail (Press Ctrl+C to stop):"
                echo "=========================================="
                tail -f "$selected_log"
                ;;
        esac
    else
        warning "Invalid selection"
    fi
}

# Additional utility functions...
confirm_action() {
    local message="$1"
    local default="$2"
    local default_char
    
    if [ "$default" = true ]; then
        default_char="Y/n"
    else
        default_char="y/N"
    fi
    
    printf "${COLORS[YELLOW]}?${COLORS[NC]} %s [%s]: " "$message" "$default_char"
    read -r response
    
    if [ -z "$response" ]; then
        echo "$default"
    elif [[ "$response" =~ ^[Yy]([Ee][Ss])?$ ]]; then
        echo true
    else
        echo false
    fi
}

# New interactive functions for testing latest features
interactive_search_files() {
    echo ""
    echo "Search Files Test"
    echo "================="
    
    printf "Enter search query: "
    read -r search_query
    
    printf "Enter file type filter (or press Enter for all): "
    read -r file_type
    
    echo ""
    echo "Searching for files..."
    local start_time=$(date +%s.%3N)
    
    local search_cmd=("$DFS_BINARY" 
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" 
        --bootstrap-addr "$BOOTSTRAP_ADDR" 
        --non-interactive 
        search "$search_query" --limit 10)
    
    if [ -n "$file_type" ]; then
        search_cmd+=(--file-type "$file_type")
    fi
    
    if "${search_cmd[@]}" > "$RESULTS_DIR/interactive_search.txt" 2>&1; then
        local end_time=$(date +%s.%3N)
        local duration=$(echo "$end_time - $start_time" | bc -l)
        
        success "Search completed successfully!"
        printf "  Duration: %.2fs\n" "$duration"
        echo ""
        echo "Search results:"
        cat "$RESULTS_DIR/interactive_search.txt"
    else
        error "Search failed"
        cat "$RESULTS_DIR/interactive_search.txt"
    fi
}

interactive_recent_files() {
    echo ""
    echo "Recent Files Test"
    echo "================="
    
    printf "Enter number of files to show (default 5): "
    read -r count
    count=${count:-5}
    
    printf "Enter days to look back (default 1): "
    read -r days
    days=${days:-1}
    
    echo ""
    echo "Getting recent files..."
    local start_time=$(date +%s.%3N)
    
    if "$DFS_BINARY" \
        --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
        --bootstrap-addr "$BOOTSTRAP_ADDR" \
        --non-interactive \
        recent --count "$count" --days "$days" > "$RESULTS_DIR/interactive_recent.txt" 2>&1; then
        
        local end_time=$(date +%s.%3N)
        local duration=$(echo "$end_time - $start_time" | bc -l)
        
        success "Recent files query completed!"
        printf "  Duration: %.2fs\n" "$duration"
        echo ""
        echo "Recent files:"
        cat "$RESULTS_DIR/interactive_recent.txt"
    else
        error "Recent files query failed"
        cat "$RESULTS_DIR/interactive_recent.txt"
    fi
}

interactive_batch_operations() {
    echo ""
    echo "Batch Operations Test"
    echo "===================="
    
    echo "Choose batch operation:"
    echo "1) Batch tag (dry run)"
    echo "2) Test batch put"
    echo "3) Test batch get"
    echo ""
    
    printf "Enter choice (1-3): "
    read -r batch_choice
    
    case "$batch_choice" in
        1)
            printf "Enter pattern to match files: "
            read -r pattern
            pattern=${pattern:-"*"}
            
            printf "Enter tags to add (comma-separated): "
            read -r add_tags
            add_tags=${add_tags:-"interactive-test"}
            
            echo ""
            echo "Running batch tag operation (dry run)..."
            if "$DFS_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --non-interactive \
                batch-tag "$pattern" --add-tags "$add_tags" --dry-run > "$RESULTS_DIR/interactive_batch_tag.txt" 2>&1; then
                success "Batch tag completed!"
                echo ""
                echo "Results:"
                cat "$RESULTS_DIR/interactive_batch_tag.txt"
            else
                error "Batch tag failed"
                cat "$RESULTS_DIR/interactive_batch_tag.txt"
            fi
            ;;
        2)
            # Create some test files for batch put
            local batch_dir="$DATA_DIR/interactive_batch"
            mkdir -p "$batch_dir"
            
            for i in {1..3}; do
                echo "Interactive batch test file $i" > "$batch_dir/batch_test_$i.txt"
            done
            
            echo ""
            echo "Created 3 test files, running batch put..."
            if timeout 60 "$DFS_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --non-interactive \
                batch-put "batch_test_*.txt" --base-dir "$batch_dir" --parallel 2 > "$RESULTS_DIR/interactive_batch_put.txt" 2>&1; then
                success "Batch put completed!"
                echo ""
                echo "Results:"
                tail -10 "$RESULTS_DIR/interactive_batch_put.txt"
            else
                error "Batch put failed"
                tail -10 "$RESULTS_DIR/interactive_batch_put.txt"
            fi
            ;;
        3)
            printf "Enter pattern for files to download: "
            read -r dl_pattern
            dl_pattern=${dl_pattern:-"batch_test*"}
            
            local download_dir="$RESULTS_DIR/interactive_batch_download"
            mkdir -p "$download_dir"
            
            echo ""
            echo "Running batch get..."
            if timeout 60 "$DFS_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --non-interactive \
                batch-get "$dl_pattern" "$download_dir" --parallel 2 > "$RESULTS_DIR/interactive_batch_get.txt" 2>&1; then
                success "Batch get completed!"
                echo ""
                echo "Downloaded files:"
                ls -la "$download_dir" 2>/dev/null || echo "No files downloaded"
            else
                error "Batch get failed"
                tail -10 "$RESULTS_DIR/interactive_batch_get.txt"
            fi
            ;;
        *)
            warning "Invalid choice"
            ;;
    esac
}

interactive_health_operations() {
    echo ""
    echo "Health & Maintenance Operations"
    echo "==============================="
    
    echo "Choose operation:"
    echo "1) Check cluster health"
    echo "2) Run storage stats"
    echo "3) Test repair operations"
    echo "4) Test cleanup operations"
    echo "5) Run performance benchmark"
    echo "6) Check quota usage"
    echo ""
    
    printf "Enter choice (1-6): "
    read -r health_choice
    
    case "$health_choice" in
        1)
            echo ""
            echo "Checking cluster health..."
            if timeout 30 "$DFS_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --non-interactive \
                health > "$RESULTS_DIR/interactive_health.txt" 2>&1; then
                success "Health check completed!"
                echo ""
                cat "$RESULTS_DIR/interactive_health.txt"
            else
                error "Health check failed"
                cat "$RESULTS_DIR/interactive_health.txt"
            fi
            ;;
        2)
            echo ""
            echo "Getting storage statistics..."
            if "$DFS_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --non-interactive \
                stats > "$RESULTS_DIR/interactive_stats.txt" 2>&1; then
                success "Stats retrieved!"
                echo ""
                cat "$RESULTS_DIR/interactive_stats.txt"
            else
                error "Stats retrieval failed"
                cat "$RESULTS_DIR/interactive_stats.txt"
            fi
            ;;
        3)
            echo ""
            echo "Testing repair operations..."
            if timeout 30 "$DFS_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --non-interactive \
                repair --auto --threshold 70 > "$RESULTS_DIR/interactive_repair.txt" 2>&1; then
                success "Repair test completed!"
                echo ""
                cat "$RESULTS_DIR/interactive_repair.txt"
            else
                error "Repair test failed"
                cat "$RESULTS_DIR/interactive_repair.txt"
            fi
            ;;
        4)
            echo ""
            echo "Testing cleanup operations (dry run)..."
            if "$DFS_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --non-interactive \
                cleanup --orphaned --duplicates --dry-run > "$RESULTS_DIR/interactive_cleanup.txt" 2>&1; then
                success "Cleanup test completed!"
                echo ""
                cat "$RESULTS_DIR/interactive_cleanup.txt"
            else
                error "Cleanup test failed"
                cat "$RESULTS_DIR/interactive_cleanup.txt"
            fi
            ;;
        5)
            echo ""
            printf "Enter benchmark duration in seconds (default 5): "
            read -r duration
            duration=${duration:-5}
            
            echo "Running performance benchmark ($duration seconds)..."
            if timeout $((duration + 30)) "$DFS_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --non-interactive \
                benchmark --storage --duration "$duration" > "$RESULTS_DIR/interactive_benchmark.txt" 2>&1; then
                success "Benchmark completed!"
                echo ""
                cat "$RESULTS_DIR/interactive_benchmark.txt"
            else
                error "Benchmark failed"
                cat "$RESULTS_DIR/interactive_benchmark.txt"
            fi
            ;;
        6)
            echo ""
            echo "Checking quota usage..."
            if "$DFS_BINARY" \
                --bootstrap-peer "$BOOTSTRAP_PEER_ID" \
                --bootstrap-addr "$BOOTSTRAP_ADDR" \
                --non-interactive \
                quota --usage > "$RESULTS_DIR/interactive_quota.txt" 2>&1; then
                success "Quota check completed!"
                echo ""
                cat "$RESULTS_DIR/interactive_quota.txt"
            else
                error "Quota check failed"
                cat "$RESULTS_DIR/interactive_quota.txt"
            fi
            ;;
        *)
            warning "Invalid choice"
            ;;
    esac
}

# Export functions for use in main script
export -f monitor_network_health_interactive
export -f test_file_operations_interactive
export -f interactive_store_file
export -f interactive_retrieve_file
export -f interactive_list_files
export -f interactive_stress_test
export -f interactive_search_files
export -f interactive_recent_files
export -f interactive_batch_operations
export -f interactive_health_operations
export -f show_performance_metrics
export -f start_new_node_interactive
export -f stop_node_interactive
export -f generate_report_interactive
export -f view_logs_interactive
export -f confirm_action
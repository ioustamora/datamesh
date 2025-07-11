# DataMesh CLI fish completion

# Global options
complete -c datamesh -s f -l format -d "Output format" -xa "table json compact csv"
complete -c datamesh -s v -l verbose -d "Verbose output"
complete -c datamesh -l no-color -d "Disable colored output"
complete -c datamesh -s c -l config -d "Configuration file" -r
complete -c datamesh -s i -l interactive -d "Interactive mode"
complete -c datamesh -l dry-run -d "Show what would be done"
complete -c datamesh -s h -l help -d "Show help"

# Main commands
complete -c datamesh -f -n "__fish_use_subcommand" -a "file" -d "File operations"
complete -c datamesh -f -n "__fish_use_subcommand" -a "network" -d "Network operations"
complete -c datamesh -f -n "__fish_use_subcommand" -a "system" -d "System operations"
complete -c datamesh -f -n "__fish_use_subcommand" -a "governance" -d "Governance operations"
complete -c datamesh -f -n "__fish_use_subcommand" -a "interactive" -d "Start interactive shell"
complete -c datamesh -f -n "__fish_use_subcommand" -a "service" -d "Service management"
complete -c datamesh -f -n "__fish_use_subcommand" -a "status" -d "Show system status"
complete -c datamesh -f -n "__fish_use_subcommand" -a "guide" -d "Getting started guide"

# File subcommands
complete -c datamesh -f -n "__fish_seen_subcommand_from file" -a "put" -d "Store a file"
complete -c datamesh -f -n "__fish_seen_subcommand_from file" -a "get" -d "Retrieve a file"
complete -c datamesh -f -n "__fish_seen_subcommand_from file" -a "list" -d "List files"
complete -c datamesh -f -n "__fish_seen_subcommand_from file" -a "search" -d "Search files"
complete -c datamesh -f -n "__fish_seen_subcommand_from file" -a "batch" -d "Batch operations"
complete -c datamesh -f -n "__fish_seen_subcommand_from file" -a "share" -d "Share files"

# File put options
complete -c datamesh -n "__fish_seen_subcommand_from file; and __fish_seen_subcommand_from put" -l name -d "Custom file name"
complete -c datamesh -n "__fish_seen_subcommand_from file; and __fish_seen_subcommand_from put" -l tags -d "File tags"
complete -c datamesh -n "__fish_seen_subcommand_from file; and __fish_seen_subcommand_from put" -l public-key -d "Public key for encryption"

# File get options
complete -c datamesh -n "__fish_seen_subcommand_from file; and __fish_seen_subcommand_from get" -s o -l output -d "Output path" -r
complete -c datamesh -n "__fish_seen_subcommand_from file; and __fish_seen_subcommand_from get" -l private-key -d "Private key for decryption"

# Network subcommands
complete -c datamesh -f -n "__fish_seen_subcommand_from network" -a "peers" -d "Show peer information"
complete -c datamesh -f -n "__fish_seen_subcommand_from network" -a "health" -d "Network health check"
complete -c datamesh -f -n "__fish_seen_subcommand_from network" -a "topology" -d "Network topology"
complete -c datamesh -f -n "__fish_seen_subcommand_from network" -a "bandwidth" -d "Bandwidth testing"
complete -c datamesh -f -n "__fish_seen_subcommand_from network" -a "bootstrap" -d "Bootstrap management"

# System subcommands
complete -c datamesh -f -n "__fish_seen_subcommand_from system" -a "config" -d "Configuration management"
complete -c datamesh -f -n "__fish_seen_subcommand_from system" -a "stats" -d "Statistics and metrics"
complete -c datamesh -f -n "__fish_seen_subcommand_from system" -a "storage" -d "Storage management"
complete -c datamesh -f -n "__fish_seen_subcommand_from system" -a "api" -d "API server management"
complete -c datamesh -f -n "__fish_seen_subcommand_from system" -a "benchmark" -d "Run benchmarks"

# Governance subcommands
complete -c datamesh -f -n "__fish_seen_subcommand_from governance" -a "user" -d "User management"
complete -c datamesh -f -n "__fish_seen_subcommand_from governance" -a "proposal" -d "Proposal management"
complete -c datamesh -f -n "__fish_seen_subcommand_from governance" -a "vote" -d "Vote on proposals"
complete -c datamesh -f -n "__fish_seen_subcommand_from governance" -a "economics" -d "Economic operations"

# Service subcommands
complete -c datamesh -f -n "__fish_seen_subcommand_from service" -a "start" -d "Start service"
complete -c datamesh -f -n "__fish_seen_subcommand_from service" -a "stop" -d "Stop service"
complete -c datamesh -f -n "__fish_seen_subcommand_from service" -a "restart" -d "Restart service"
complete -c datamesh -f -n "__fish_seen_subcommand_from service" -a "status" -d "Service status"
complete -c datamesh -f -n "__fish_seen_subcommand_from service" -a "logs" -d "Show service logs"

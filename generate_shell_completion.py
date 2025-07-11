#!/usr/bin/env python3
"""
Shell Completion Generator for DataMesh CLI

This script generates shell completion files for bash, zsh, and fish
based on the DataMesh CLI command structure.
"""

import os
import json
from pathlib import Path

# DataMesh command structure
COMMANDS = {
    "file": {
        "description": "File operations",
        "subcommands": {
            "put": {
                "description": "Store a file in the network",
                "options": ["--name", "--tags", "--public-key"],
                "args": ["<file_path>"]
            },
            "get": {
                "description": "Retrieve a file from the network", 
                "options": ["--output", "--private-key"],
                "args": ["<identifier>"]
            },
            "list": {
                "description": "List files",
                "options": ["--tags", "--public-key", "--long"],
                "args": []
            },
            "search": {
                "description": "Search files",
                "options": ["--content", "--limit"],
                "args": ["<query>"]
            },
            "batch": {
                "description": "Batch operations",
                "subcommands": {
                    "put": {"options": ["--preserve-structure"], "args": ["<patterns>..."]},
                    "get": {"options": ["--output-dir"], "args": ["<patterns>..."]},
                    "tag": {"options": ["--add", "--remove"], "args": ["<patterns>..."]}
                }
            },
            "share": {
                "description": "Share files",
                "options": ["--with", "--public", "--expires"],
                "args": ["<file>"]
            }
        }
    },
    "network": {
        "description": "Network operations",
        "subcommands": {
            "peers": {
                "description": "Show peer information",
                "options": ["--long", "--status"],
                "args": []
            },
            "health": {
                "description": "Network health check",
                "options": ["--full", "--monitor"],
                "args": []
            },
            "topology": {
                "description": "Network topology analysis",
                "options": ["--routing", "--output"],
                "args": []
            },
            "bandwidth": {
                "description": "Bandwidth testing",
                "options": ["--duration"],
                "args": ["[peer]"]
            },
            "bootstrap": {
                "description": "Bootstrap node management",
                "subcommands": {
                    "start": {"options": ["--port"], "args": []},
                    "stop": {"options": [], "args": []},
                    "list": {"options": [], "args": []},
                    "add": {"options": [], "args": ["<address>"]},
                    "remove": {"options": [], "args": ["<peer_id>"]}
                }
            }
        }
    },
    "system": {
        "description": "System operations",
        "subcommands": {
            "config": {
                "description": "Configuration management",
                "subcommands": {
                    "show": {"options": [], "args": ["[section]"]},
                    "set": {"options": [], "args": ["<key>", "<value>"]},
                    "get": {"options": [], "args": ["<key>"]},
                    "init": {"options": ["--output", "--force"], "args": []},
                    "validate": {"options": [], "args": ["[file]"]}
                }
            },
            "stats": {
                "description": "Statistics and metrics",
                "options": ["--long", "--watch"],
                "args": []
            },
            "storage": {
                "description": "Storage management",
                "subcommands": {
                    "cleanup": {"options": ["--orphaned", "--compact"], "args": []},
                    "repair": {"options": ["--integrity", "--fix"], "args": []},
                    "optimize": {"options": ["--defrag", "--rebalance"], "args": []},
                    "quota": {"options": ["--long"], "args": []}
                }
            },
            "api": {
                "description": "API server management",
                "subcommands": {
                    "start": {"options": ["--port", "--bind"], "args": []},
                    "stop": {"options": [], "args": []},
                    "status": {"options": [], "args": []},
                    "docs": {"options": ["--format", "--output"], "args": []}
                }
            },
            "benchmark": {
                "description": "Run benchmarks",
                "options": ["--test-type", "--duration"],
                "args": []
            }
        }
    },
    "governance": {
        "description": "Governance operations",
        "subcommands": {
            "user": {
                "description": "User management",
                "subcommands": {
                    "register": {"options": ["--password"], "args": ["<email>"]},
                    "login": {"options": ["--password"], "args": ["<email>"]},
                    "profile": {"options": [], "args": ["[user_id]"]},
                    "update": {"options": ["--email", "--password"], "args": []}
                }
            },
            "proposal": {
                "description": "Proposal management",
                "subcommands": {
                    "list": {"options": ["--active", "--proposal-type"], "args": []},
                    "create": {"options": ["--proposal-type"], "args": ["<title>", "<description>"]},
                    "show": {"options": [], "args": ["<proposal_id>"]}
                }
            },
            "vote": {
                "description": "Vote on proposals",
                "options": ["--reason"],
                "args": ["<proposal_id>", "<vote>"]
            },
            "economics": {
                "description": "Economic operations",
                "subcommands": {
                    "balance": {"options": [], "args": ["[user_id]"]},
                    "transfer": {"options": ["--memo"], "args": ["<to>", "<amount>"]},
                    "stake": {"options": ["--duration"], "args": ["<amount>"]},
                    "history": {"options": ["--limit"], "args": []}
                }
            }
        }
    },
    # Quick commands
    "interactive": {
        "description": "Start interactive shell",
        "options": [],
        "args": []
    },
    "service": {
        "description": "Service management",
        "subcommands": {
            "start": {"options": ["--foreground"], "args": []},
            "stop": {"options": [], "args": []},
            "restart": {"options": [], "args": []},
            "status": {"options": [], "args": []},
            "logs": {"options": ["--follow", "--lines"], "args": []}
        }
    },
    "status": {
        "description": "Show system status",
        "options": [],
        "args": []
    },
    "guide": {
        "description": "Getting started guide",
        "options": [],
        "args": ["[topic]"]
    }
}

# Global options available for all commands
GLOBAL_OPTIONS = [
    "--format", "-f",
    "--verbose", "-v",
    "--no-color",
    "--config", "-c", 
    "--interactive", "-i",
    "--dry-run",
    "--help", "-h"
]

def generate_bash_completion():
    """Generate bash completion script"""
    script = '''#!/bin/bash
# DataMesh CLI bash completion

_datamesh_completion() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Global options
    local global_opts="--format -f --verbose -v --no-color --config -c --interactive -i --dry-run --help -h"
    
    # Main commands
    local commands="file network system governance interactive service status guide"
    
    # Handle global options
    case ${prev} in
        --format|-f)
            COMPREPLY=( $(compgen -W "table json compact csv" -- ${cur}) )
            return 0
            ;;
        --config|-c)
            COMPREPLY=( $(compgen -f -- ${cur}) )
            return 0
            ;;
    esac
    
    # Get command path
    local cmd_path=""
    local i=1
    while [[ $i -lt $COMP_CWORD ]]; do
        local word="${COMP_WORDS[$i]}"
        if [[ ! "$word" =~ ^- ]]; then
            if [[ -z "$cmd_path" ]]; then
                cmd_path="$word"
            else
                cmd_path="$cmd_path $word"
            fi
        fi
        ((i++))
    done
    
    # Complete based on command path
    case "$cmd_path" in
        "")
            COMPREPLY=( $(compgen -W "$commands $global_opts" -- ${cur}) )
            ;;
        "file")
            COMPREPLY=( $(compgen -W "put get list search batch share" -- ${cur}) )
            ;;
        "file put")
            case ${prev} in
                --name|--tags|--public-key) return 0 ;;
                *) COMPREPLY=( $(compgen -f -- ${cur}) ) ;;
            esac
            ;;
        "file get")
            case ${prev} in
                --output|-o) COMPREPLY=( $(compgen -f -- ${cur}) ) ;;
                --private-key) return 0 ;;
            esac
            ;;
        "file list")
            COMPREPLY=( $(compgen -W "--tags --public-key --long" -- ${cur}) )
            ;;
        "file search")
            COMPREPLY=( $(compgen -W "--content --limit" -- ${cur}) )
            ;;
        "file batch")
            COMPREPLY=( $(compgen -W "put get tag" -- ${cur}) )
            ;;
        "file share")
            COMPREPLY=( $(compgen -W "--with --public --expires" -- ${cur}) )
            ;;
        "network")
            COMPREPLY=( $(compgen -W "peers health topology bandwidth bootstrap" -- ${cur}) )
            ;;
        "network peers")
            COMPREPLY=( $(compgen -W "--long --status" -- ${cur}) )
            ;;
        "network health")
            COMPREPLY=( $(compgen -W "--full --monitor" -- ${cur}) )
            ;;
        "network topology")
            COMPREPLY=( $(compgen -W "--routing --output" -- ${cur}) )
            ;;
        "network bandwidth")
            COMPREPLY=( $(compgen -W "--duration" -- ${cur}) )
            ;;
        "network bootstrap")
            COMPREPLY=( $(compgen -W "start stop list add remove" -- ${cur}) )
            ;;
        "system")
            COMPREPLY=( $(compgen -W "config stats storage api benchmark" -- ${cur}) )
            ;;
        "system config")
            COMPREPLY=( $(compgen -W "show set get init validate" -- ${cur}) )
            ;;
        "system stats")
            COMPREPLY=( $(compgen -W "--long --watch" -- ${cur}) )
            ;;
        "system storage")
            COMPREPLY=( $(compgen -W "cleanup repair optimize quota" -- ${cur}) )
            ;;
        "system api")
            COMPREPLY=( $(compgen -W "start stop status docs" -- ${cur}) )
            ;;
        "governance")
            COMPREPLY=( $(compgen -W "user proposal vote economics" -- ${cur}) )
            ;;
        "governance user")
            COMPREPLY=( $(compgen -W "register login profile update" -- ${cur}) )
            ;;
        "governance proposal")
            COMPREPLY=( $(compgen -W "list create show" -- ${cur}) )
            ;;
        "governance economics")
            COMPREPLY=( $(compgen -W "balance transfer stake history" -- ${cur}) )
            ;;
        "service")
            COMPREPLY=( $(compgen -W "start stop restart status logs" -- ${cur}) )
            ;;
        *)
            COMPREPLY=( $(compgen -W "$global_opts" -- ${cur}) )
            ;;
    esac
}

complete -F _datamesh_completion datamesh
'''
    return script

def generate_zsh_completion():
    """Generate zsh completion script"""
    script = '''#compdef datamesh

# DataMesh CLI zsh completion

_datamesh() {
    local context state line
    typeset -A opt_args
    
    local global_opts=(
        '(-f --format)'{-f,--format}'[Output format]:format:(table json compact csv)'
        '(-v --verbose)'{-v,--verbose}'[Verbose output]'
        '--no-color[Disable colored output]'
        '(-c --config)'{-c,--config}'[Configuration file]:file:_files'
        '(-i --interactive)'{-i,--interactive}'[Interactive mode]'
        '--dry-run[Show what would be done]'
        '(-h --help)'{-h,--help}'[Show help]'
    )
    
    _arguments -C \
        $global_opts \
        '1: :->commands' \
        '*: :->args' && return 0
    
    case $state in
        commands)
            local commands=(
                'file:File operations'
                'network:Network operations'
                'system:System operations'
                'governance:Governance operations'
                'interactive:Start interactive shell'
                'service:Service management'
                'status:Show system status'
                'guide:Getting started guide'
            )
            _describe 'commands' commands
            ;;
        args)
            case $words[2] in
                file)
                    local file_commands=(
                        'put:Store a file'
                        'get:Retrieve a file'
                        'list:List files'
                        'search:Search files'
                        'batch:Batch operations'
                        'share:Share files'
                    )
                    _describe 'file commands' file_commands
                    ;;
                network)
                    local network_commands=(
                        'peers:Show peer information'
                        'health:Network health check'
                        'topology:Network topology'
                        'bandwidth:Bandwidth testing'
                        'bootstrap:Bootstrap management'
                    )
                    _describe 'network commands' network_commands
                    ;;
                system)
                    local system_commands=(
                        'config:Configuration management'
                        'stats:Statistics and metrics'
                        'storage:Storage management'
                        'api:API server management'
                        'benchmark:Run benchmarks'
                    )
                    _describe 'system commands' system_commands
                    ;;
                governance)
                    local governance_commands=(
                        'user:User management'
                        'proposal:Proposal management'
                        'vote:Vote on proposals'
                        'economics:Economic operations'
                    )
                    _describe 'governance commands' governance_commands
                    ;;
                service)
                    local service_commands=(
                        'start:Start service'
                        'stop:Stop service'
                        'restart:Restart service'
                        'status:Service status'
                        'logs:Show service logs'
                    )
                    _describe 'service commands' service_commands
                    ;;
            esac
            ;;
    esac
}

_datamesh
'''
    return script

def generate_fish_completion():
    """Generate fish completion script"""
    script = '''# DataMesh CLI fish completion

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
'''
    return script

def write_completion_files():
    """Write completion files to disk"""
    completions_dir = Path("completions")
    completions_dir.mkdir(exist_ok=True)
    
    # Bash completion
    bash_file = completions_dir / "datamesh.bash"
    with open(bash_file, "w") as f:
        f.write(generate_bash_completion())
    print(f"Generated bash completion: {bash_file}")
    
    # Zsh completion
    zsh_file = completions_dir / "_datamesh"
    with open(zsh_file, "w") as f:
        f.write(generate_zsh_completion())
    print(f"Generated zsh completion: {zsh_file}")
    
    # Fish completion
    fish_file = completions_dir / "datamesh.fish"
    with open(fish_file, "w") as f:
        f.write(generate_fish_completion())
    print(f"Generated fish completion: {fish_file}")
    
    # Installation instructions
    install_file = completions_dir / "INSTALL.md"
    with open(install_file, "w") as f:
        f.write("""# Shell Completion Installation

## Bash
Copy the bash completion file to your bash completions directory:
```bash
# On most Linux systems:
sudo cp datamesh.bash /etc/bash_completion.d/
# Or user-specific:
mkdir -p ~/.local/share/bash-completion/completions
cp datamesh.bash ~/.local/share/bash-completion/completions/datamesh
```

## Zsh
Copy the zsh completion file to your zsh completions directory:
```bash
# Add to your .zshrc:
fpath=(~/.local/share/zsh/completions $fpath)
autoload -U compinit && compinit

# Copy the completion file:
mkdir -p ~/.local/share/zsh/completions
cp _datamesh ~/.local/share/zsh/completions/
```

## Fish
Copy the fish completion file to your fish completions directory:
```bash
mkdir -p ~/.config/fish/completions
cp datamesh.fish ~/.config/fish/completions/
```

## Testing
After installation, restart your shell or source your configuration file.
Test completions by typing:
```bash
datamesh <TAB>
datamesh file <TAB>
datamesh network <TAB>
```
""")
    print(f"Generated installation instructions: {install_file}")

if __name__ == "__main__":
    write_completion_files()
    print("Shell completion files generated successfully!")
    print("See completions/INSTALL.md for installation instructions.")
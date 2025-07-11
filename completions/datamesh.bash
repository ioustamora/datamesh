#!/bin/bash
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

#!/usr/bin/env sh
set -eu

usage() {
    cat <<'EOF'
Chaos Game Mode installer wrapper

Usage:
  ./install.sh [install|update|uninstall] [options]

Options:
  --install-dir PATH     Install directory passed to install.ps1
  --no-path              Do not add/remove the install dir from the user PATH
  -h, --help             Show this help

This wrapper is for Git Bash/MSYS/Cygwin on Windows. It delegates to install.ps1.
EOF
}

case "${OS:-}" in
    Windows_NT) ;;
    *)
        uname_value="$(uname -s 2>/dev/null || true)"
        case "$uname_value" in
            MINGW*|MSYS*|CYGWIN*) ;;
            *)
                echo "Chaos Game Mode is currently a Windows app. Run this from Git Bash/MSYS/Cygwin on Windows." >&2
                exit 1
                ;;
        esac
        ;;
esac

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
action="Install"
install_dir=""
no_path=0

while [ "$#" -gt 0 ]; do
    case "$1" in
        install|Install)
            action="Install"
            shift
            ;;
        update|Update)
            action="Update"
            shift
            ;;
        uninstall|Uninstall)
            action="Uninstall"
            shift
            ;;
        --install-dir)
            if [ "$#" -lt 2 ]; then
                echo "--install-dir requires a path" >&2
                exit 1
            fi
            install_dir=$2
            shift 2
            ;;
        --no-path)
            no_path=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

find_powershell() {
    for candidate in pwsh.exe powershell.exe pwsh powershell; do
        if command -v "$candidate" >/dev/null 2>&1; then
            command -v "$candidate"
            return 0
        fi
    done
    return 1
}

to_windows_path() {
    path=$1
    case "$path" in
        [A-Za-z]:\\*) printf '%s\n' "$path" ;;
        *)
            if command -v cygpath >/dev/null 2>&1; then
                cygpath -w "$path"
            else
                printf '%s\n' "$path"
            fi
            ;;
    esac
}

powershell_bin=$(find_powershell) || {
    echo "PowerShell was not found. Install PowerShell or run install.ps1 directly." >&2
    exit 1
}

ps_script=$(to_windows_path "$script_dir/install.ps1")

set -- -NoProfile -ExecutionPolicy Bypass -File "$ps_script" -Action "$action"

if [ -n "$install_dir" ]; then
    install_dir=$(to_windows_path "$install_dir")
    set -- "$@" -InstallDir "$install_dir"
fi

if [ "$no_path" -eq 1 ]; then
    set -- "$@" -NoPath
fi

exec "$powershell_bin" "$@"

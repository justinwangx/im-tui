#!/usr/bin/env bash
set -e

main() {
    BIN_DIR=${BIN_DIR-"$HOME/.bin"}
    mkdir -p "$BIN_DIR"

    # Check OS - only support macOS
    PLATFORM="$(uname -s)"
    if [[ "$PLATFORM" != "Darwin" ]]; then
        echo "Error: im only supports macOS as it interfaces with Messages.app"
        exit 1
    fi

    # Add binary directory to PATH in the appropriate shell config
    case $SHELL in
    */zsh)
        PROFILE=$HOME/.zshrc
        ;;
    */bash)
        PROFILE=$HOME/.bashrc
        ;;
    */fish)
        PROFILE=$HOME/.config/fish/config.fish
        ;;
    */ash)
        PROFILE=$HOME/.profile
        ;;
    *)
        echo "Could not detect shell, manually add ${BIN_DIR} to your PATH."
        PROFILE=""
        ;;
    esac

    if [[ -n "$PROFILE" && ":$PATH:" != *":${BIN_DIR}:"* ]]; then
        echo "Adding $BIN_DIR to PATH in $PROFILE"
        echo >> "$PROFILE" && echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$PROFILE"
    fi

    # Determine architecture
    ARCHITECTURE="$(uname -m)"
    if [ "${ARCHITECTURE}" = "x86_64" ]; then
        # Redirect stderr to /dev/null to avoid printing errors if non-Rosetta
        if [ "$(sysctl -n sysctl.proc_translated 2>/dev/null)" = "1" ]; then
            ARCHITECTURE="aarch64" # Rosetta
        else
            ARCHITECTURE="x86_64" # Intel
        fi
    elif [ "${ARCHITECTURE}" = "arm64" ] || [ "${ARCHITECTURE}" = "aarch64" ]; then
        ARCHITECTURE="aarch64" # Arm
    else
        ARCHITECTURE="x86_64" # Default to x86_64 for other architectures
    fi

    BINARY_URL="https://github.com/justinwangx/im-tui/releases/latest/download/im-${ARCHITECTURE}-apple-darwin"
    echo "Downloading from: $BINARY_URL"

    echo "Downloading latest binary..."
    ensure curl -L "$BINARY_URL" -o "$BIN_DIR/im"
    chmod +x "$BIN_DIR/im"

    echo "im installed successfully! ✅"
    
    if [[ -n "$PROFILE" ]]; then
        echo "NOTE: You may need to restart your terminal or run 'source $PROFILE' to update your PATH"
    fi
}

# Run a command that should never fail. If the command fails execution
# will immediately terminate with an error showing the failing
# command.
ensure() {
    if ! "$@"; then 
        echo "Error: command failed: $*" 
        exit 1
    fi
}

main "$@" || exit 1
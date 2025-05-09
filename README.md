# gf

a cli for viewing texts from loved ones (macOS)

## Installation

You can install `gf` by running the following command in your terminal:

```sh
curl -fsSL https://raw.githubusercontent.com/justinwangx/gf-cli/main/install.sh | sh -
```

If you have Rust and Cargo installed, you can install from source:

```sh
git clone https://github.com/justinwangx/gf-cli.git
cd gf-cli
cargo install --path .
```

## Usage

### Configure a default contact

```sh
# Set a default contact
gf --set 4163330321
# Set the display name for the default contact
gf --name "Aileen"
```

### Check the last received message

```sh
# See the last message received from your default contact
gf

# Or specify a contact for one-time use
gf --contact 6137770408
```

### Managing multiple contacts

```sh
# Add a named contact
gf add freeman 6137770408 --display-name "Freeman"

# List all configured contacts
gf list

# Check the last message received from a named contact (case-insensitive)
gf freeman

# Remove a contact
gf remove freeman

# Show configuration file location (for manual editing)
gf config
```

## How it works

`gf` reads directly from the macOS Messages database to retrieve the last message received from the specified contact (not messages you sent to them). It displays the message along with a human-readable timestamp.

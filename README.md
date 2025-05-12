# im-tui

text the people you love from the terminal (macOS)

## Installation

You can install `im` by running the following command in your terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/justinwangx/im-tui/main/install.sh | sh -
```

Or build from source:

```bash
git clone https://github.com/justinwangx/im-tui.git
cd im-tui
cargo install --path .
```

## Usage

### Basic Usage

Message your default contact:

```bash
im
```

Message a specific contact (one-time use):

```bash
im --contact 3015551234
```

Configure your default contact:

```bash
im --set 4163330321
```

Configure the display name for your default contact:

```bash
im --name "Aileen"
```

### Contact Management

Message a specific contact:

```bash
im freeman
```

Add a new contact:

```bash
im add freeman 6137770408
```

Add a new contact with display-name:

```bash
im add freeman 6137770408 --display-name "Freeman"
```

List all contacts:

```bash
im contacts
```

Remove a contact:

```bash
im remove freeman
```

## License

MIT

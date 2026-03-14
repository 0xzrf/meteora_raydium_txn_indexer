# MET-Ray Indexer

A Rust-based indexer for Laserstream that supports both mainnet and devnet environments.

## Setup

### Build the Project

```bash
cargo build
```

### Environment Configuration

Create a `.env` file in the project root with the following variables:

```shellscript
API_KEY=                    # Laserstream API key
DB_CONNECTION=              # Database connection URL
LASERSTREAM_URL=            # Mainnet URL for Laserstream
LASERSTREAM_DEVNET_URL=     # Devnet URL for Laserstream
IS_DEVNET=                  # Set to true for devnet environment, false for mainnet
DEBUGGING=                  # Enable debugging logs (true/false)
FEE_RECEIVER=               # Fee receiver address to index fee transfer to it
```

## Running the Application

### Development Mode

```bash
cargo run
```

### Production Mode

For optimized production deployment:

```bash
cargo run --release
```

## Testing

Run tests with different configurations:

```bash
# Run all tests (without logs)
cargo test

# Run tests with output logs
cargo test -- --show-output

# Run ignored tests (WARNING: may delete database data)
cargo test -- --ignored
```

**⚠️ Warning:** Tests marked as `ignored` may perform destructive operations on your database. Use with caution and ensure you have proper backups.

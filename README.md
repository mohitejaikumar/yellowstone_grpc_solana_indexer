# Solana Indexer

A high-performance, scalable Solana blockchain data indexer built in Rust. This project streams real-time Solana blockchain data via Yellowstone gRPC, processes it through Redis, and stores it in ScyllaDB for efficient querying and analysis.

## Architecture

The indexer consists of three main components working together in a pipeline:

```
Yellowstone gRPC → Redis Stream → ScyllaDB
      ↓              ↓              ↓
   Engine      DB Processor    Storage
```

### Components

- **Engine** (`engine/`): Connects to Yellowstone gRPC endpoints, subscribes to DeFi transactions, and streams data to Redis
- **DB Processor** (`db_processor/`): Consumes messages from Redis streams and writes processed data to ScyllaDB
- **Yellowstone gRPC** (`yellowstone_gRPC/`): Custom gRPC client library for interacting with Solana's Yellowstone gRPC interface

## Features

- **Real-time Data Streaming**: Live blockchain data ingestion via Yellowstone gRPC
- **Scalable Processing**: Redis-based message queuing for reliable data processing
- **High-Performance Storage**: ScyllaDB for fast, distributed data storage
- **DeFi Focus**: Specialized subscriptions for DeFi-related transactions
- **Type Safety**: Comprehensive Rust type system with automatic serialization/deserialization
- **Error Resilience**: Robust error handling and logging throughout the pipeline

## Project Structure

```
solana-indexer/
├── engine/                 # Data ingestion service
│   ├── src/
│   │   ├── main.rs        # Main engine application
│   │   └── config.rs      # Configuration management
│   └── Cargo.toml
├── db_processor/          # Data processing service
│   ├── src/
│   │   ├── main.rs        # Main processor application
│   │   ├── processor.rs   # Message processing logic
│   │   ├── scylla_client.rs   # ScyllaDB client
│   │   ├── redis_client.rs    # Redis client
│   │   ├── scylla_types.rs    # Database schema types
│   │   └── config.rs      # Configuration management
│   └── Cargo.toml
├── yellowstone_gRPC/      # Custom gRPC client library
│   ├── src/
│   │   ├── client.rs      # gRPC client implementation
│   │   ├── subscriptions.rs   # Subscription management
│   │   ├── types.rs       # Data type definitions
│   │   └── lib.rs         # Library interface
│   └── Cargo.toml
└── Cargo.toml            # Workspace configuration
```

## 🛠️ Prerequisites

- **Rust** (latest stable version)
- **Redis** server
- **ScyllaDB** cluster
- Access to a **Yellowstone gRPC** endpoint

## ⚙️ Configuration

The application uses environment variables for configuration. Create a `.env` file in the project root:

```bash
# Yellowstone gRPC Configuration
YELLOWSTONE_ENDPOINT=https://your-yellowstone-endpoint.com
YELLOWSTONE_TOKEN=your_optional_auth_token

# Redis Configuration
REDIS_URL=redis://localhost:6379

# ScyllaDB Configuration
SCYLLA_NODES=127.0.0.1:9042,127.0.0.1:9043,127.0.0.1:9044
```

### Environment Variables

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `YELLOWSTONE_ENDPOINT` | Yellowstone gRPC endpoint URL | ✅ | - |
| `YELLOWSTONE_TOKEN` | Authentication token for Yellowstone | ❌ | None |
| `REDIS_URL` | Redis connection string | ✅ | - |
| `SCYLLA_NODES` | Comma-separated ScyllaDB node addresses | ❌ | `127.0.0.1:9042` |

## 🚀 Quick Start

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd solana-indexer
   ```

2. **Install dependencies**
   ```bash
   cargo build --release
   ```

3. **Set up your environment**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. **Start Redis and ScyllaDB**
   ```bash
   # Redis
   redis-server
   
   # ScyllaDB (or use Docker)
   docker run --name scylla -p 9042:9042 -d scylladb/scylla
   ```

5. **Run the indexer**

   Start the engine (data ingestion):
   ```bash
   cargo run --bin engine
   ```

   In another terminal, start the database processor:
   ```bash
   cargo run --bin db_processor
   ```

## 📊 Data Schema

### Example Transaction Data

Here's an example of a structured `SolanaTransaction` log that the indexer processes:

```
SolanaTransaction:
  signature: wEfC4JFZj6pHadcbJwYaM6pumTyJA7WVTPMHvmmwuAbgeyuGMauDi6AfG8LRyt8kvrhttRG9AKupnwpr71ruKFY
  slot: 363515210
  is_vote: false
  index: 28
  success: true
  fee: Some(5000)
  compute_units_consumed: Some(34734)
  instructions: [
    TransactionInstruction:
      program_id: 11111111111111111111111111111111
      accounts: [
        "2xYWviZJpxhifFw2emoxn8eCVa9u67sdEbL7zhokvbLW",
        "35eNC63jS5ughGqjfkMzmQpRY412mVW6v3QvobRQUKii",
      ]
      data: AwAAAB0WRELKkUzYlJGHY2nIWCtH3XY1p8C51bApCV4YK1ZrIAAAAAAAAAA3Z2p6ZjI2OHBDb29DaXc0RWV1dWdFeHFvN25NSDY3QsZ4igMAAAAApQAAAAAAAAAG3fbh12Whk9nL4UbO63msHLSF7V9bN5E6jPWFfv8AqQ==

    TransactionInstruction:
      program_id: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
      accounts: [
        "35eNC63jS5ughGqjfkMzmQpRY412mVW6v3QvobRQUKii",
        "So11111111111111111111111111111111111111112",
        "2xYWviZJpxhifFw2emoxn8eCVa9u67sdEbL7zhokvbLW",
        "SysvarRent111111111111111111111111111111111",
      ]
      data: AQ==

    TransactionInstruction:
      program_id: 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8
      accounts: [
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "3BiUMV1gyEdzRZaWMET3mTCd6QDthJxDkKxXzrMGxgVR",
        "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
        "H22prfsjFfrXQbmdCVqMD53A8Usf9j7UcJ2XDPgmnz49",
        "DDkfWvyrd36rULgMvSYVYcPZRuFbfveEPBPuXT5Ki6yn",
        "7g5c5UBkoE2Yz9r92F85USaB9iEuyG1LX21twzsBB3bW",
        "CJfFpQcNRofS6juvJFFGCzLtmimuHbuLcMun67mCDPcX",
        "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX",
        "DQn1qK4qPKHM2Cx28rWZW8XtERs8Qss6qFNXVBbVHyei",
        "DssV8TvzW14iYrRFt8XqXyb2E2dCc71BFoAHq5mMSvkb",
        "D8H2YXmVUskcy9Ciz7P6rEJBavFr5sQ1dRaq3nip8tiY",
        "4Tprxbq8BBsLXvE63zCJyYp25Qu38RHEL5YijEk3jfPN",
        "4hyTTsqWV1Mi2LFc2cmS5BVodbSk7ecUGzcYmfgQiEfe",
        "FjbKafcCyaq1Eo5MDMd2LEb3G44XVpHDG8TCEuD6H8C9",
        "GEWZ6F8P3kLWcjMZuK9t5guNqvVddA1ikRVNgevkGEni",
        "35eNC63jS5ughGqjfkMzmQpRY412mVW6v3QvobRQUKii",
        "D7dmrhFvJXKww7wyEatV5ZNShE22nDA92NFhfndzYnEH",
        "2xYWviZJpxhifFw2emoxn8eCVa9u67sdEbL7zhokvbLW",
      ]
      data: CdZaawMAAAAAqL5vAAAAAAA=

    TransactionInstruction:
      program_id: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
      accounts: [
        "35eNC63jS5ughGqjfkMzmQpRY412mVW6v3QvobRQUKii",
        "2xYWviZJpxhifFw2emoxn8eCVa9u67sdEbL7zhokvbLW",
        "2xYWviZJpxhifFw2emoxn8eCVa9u67sdEbL7zhokvbLW",
      ]
      data: CQ==

  ]
  account_keys: [
    "2xYWviZJpxhifFw2emoxn8eCVa9u67sdEbL7zhokvbLW",
    "35eNC63jS5ughGqjfkMzmQpRY412mVW6v3QvobRQUKii",
    "3BiUMV1gyEdzRZaWMET3mTCd6QDthJxDkKxXzrMGxgVR",
    "H22prfsjFfrXQbmdCVqMD53A8Usf9j7UcJ2XDPgmnz49",
    "DDkfWvyrd36rULgMvSYVYcPZRuFbfveEPBPuXT5Ki6yn",
    "7g5c5UBkoE2Yz9r92F85USaB9iEuyG1LX21twzsBB3bW",
    "CJfFpQcNRofS6juvJFFGCzLtmimuHbuLcMun67mCDPcX",
    "DQn1qK4qPKHM2Cx28rWZW8XtERs8Qss6qFNXVBbVHyei",
    "DssV8TvzW14iYrRFt8XqXyb2E2dCc71BFoAHq5mMSvkb",
    "D8H2YXmVUskcy9Ciz7P6rEJBavFr5sQ1dRaq3nip8tiY",
    "4Tprxbq8BBsLXvE63zCJyYp25Qu38RHEL5YijEk3jfPN",
    "4hyTTsqWV1Mi2LFc2cmS5BVodbSk7ecUGzcYmfgQiEfe",
    "FjbKafcCyaq1Eo5MDMd2LEb3G44XVpHDG8TCEuD6H8C9",
    "D7dmrhFvJXKww7wyEatV5ZNShE22nDA92NFhfndzYnEH",
    "11111111111111111111111111111111",
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    "So11111111111111111111111111111111111111112",
    "SysvarRent111111111111111111111111111111111",
    "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
    "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
    "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX",
    "GEWZ6F8P3kLWcjMZuK9t5guNqvVddA1ikRVNgevkGEni",
  ]
  log_messages: [
    "Program 11111111111111111111111111111111 invoke [1]",
    "Program 11111111111111111111111111111111 success",
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]",
    "Program log: Instruction: InitializeAccount",
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 3443 of 602850 compute units",
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
    "Program 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8 invoke [1]",
    "Program log: ray_log: A9ZaawMAAAAAqL5vAAAAAAABAAAAAAAAANZaawMAAAAASBoQahoAAABAkzQxZwAAACx63wAAAAAA",
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
    "Program log: Instruction: Transfer",
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4736 of 583808 compute units",
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
    "Program log: Instruction: Transfer",
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 576603 compute units",
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
    "Program 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8 consumed 28226 of 599407 compute units",
    "Program 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8 success",
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]",
    "Program log: Instruction: CloseAccount",
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2915 of 571181 compute units",
    "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
  ]
  pre_balances: [1727976462, 0, 6124800, 23357760, 16258560, 2039280, 443209199920, 3591360, 101977920, 101977920, 79594560, 2039280, 2039280, 2039280, 1, 4674972223, 1126241608121, 1009200, 2500659929, 32325306034, 1506941574, 0]
  post_balances: [1670604208, 0, 6124800, 23357760, 16258560, 2039280, 443266567174, 3591360, 101977920, 101977920, 79594560, 2039280, 2039280, 2039280, 1, 4674972223, 1126241608121, 1009200, 2500659929, 32325306034, 1506941574, 0]
  timestamp: 2025-08-30 13:05:12.954036272 UTC
```

This example shows a DeFi transaction involving multiple programs including the System Program, Token Program, and what appears to be a Raydium swap. The transaction includes account creation, token transfers, and detailed execution logs.

### Transactions Table
```sql
CREATE TABLE transactions (
    signature TEXT PRIMARY KEY,
    slot BIGINT,
    is_vote BOOLEAN,
    tx_index BIGINT,
    success BOOLEAN,
    fee BIGINT,
    compute_units_consumed BIGINT,
    instructions_json TEXT,
    account_keys_json TEXT,
    log_messages_json TEXT,
    pre_balances_json TEXT,
    post_balances_json TEXT,
    timestamp_ms BIGINT
);
```

### Accounts Table
```sql
CREATE TABLE accounts (
    pubkey TEXT,
    slot BIGINT,
    lamports BIGINT,
    owner TEXT,
    executable BOOLEAN,
    rent_epoch BIGINT,
    data TEXT,
    write_version BIGINT,
    txn_signature TEXT,
    timestamp_ms BIGINT,
    PRIMARY KEY (pubkey, slot, write_version)
);
```

## 🔧 Development

### Building
```bash
# Build all components
cargo build

# Build with optimizations
cargo build --release
```

### Testing
```bash
# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Logging
The application uses structured logging with different levels:
- `RUST_LOG=info` - General information
- `RUST_LOG=debug` - Detailed debugging
- `RUST_LOG=trace` - Very verbose output

## 🏗️ Architecture Details

### Data Flow

1. **Ingestion**: The engine connects to Yellowstone gRPC and subscribes to DeFi transactions
2. **Streaming**: Real-time data is pushed to Redis streams for reliable queuing
3. **Processing**: The DB processor consumes messages from Redis and transforms them
4. **Storage**: Processed data is batch-written to ScyllaDB for efficient storage
5. **Acknowledgment**: Successfully processed messages are acknowledged in Redis

### Scalability Features

- **Horizontal Scaling**: Multiple processor instances can consume from the same Redis stream
- **Batch Processing**: Configurable batch sizes for optimal throughput
- **Consumer Groups**: Redis consumer groups ensure message delivery guarantees
- **Connection Pooling**: Efficient database connection management

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- Create an issue for bug reports or feature requests
- Check existing issues before creating new ones
- Provide detailed information about your environment and the problem

## Acknowledgments

- [Yellowstone gRPC](https://github.com/rpcpool/yellowstone-grpc) for Solana data streaming
- [ScyllaDB](https://www.scylladb.com/) for high-performance database storage
- [Redis](https://redis.io/) for reliable message queuing
- The Solana community for blockchain infrastructure

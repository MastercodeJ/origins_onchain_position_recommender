# Origins Onchain Position Recommender

A Rust-based application for analyzing and recommending positions in the Origins protocol ecosystem.

## Features

- **Position Analysis**: Analyze onchain positions with risk and liquidity scoring
- **Recommendation Engine**: Generate intelligent position recommendations
- **Market Data Integration**: Fetch and process market data for analysis
- **Configurable**: Flexible configuration through TOML files
- **CLI Interface**: Command-line interface with verbose logging

## Project Structure

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library root
└── lib/                 # Core library modules
    ├── lib.rs           # Library module exports
    ├── config.rs        # Configuration management
    ├── position.rs      # Position data structures and logic
    ├── recommender.rs   # Recommendation engine
    └── utils.rs         # Utility functions
```

## Dependencies

- **ethers**: Ethereum interaction and smart contract calls
- **tokio**: Async runtime
- **serde**: Serialization/deserialization
- **clap**: Command-line argument parsing
- **tracing**: Structured logging
- **rust_decimal**: High-precision decimal arithmetic
- **chrono**: Date and time handling

## Configuration

The application uses a TOML configuration file (`config.toml`) with the following options:

- `rpc_url`: Ethereum RPC endpoint
- `origins_contract_address`: Origins protocol contract address
- `position_threshold`: Minimum position value to consider
- `recommendation_interval`: Time between recommendation cycles
- `max_positions`: Maximum number of positions to recommend

## Usage

### Basic Usage

```bash
# Run with default configuration
cargo run

# Run with custom config file
cargo run -- --config my_config.toml

# Run with verbose logging
cargo run -- --verbose
```

### Building

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## Development

### Adding New Features

1. Add new modules to `src/lib/`
2. Export them in `src/lib/lib.rs`
3. Update imports in `src/main.rs` if needed

### Code Organization

- **config.rs**: Configuration loading and validation
- **position.rs**: Position data structures, market data, and analysis
- **recommender.rs**: Core recommendation logic and algorithms
- **utils.rs**: Utility functions for calculations and formatting

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## Roadmap

- [ ] Real blockchain integration
- [ ] Advanced risk metrics
- [ ] Machine learning recommendations
- [ ] Web dashboard
- [ ] API endpoints
- [ ] Historical data analysis


# yahboom_gps

[![github]](https://github.com/YenHarvey/yahboom_gps)&ensp;[![crates-io]](https://crates.io/crates/yahboom_gps)&ensp;[![docs-rs]](https://docs.rs/yahboom_gps)

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

<br>

`yahboom_gps` is a Rust library for initializing and reading GPS data from a Yahboom GPS module via a serial port.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
yahboom_gps = "0.1.0"
```

## Example

Here is a complete example demonstrating how to use the `yahboom_gps` library to initialize the GPS module, read GPS messages, and parse GPS data.

```rust
use yahboom_gps::{gps_init, read_complete_gps_message, parse_gps_data};
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize the GPS module
    let mut port = gps_init("COM3", 9600)?;

    // Continuously read and parse GPS messages
    while let Ok(Some(message)) = read_complete_gps_message(&mut port) {
        // Parse the GPS data
        let parsed_data = parse_gps_data(&message);

        // Print the parsed GPS data
        println!("Parsed GPS Data: {}", serde_json::to_string_pretty(&parsed_data)?);
        println!("--- End of message ---");
    }

    Ok(())
}
```

## Documentation

For more detailed documentation, please visit [docs.rs](https://docs.rs/yahboom_gps).

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for more details.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

## Contact

If you have any questions or suggestions, please feel free to contact me at [401].

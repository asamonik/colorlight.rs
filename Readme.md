# colorlight.rs

**A Rust library for detecting and sending frames to Colorlight 5A-75 LED receiver cards via Ethernet.**

## Features

- Detect receiver card
- Send display frames (brightness, color temperature)
- Send row-based pixel data frames (BGR pixel data)

## Requirements

- Rust (stable or later)
- Elevated privileges (CAP_NET_RAW or equivalent)
- Network interface connected to the Colorlight 5A-75 card

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
colorlight_rs = { git = "https://github.com/asamonik/colorlight.rs.git", branch = "main" }
```

## Example Usage

```rust
use colorlight_rs::ColorlightCard;

fn main() -> std::io::Result<()> {
    let interface_name = "eth0";
    let mut controller = ColorlightCard::open(interface_name)?;

    let info = controller.detect_receiver()?;
    println!("Receiver Info: {:?}", info);

    controller.send_display_frame(0xFF, 0xFF, 0x76, 0x06)?;

    let row_data_bgr: Vec<u8> = vec![0u8; 256 * 3];
    controller.send_row(0, &row_data_bgr)?;

    Ok(())
}
```

## Acknowledgements

Many thanks to hkubota for his detailed [blog post](https://hkubota.wordpress.com/2022/01/31/winter-project-colorlight-5a-75b-protocol/) on the Colorlight 5A-75B protocol.

## License

[MIT](./LICENSE)
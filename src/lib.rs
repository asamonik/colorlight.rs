//! colorlight_rs
//!
//! A library for detecting and sending frames to the
//! Colorlight 5A-75 LED receiver cards via Ethernet
//! frames.
//!
//! # Features
//! - Detect receiver card (send detect frame, parse response).
//! - Send display frames (brightness, color temperature).
//! - Send row-based pixel data frames (BGR pixel data).

use pnet::datalink::{self, Config};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::EthernetPacket;
use std::io::{Error, ErrorKind};

/// DST_MAC relevant for broadcast frame
pub const SRC_MAC: [u8; 6] = [0x22, 0x22, 0x33, 0x44, 0x55, 0x66];
pub const DST_MAC: [u8; 6] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

#[allow(dead_code)]
mod eth_types {
    pub const DETECT_RECEIVER_REQ: u16      = 0x0700;
    pub const DETECT_RECEIVER_RSP: u16      = 0x0805;
    pub const DETECT_RECEIVER_RSP_ACK: u16  = 0x0700;
    pub const DISPLAY_FRAME: u16            = 0x0107;
    pub const BRIGHTNESS_BASE: u16          = 0x0A00;
    pub const PIXEL_ROW_BASE: u16           = 0x5500; 
}

#[derive(Debug)]
pub struct ReceiverCardInfo {
    pub version_major: u8,
    pub version_minor: u8,
    pub pixel_columns: u16,
    pub pixel_rows: u16,
}

pub struct ColorlightCard {
    tx: Box<dyn pnet::datalink::DataLinkSender>,
    rx: Box<dyn pnet::datalink::DataLinkReceiver>,
}

impl ColorlightCard {
    /// opens a raw socket on the given network interface, needs CAP_NET_RAW capability on Linux for example
    pub fn open(interface_name: &str) -> Result<Self, Error> {
        let interfaces = datalink::interfaces();
        let interface = interfaces
            .into_iter()
            .find(|iface| iface.name == interface_name)
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("No interface named {}", interface_name),
                )
            })?;

        let mut cfg = Config::default();
        cfg.read_buffer_size = 4096; 
        cfg.write_buffer_size = 4096;

        let (tx, rx) = match datalink::channel(&interface, cfg)? {
            Ethernet(tx, rx) => (tx, rx),
            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Unsupported channel type (only Ethernet is supported)",
                ))
            }
        };

        Ok(Self {
            tx,
            rx,
        })
    }

    /// Sends the “Detect Receiver Card” frame and waits (optionally) for the broadcast
    /// response (0x0805). Returns parsed `ReceiverCardInfo` if successful.
    pub fn detect_receiver(&mut self) -> Result<ReceiverCardInfo, Error> {
        // send detect request
        let detect_req = build_detect_receiver_req();
        self.send_ethernet_frame(&detect_req)?;

        // listen for a broadcast response 0x0805
        let mut info: Option<ReceiverCardInfo> = None;
        let max_attempts = 100;
        for _ in 0..max_attempts {
            if let Ok(packet) = self.rx.next() {
                if packet.len() >= 14 {
                    let eth_pkt = EthernetPacket::new(packet).unwrap();
                    if eth_pkt.get_ethertype().0 == eth_types::DETECT_RECEIVER_RSP {
                        // Parse the response frame
                        let data = &packet[14..]; // skip Ethernet header
                        info = Some(parse_detect_receiver_response(data));
                        break;
                    }
                }
            }
        }

        let card_info = info.ok_or_else(|| {
            Error::new(
                ErrorKind::TimedOut,
                "No broadcast response (eth.type=0x0805) from receiver card",
            )
        })?;

        // ack with Data[2] = 1
        let ack_req = build_detect_receiver_ack();
        self.send_ethernet_frame(&ack_req)?;

        Ok(card_info)
    }

    /// Sends a “Display Frame” (EtherType = 0x0107).
    /// This can also set brightness and color temperature if needed.
    ///
    /// * `brightness` is 0..=0xFF (like 0xff for 100%, 0x40 for ~25%, etc.)
    /// * `(r, g, b)` can adjust color temperature or global color scaling.
    pub fn send_display_frame(
        &mut self,
        brightness: u8,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<(), Error> {
        let frame = build_display_frame(brightness, r, g, b);
        self.send_ethernet_frame(&frame)?;
        Ok(())
    }

    /// Sends a row of pixel data (EtherType = 0x5500 or 0x5501). 
    /// The data is assumed BGR format. 
    /// 
    /// The row index can exceed 255, so the top bit sets whether we use 0x5500 or 0x5501.
    /// Each row frame has a length: 7 bytes of header + row_len * 3 (assuming BGR). 
    /// In many panels, row_len might be 128 or 256 pixels wide.
    pub fn send_row(&mut self, row_number: u16, row_data_bgr: &[u8]) -> Result<(), Error> {
        let frame = build_pixel_row_frame(row_number, row_data_bgr);
        self.send_ethernet_frame(&frame)?;
        Ok(())
    }

    fn send_ethernet_frame(&mut self, payload: &[u8]) -> Result<(), Error> {
        self.tx.send_to(payload, None).ok_or_else(|| {
            Error::new(
                ErrorKind::Other,
                "Failed to send raw Ethernet frame using pnet DataLinkSender",
            )
        })??;
        Ok(())
    }
}

/// Helper function: Build “Detect Receiver Card” request
fn build_detect_receiver_req() -> Vec<u8> {
    let total_len = 14 + 270;
    let mut frame = vec![0u8; total_len];

    frame[0..6].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
    frame[6..12].copy_from_slice(&[0x22, 0x22, 0x33, 0x44, 0x55, 0x66]);

    frame[12] = 0x07;
    frame[13] = 0x00;

    frame
}

/// Helper function: Build detect receiver ack (eth.type=0x0700, Data[2] = 1)
fn build_detect_receiver_ack() -> Vec<u8> {
    let total_len = 14 + 270; 
    let mut frame = vec![0u8; total_len];

    frame[0..6].copy_from_slice(&DST_MAC); 
    frame[6..12].copy_from_slice(&SRC_MAC); 
    frame[12] = 0x07;
    frame[13] = 0x00;

    frame[14 + 2] = 1;

    frame
}

/// Parse the 0x0805 “Detect Receiver Response Frame” data into a ReceiverCardInfo
fn parse_detect_receiver_response(data: &[u8]) -> ReceiverCardInfo {
    // Data[0] = 0x5A (Receiver card version "5A")
    // Data[1] = version major
    // Data[2] = version minor
    // Data[20..22] = pixel columns (HSB + LSB)
    // Data[22..24] = pixel rows    (HSB + LSB)
    if data.len() < 24 {
        // fallback
        return ReceiverCardInfo {
            version_major: 0,
            version_minor: 0,
            pixel_columns: 0,
            pixel_rows: 0,
        };
    }
    let version_major = data[1];
    let version_minor = data[2];
    let cols = ((data[20] as u16) << 8) | data[21] as u16;
    let rows = ((data[22] as u16) << 8) | data[23] as u16;

    ReceiverCardInfo {
        version_major,
        version_minor,
        pixel_columns: cols,
        pixel_rows: rows,
    }
}

/// Build a “display frame” (EtherType = 0x0107, data length = 98).
/// [21] brightness, [22] = 5, [24..27] brightness for R, G, B.
fn build_display_frame(brightness: u8, r: u8, g: u8, b: u8) -> Vec<u8> {
    let total_len = 14 + 98;
    let mut frame = vec![0u8; total_len];

    frame[0..6].copy_from_slice(&DST_MAC);
    frame[6..12].copy_from_slice(&SRC_MAC);
    frame[12] = 0x01;
    frame[13] = 0x07;

    frame[14 + 21] = brightness;
    frame[14 + 22] = 5;
    frame[14 + 24] = r;
    frame[14 + 25] = g;
    frame[14 + 26] = b;

    frame
}

/// Build a single “pixel row” frame (EtherType = 0x5500 or 0x5501).
/// Data length is 7 + (3 * pixel_count). Format: 
/// [0] row LSB
/// [1] MSB of pixel offset
/// [2] LSB of pixel offset
/// [3] MSB of pixel count
/// [4] LSB of pixel count
/// [5] 0x08
/// [6] 0x80 or 0x88
/// [7..] = the BGR pixel data
fn build_pixel_row_frame(row_number: u16, row_data_bgr: &[u8]) -> Vec<u8> {
    let pixel_count = (row_data_bgr.len() / 3) as u16; 
    let header_len = 7;
    let data_len = header_len + row_data_bgr.len();
    let total_len = 14 + data_len;

    let ethertype = if row_number < 256 {
        0x5500
    } else {
        0x5501
    };

    let mut frame = vec![0u8; total_len];

    frame[0..6].copy_from_slice(&DST_MAC);
    frame[6..12].copy_from_slice(&SRC_MAC);
    frame[12] = (ethertype >> 8) as u8;
    frame[13] = (ethertype & 0xff) as u8;

    let data_offset = 14;
    frame[data_offset + 0] = (row_number & 0xff) as u8;
    frame[data_offset + 1] = 0x00;
    frame[data_offset + 2] = 0x00;
    frame[data_offset + 3] = ((pixel_count >> 8) & 0xff) as u8;
    frame[data_offset + 4] = (pixel_count & 0xff) as u8;
    frame[data_offset + 5] = 0x08;
    frame[data_offset + 6] = 0x88; 

    frame[(data_offset + header_len)..(data_offset + header_len + row_data_bgr.len())]
        .copy_from_slice(&row_data_bgr);

    frame
}

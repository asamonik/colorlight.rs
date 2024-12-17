#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_build_detect_receiver_req() {
        let frame = build_detect_receiver_req();
        assert_eq!(frame.len(), 284);
        assert_eq!(&frame[0..6], &DST_MAC);
        assert_eq!(&frame[6..12], &SRC_MAC);
        assert_eq!(frame[12], 0x07);
        assert_eq!(frame[13], 0x00);
    }

    #[test]
    fn test_build_detect_receiver_ack() {
        let frame = build_detect_receiver_ack();
        assert_eq!(frame.len(), 284);
        assert_eq!(&frame[0..6], &DST_MAC);
        assert_eq!(&frame[6..12], &SRC_MAC);
        assert_eq!(frame[12], 0x07);
        assert_eq!(frame[13], 0x00);
        assert_eq!(frame[16], 1);
    }

    #[test]
    fn test_parse_detect_receiver_response() {
        let data = vec![
            0x5A, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x02, 0x00,
        ];
        let info = parse_detect_receiver_response(&data);
        assert_eq!(info.version_major, 1);
        assert_eq!(info.version_minor, 2);
        assert_eq!(info.pixel_columns, 256);
        assert_eq!(info.pixel_rows, 512);
    }

    #[test]
    fn test_build_display_frame() {
        let frame = build_display_frame(0xFF, 0xFF, 0x76, 0x06);
        assert_eq!(frame.len(), 112);
        assert_eq!(&frame[0..6], &DST_MAC);
        assert_eq!(&frame[6..12], &SRC_MAC);
        assert_eq!(frame[12], 0x01);
        assert_eq!(frame[13], 0x07);
        assert_eq!(frame[35], 0xFF);
        assert_eq!(frame[36], 5);
        assert_eq!(frame[38], 0xFF);
        assert_eq!(frame[39], 0x76);
        assert_eq!(frame[40], 0x06);
    }
}
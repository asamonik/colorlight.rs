use colorlight::ColorlightCard;
use std::thread;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let interface_name = "en0";
    let mut controller = ColorlightCard::open(interface_name)?;

    // Detect the receiver card
    let info = controller.detect_receiver()?;
    println!("Receiver Info: {:?}", info);

    // Generate a image for the whole screen
    let pixel_columns = info.pixel_columns as usize;
    let pixel_rows = info.pixel_rows as usize;
    let mut row_data_bgr: Vec<u8> = vec![0u8; pixel_columns * 3]; // BGR format

    // Set the pixles
    for pixel in row_data_bgr.chunks_mut(3) {
        pixel[0] = 0x00;
        pixel[1] = 0x00;
        pixel[2] = 0xFF;
    }

    loop {
        // send the data
        for row in 0..pixel_rows {
            controller.send_row(row as u16, &row_data_bgr)?;
        }

        // display the frame
        controller.send_display_frame(0xFF, 0xFF, 0xFF, 0xFF)?;

        // sleep to avoid flickering
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}
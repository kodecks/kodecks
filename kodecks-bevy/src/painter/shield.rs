use image::{DynamicImage, GenericImage, GenericImageView, ImageReader};
use std::{io::Cursor, sync::LazyLock};

pub fn draw_shield(image: &mut DynamicImage, base_x: u32, base_y: u32, shield: u8) {
    static SHIELD: LazyLock<DynamicImage> = LazyLock::new(|| {
        ImageReader::new(Cursor::new(include_bytes!("shield.png")))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap()
    });
    let shield = shield as i32;
    for i in 0..shield {
        for (x, y, pixel) in SHIELD.as_rgba8().unwrap().enumerate_pixels() {
            if SHIELD.get_pixel(x, y)[3] != 0 {
                let x = (x as i32 + base_x as i32 - SHIELD.width() as i32 + ((i + 1 - shield) * 3))
                    as u32;
                let y = y + base_y - SHIELD.height();
                image.put_pixel(x, y, *pixel);
            }
        }
    }
}

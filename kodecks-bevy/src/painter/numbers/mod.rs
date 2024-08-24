use bevy::{ecs::system::Resource, utils::HashMap};
use image::{DynamicImage, GenericImage, GenericImageView, ImageReader, Rgba};
use std::{io::Cursor, sync::LazyLock};

#[derive(Default, Resource)]
pub struct NumberPainter {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Alignment {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawOptions {
    pub x: u32,
    pub y: u32,
    pub h_align: Alignment,
    pub v_align: Alignment,
    pub foreground: Rgba<u8>,
    pub background: Rgba<u8>,
}

impl Default for DrawOptions {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            h_align: Alignment::Start,
            v_align: Alignment::Start,
            foreground: Rgba([0, 0, 0, 255]),
            background: Rgba([255, 255, 255, 255]),
        }
    }
}

impl NumberPainter {
    pub fn draw(&self, s: &str, options: &DrawOptions, image: &mut DynamicImage) {
        let mut x = options.x;
        let mut y = options.y;
        if options.h_align != Alignment::Start {
            let width = s
                .chars()
                .map(|c| self.get_number(c).width() - 1)
                .sum::<u32>();
            x = x.saturating_sub(if options.h_align == Alignment::End {
                width
            } else {
                width / 2
            });
        }
        if options.v_align != Alignment::Start {
            let height = s
                .chars()
                .map(|c| self.get_number(c).height())
                .max()
                .unwrap_or(0);
            y = y.saturating_sub(if options.v_align == Alignment::End {
                height
            } else {
                height / 2
            });
        }
        for c in s.chars() {
            let number = self.get_number(c);
            for (px, py, color) in number.pixels() {
                let color = if color[3] > 0 {
                    options.foreground
                } else {
                    let outline = (-1..=1)
                        .flat_map(|dx| (-1..=1).map(move |dy| (dx, dy)))
                        .filter(|&(dx, dy)| dx != 0 || dy != 0)
                        .map(|(dx, dy)| (px as i32 + dx, py as i32 + dy))
                        .filter(|&(px, py)| {
                            px >= 0
                                && px < number.width() as i32
                                && py >= 0
                                && py < number.height() as i32
                        })
                        .map(|(px, py)| number.get_pixel(px as u32, py as u32))
                        .any(|n| n[3] > 0);
                    if outline {
                        options.background
                    } else {
                        continue;
                    }
                };
                image.put_pixel(x + px, y + py, color);
            }
            x += number.width() - 1;
        }
    }

    fn get_number(&self, n: char) -> &DynamicImage {
        static NUMBERS: LazyLock<HashMap<char, DynamicImage>> = LazyLock::new(|| {
            NUMBER_IMAGES
                .iter()
                .map(|&(c, data)| {
                    (
                        c,
                        ImageReader::new(Cursor::new(data))
                            .with_guessed_format()
                            .unwrap()
                            .decode()
                            .unwrap(),
                    )
                })
                .collect()
        });

        NUMBERS.get(&n).or_else(|| NUMBERS.get(&'o')).unwrap()
    }
}

const NUMBER_IMAGES: &[(char, &[u8])] = &[
    ('0', include_bytes!("num_0.png")),
    ('1', include_bytes!("num_1.png")),
    ('2', include_bytes!("num_2.png")),
    ('3', include_bytes!("num_3.png")),
    ('4', include_bytes!("num_4.png")),
    ('5', include_bytes!("num_5.png")),
    ('6', include_bytes!("num_6.png")),
    ('7', include_bytes!("num_7.png")),
    ('8', include_bytes!("num_8.png")),
    ('9', include_bytes!("num_9.png")),
    ('k', include_bytes!("num_k.png")),
    ('m', include_bytes!("num_m.png")),
    ('-', include_bytes!("num_minus.png")),
    ('+', include_bytes!("num_plus.png")),
    ('o', include_bytes!("num_s0.png")),
];

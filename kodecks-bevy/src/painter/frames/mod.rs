use super::{
    numbers::{Alignment, DrawOptions, NumberPainter},
    shield::draw_shield,
};
use bevy::{ecs::system::Resource, utils::HashMap};
use dashmap::DashMap;
use image::{DynamicImage, GenericImageView, ImageReader, Rgba};
use kodecks::{color::Color, computed::ComputedAttribute};
use std::{io::Cursor, sync::LazyLock};

#[derive(Default, Resource)]
pub struct CardFramePainter {
    frames: DashMap<CardFrame, DynamicImage>,
    number: NumberPainter,
}

impl CardFramePainter {
    pub fn generate_frame(&self, frame: CardFrame) -> DynamicImage {
        self.frames
            .entry(frame)
            .or_insert_with(|| self.generate(&frame))
            .clone()
    }

    pub fn get_color(&self, color: Color) -> Rgba<u8> {
        Self::get_frame(color).get_pixel(0, 3)
    }

    fn generate(&self, frame: &CardFrame) -> DynamicImage {
        let mut frame_base = Self::get_frame(frame.color).clone();
        let background = self.get_color(Color::empty());
        if let Some(power) = frame.power {
            self.number.draw(
                &format!("{power}").replace('0', "o"),
                &DrawOptions {
                    x: 1,
                    y: 47,
                    foreground: [255, 255, 255, 255].into(),
                    background,
                    h_align: Alignment::Start,
                    v_align: Alignment::End,
                },
                &mut frame_base,
            );
        }
        if let Some(shields) = frame.shields {
            draw_shield(&mut frame_base, 35, 47, shields);
        }
        frame_base
    }

    fn get_frame(color: Color) -> &'static DynamicImage {
        static FRAMES: LazyLock<HashMap<Color, DynamicImage>> = LazyLock::new(|| {
            FRAME_IMAGES
                .iter()
                .map(|(color, data)| {
                    let image = ImageReader::new(Cursor::new(data))
                        .with_guessed_format()
                        .unwrap()
                        .decode()
                        .unwrap();
                    (*color, image)
                })
                .collect()
        });
        FRAMES.get(&color).unwrap()
    }
}

const FRAME_IMAGES: &[(Color, &[u8])] = &[
    (Color::RED, include_bytes!("frame_red.png")),
    (Color::YELLOW, include_bytes!("frame_yellow.png")),
    (Color::GREEN, include_bytes!("frame_green.png")),
    (Color::BLUE, include_bytes!("frame_blue.png")),
    (Color::empty(), include_bytes!("frame_colorless.png")),
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CardFrame {
    pub color: Color,
    pub cost: u8,
    pub power: Option<u32>,
    pub shields: Option<u8>,
}

impl CardFrame {
    pub fn new(attr: &ComputedAttribute) -> Self {
        Self {
            color: attr.color,
            cost: attr.cost.value(),
            power: attr.power.map(|p| p.value()),
            shields: attr.shields.map(|p| p.value()),
        }
    }
}

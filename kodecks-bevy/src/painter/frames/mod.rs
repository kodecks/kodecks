use super::numbers::{Alignment, DrawOptions, NumberPainter};
use bevy::{ecs::system::Resource, utils::HashMap};
use dashmap::DashMap;
use image::{DynamicImage, GenericImage, GenericImageView, ImageReader, Rgba};
use kodecks::{archetype::CreatureType, color::Color, computed::ComputedAttribute};
use std::{io::Cursor, sync::LazyLock};

#[derive(Default, Resource)]
pub struct CardFramePainter {
    frames: DashMap<CardFrame, DynamicImage>,
    deck_frames: DashMap<CardFrame, DynamicImage>,
    number: NumberPainter,
}

impl CardFramePainter {
    pub fn generate_frame(&self, frame: CardFrame) -> DynamicImage {
        self.frames
            .entry(frame)
            .or_insert_with(|| self.generate(&frame))
            .clone()
    }

    pub fn generate_deck_frame(&self, frame: CardFrame) -> DynamicImage {
        self.deck_frames
            .entry(frame)
            .or_insert_with(|| self.generate_deck(&frame))
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
                    y: 54,
                    foreground: [255, 255, 255, 255].into(),
                    background,
                    h_align: Alignment::Start,
                    v_align: Alignment::End,
                },
                &mut frame_base,
            );
        }
        if let Some(creature_type) = frame.creature_type {
            let image = Self::get_creature_type(creature_type);
            for (x, y, pixel) in image.as_rgba8().unwrap().enumerate_pixels() {
                if pixel[3] != 0 {
                    frame_base.put_pixel(x + 30, y + 2, *pixel);
                }
            }
        }

        for i in 0..frame.shards {
            let image = Self::get_shard(frame.color);
            let shards_x = 33 - (i as u32 % 3) * 5;
            let shards_y = 48 - (i as u32 / 3) * 5;
            for (x, y, pixel) in image.as_rgba8().unwrap().enumerate_pixels() {
                if pixel[3] != 0 {
                    frame_base.put_pixel(shards_x + x, shards_y + y, *pixel);
                }
            }
        }

        frame_base
    }

    fn generate_deck(&self, frame: &CardFrame) -> DynamicImage {
        let mut frame_base = Self::get_deck_frame(frame.color).clone();
        self.number.draw(
            &frame.cost.to_string(),
            &DrawOptions {
                x: 14,
                y: 4,
                h_align: Alignment::Start,
                v_align: Alignment::Start,
                background: [0, 0, 0, 255].into(),
                foreground: [255, 255, 255, 255].into(),
            },
            &mut frame_base,
        );
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

    fn get_deck_frame(color: Color) -> &'static DynamicImage {
        static FRAMES: LazyLock<HashMap<Color, DynamicImage>> = LazyLock::new(|| {
            DECK_FRAME_IMAGES
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

    fn get_shard(color: Color) -> &'static DynamicImage {
        static SHARD_IMAGES: LazyLock<HashMap<Color, DynamicImage>> = LazyLock::new(|| {
            SHARD_TYPES
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
        SHARD_IMAGES.get(&color).unwrap()
    }

    fn get_creature_type(creature_type: CreatureType) -> &'static DynamicImage {
        static CREATURE_TYPE_IMAGES: LazyLock<HashMap<CreatureType, DynamicImage>> =
            LazyLock::new(|| {
                CREATURE_TYPES
                    .iter()
                    .map(|(t, data)| {
                        let image = ImageReader::new(Cursor::new(data))
                            .with_guessed_format()
                            .unwrap()
                            .decode()
                            .unwrap();
                        (*t, image)
                    })
                    .collect()
            });
        CREATURE_TYPE_IMAGES.get(&creature_type).unwrap()
    }
}

const FRAME_IMAGES: &[(Color, &[u8])] = &[
    (Color::RED, include_bytes!("frame_red.png")),
    (Color::YELLOW, include_bytes!("frame_yellow.png")),
    (Color::GREEN, include_bytes!("frame_green.png")),
    (Color::BLUE, include_bytes!("frame_blue.png")),
    (Color::empty(), include_bytes!("frame_colorless.png")),
];

const DECK_FRAME_IMAGES: &[(Color, &[u8])] = &[
    (Color::RED, include_bytes!("deck_frame_red.png")),
    (Color::YELLOW, include_bytes!("deck_frame_yellow.png")),
    (Color::GREEN, include_bytes!("deck_frame_green.png")),
    (Color::BLUE, include_bytes!("deck_frame_blue.png")),
    (Color::empty(), include_bytes!("deck_frame_colorless.png")),
];

const SHARD_TYPES: &[(Color, &[u8])] = &[
    (Color::RED, include_bytes!("shard_red.png")),
    (Color::YELLOW, include_bytes!("shard_yellow.png")),
    (Color::GREEN, include_bytes!("shard_green.png")),
    (Color::BLUE, include_bytes!("shard_blue.png")),
    (Color::empty(), include_bytes!("shard_colorless.png")),
];

const CREATURE_TYPES: &[(CreatureType, &[u8])] = &[
    (CreatureType::Mutant, include_bytes!("mutant.png")),
    (CreatureType::Cyborg, include_bytes!("cyborg.png")),
    (CreatureType::Robot, include_bytes!("robot.png")),
    (CreatureType::Ghost, include_bytes!("ghost.png")),
    (CreatureType::Program, include_bytes!("program.png")),
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CardFrame {
    pub color: Color,
    pub cost: u8,
    pub power: Option<u32>,
    pub shards: u8,
    pub creature_type: Option<CreatureType>,
}

impl CardFrame {
    pub fn new(attr: &ComputedAttribute) -> Self {
        Self {
            color: attr.color,
            cost: attr.cost.value(),
            power: attr.power.map(|p| p.value()),
            shards: attr.shards.value(),
            creature_type: attr.creature_type,
        }
    }
}

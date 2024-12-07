use crate::painter::frames::{CardFrame, CardFramePainter};
use crate::painter::numbers::{Alignment, DrawOptions, NumberPainter};
use crate::scene::card::Catalog;
use arc_swap::ArcSwap;
use bevy::asset::AsyncReadExt;
use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::utils::ConditionalSendFuture;
use image::codecs::png::PngDecoder;
use image::{DynamicImage, GenericImage, GenericImageView};
use kodecks::computed::ComputedAttribute;
use std::io::Cursor;
use std::sync::LazyLock;
use thiserror;
use thiserror::Error;

pub struct RenderedCardPlugin;

impl Plugin for RenderedCardPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<RenderedCardLoader>();
    }
}

#[derive(Asset, TypePath)]
pub struct RenderedCard {
    #[dependency]
    pub image: Handle<Image>,
}

struct RenderedCardLoader {
    catalog: ArcSwap<kodecks::catalog::Catalog>,
    painter: CardFramePainter,
    number: NumberPainter,
}

impl FromWorld for RenderedCardLoader {
    fn from_world(world: &mut World) -> Self {
        let catalog = ArcSwap::new((**world.get_resource::<Catalog>().unwrap()).clone());
        Self {
            catalog,
            painter: CardFramePainter::default(),
            number: NumberPainter::default(),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum RenderedCardLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
}

const STACK_FRAME: &[u8] = include_bytes!("../painter/frames/stack_frame.png");

impl AssetLoader for RenderedCardLoader {
    type Asset = Image;
    type Settings = ();
    type Error = RenderedCardLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture<
        Output = Result<<Self as AssetLoader>::Asset, <Self as AssetLoader>::Error>,
    > {
        Box::pin(async move {
            static STACK_FRAME_IMAGE: LazyLock<DynamicImage> = LazyLock::new(|| {
                let decoder = PngDecoder::new(Cursor::new(STACK_FRAME)).unwrap();
                DynamicImage::from_decoder(decoder).unwrap()
            });

            let mut contents = Vec::new();
            reader.read_to_end(&mut contents).await?;
            let decoder = PngDecoder::new(Cursor::new(contents)).unwrap();
            let image = DynamicImage::from_decoder(decoder).unwrap().into_rgba8();

            load_context.labeled_asset_scope("image".to_string(), |_| {
                Image::new(
                    Extent3d {
                        width: image.width(),
                        height: image.height(),
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    image.to_vec(),
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::RENDER_WORLD,
                )
            });

            let id = load_context
                .asset_path()
                .path()
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            let archetype = self.catalog.load()[id].clone();
            let attr = ComputedAttribute::from(&*archetype);
            let mut frame_image = self.painter.generate_frame(CardFrame::new(&attr));
            for (x, y, pixel) in image.enumerate_pixels() {
                if frame_image.get_pixel(x + 4, y + 14)[3] == 0 {
                    frame_image.put_pixel(x + 4, y + 14, *pixel);
                }
            }

            load_context.labeled_asset_scope("deck".to_string(), |_| {
                let mut frame_image = self.painter.generate_deck_frame(CardFrame::new(&attr));
                for (x, y, pixel) in image.enumerate_pixels() {
                    if y >= 16 {
                        break;
                    }
                    if frame_image.get_pixel(x + 11, y + 2)[3] == 0 {
                        frame_image.put_pixel(x + 11, y + 2, *pixel);
                    }
                }
                Image::new(
                    Extent3d {
                        width: frame_image.width(),
                        height: frame_image.height(),
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    frame_image.clone().into_bytes(),
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::RENDER_WORLD,
                )
            });

            load_context.labeled_asset_scope("stack".to_string(), |_| {
                let mut stack_image = STACK_FRAME_IMAGE.clone();
                for (x, y, pixel) in image.enumerate_pixels() {
                    if y >= 16 {
                        break;
                    }
                    if stack_image.get_pixel(x + 14, y + 2)[3] == 0 {
                        stack_image.put_pixel(x + 14, y + 2, *pixel);
                    }
                }

                Image::new(
                    Extent3d {
                        width: stack_image.width(),
                        height: stack_image.height(),
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    stack_image.into_bytes(),
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::RENDER_WORLD,
                )
            });

            let background = self.painter.get_color(Default::default());
            if !archetype.attribute.is_token {
                self.number.draw(
                    &archetype.attribute.cost.to_string(),
                    &DrawOptions {
                        x: 2,
                        y: 2,
                        h_align: Alignment::Start,
                        v_align: Alignment::Start,
                        background,
                        foreground: [255, 255, 255, 255].into(),
                    },
                    &mut frame_image,
                );
            }

            let image = Image::new(
                Extent3d {
                    width: frame_image.width(),
                    height: frame_image.height(),
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                frame_image.into_bytes(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            );

            if std::env::var("KODECKS_SLOW_ASSETS").is_ok() {
                let (send, recv) = async_channel::unbounded::<()>();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    let _ = send.try_send(());
                });
                let _ = recv.recv().await;
            }

            Ok(image)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["main.png"]
    }
}

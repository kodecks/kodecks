use crate::scene::translator::TextPurpose;
use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::asset::{AsyncReadExt, ReadAssetBytesError};
use bevy::prelude::*;
use bevy::utils::{ConditionalSendFuture, HashMap};
use fluent_bundle::FluentResource;
use fluent_syntax::parser::ParserError;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use thiserror;
use thiserror::Error;
use unic_langid::{langid, LanguageIdentifier, LanguageIdentifierError};

pub const DEFAULT_LANG: LanguageIdentifier = langid!("en-US");

pub struct FluentPlugin;

impl Plugin for FluentPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<FluentAsset>()
            .init_asset_loader::<FluentAssetLoader>();
    }
}

#[derive(Asset, TypePath)]
pub struct FluentAsset {
    pub resources: Vec<(LanguageIdentifier, Vec<Arc<FluentResource>>)>,
    pub styles: HashMap<TextPurpose, TextStyle>,
}

#[derive(Default)]
struct FluentAssetLoader {}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum FluentAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] ReadAssetBytesError),

    #[error("Could not parse asset: {0:?}")]
    ParseError(Vec<ParserError>),

    #[error("Could not parse langid: {0}")]
    LanguageIdError(#[from] LanguageIdentifierError),
}

#[derive(Deserialize)]
struct LangConfig {
    fonts: HashMap<TextPurpose, FontConfig>,
}

#[derive(Deserialize)]
struct FontConfig {
    font: String,
    size: f32,
}

impl AssetLoader for FluentAssetLoader {
    type Asset = FluentAsset;
    type Settings = ();
    type Error = FluentAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture<
        Output = Result<<Self as AssetLoader>::Asset, <Self as AssetLoader>::Error>,
    > {
        Box::pin(async move {
            let mut json_buf = Vec::new();
            reader.read_to_end(&mut json_buf).await.unwrap();
            let config = serde_json::from_slice::<LangConfig>(&json_buf).unwrap();

            let lang = load_context
                .path()
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            let langid = LanguageIdentifier::from_str(lang)
                .map_err(FluentAssetLoaderError::LanguageIdError)?;

            let mut resources = vec![(langid.clone(), load_assets(&langid, load_context).await)];
            if langid != DEFAULT_LANG {
                resources.push((DEFAULT_LANG, load_assets(&DEFAULT_LANG, load_context).await));
            }

            let fonts = config
                .fonts
                .into_iter()
                .map(|(purpose, font_config)| {
                    let font = load_context.load(&font_config.font);
                    (
                        purpose,
                        TextStyle {
                            font_size: font_config.size,
                            font,
                            ..Default::default()
                        },
                    )
                })
                .collect();

            Ok(FluentAsset {
                resources,
                styles: fonts,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["lang.json"]
    }
}

async fn load_assets<'a>(
    langid: &LanguageIdentifier,
    load_context: &'a mut LoadContext<'_>,
) -> Vec<Arc<FluentResource>> {
    let files = ["main.ftl", "cards.ftl"];
    let mut resources = Vec::new();
    for file in files {
        let res = load_path(langid, file, load_context).await;
        match res {
            Ok(res) => resources.push(res),
            Err(err) => error!("Failed to load Fluent resource: {:?}", err),
        }
    }
    resources
}

async fn load_path<'a>(
    langid: &LanguageIdentifier,
    file: &str,
    load_context: &'a mut LoadContext<'_>,
) -> Result<Arc<FluentResource>, FluentAssetLoaderError> {
    let asset_path = load_context
        .asset_path()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .resolve(&langid.to_string())
        .unwrap()
        .resolve(file)
        .unwrap();
    let data = load_context
        .read_asset_bytes(asset_path)
        .await
        .map_err(FluentAssetLoaderError::Io)?;
    let contents = String::from_utf8(data).unwrap();
    let res = FluentResource::try_new(contents)
        .map_err(|(_, errors)| FluentAssetLoaderError::ParseError(errors))?;
    Ok(Arc::new(res))
}

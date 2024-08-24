use crate::assets::fluent::{FluentAsset, DEFAULT_LANG};
use bevy::{prelude::*, utils::HashMap};
use fluent_bundle::{concurrent::FluentBundle, FluentArgs, FluentResource};
use fluent_content::{Content, Request};
use serde::Deserialize;
use std::{
    borrow::{Borrow, Cow},
    sync::Arc,
};

#[derive(Resource)]
pub struct Translator {
    bundles: Vec<FluentBundle<Arc<FluentResource>>>,
    styles: HashMap<TextPurpose, TextStyle>,
}

impl Translator {
    pub fn new(asset: &FluentAsset) -> Self {
        let bundles = asset
            .resources
            .iter()
            .map(|(langid, resources)| {
                let mut bundle = FluentBundle::new_concurrent(vec![langid.clone()]);
                for resource in resources {
                    bundle.add_resource(resource.clone()).unwrap();
                }
                bundle
            })
            .collect();
        Self {
            bundles,
            styles: asset.styles.clone(),
        }
    }

    pub fn style(&self, purpose: TextPurpose) -> TextStyle {
        self.styles.get(&purpose).cloned().unwrap_or_default()
    }

    pub fn get<'a, T, U>(&self, request: T) -> Cow<'a, str>
    where
        T: Copy + Into<Request<'a, U>>,
        U: Borrow<FluentArgs<'a>>,
    {
        let req: Request<'a, U> = request.into();

        self.content(request)
            .map(|s| Cow::Owned(s.replace(['\u{2068}', '\u{2069}'], "")))
            .unwrap_or(if req.attr.is_some() {
                Cow::Borrowed("")
            } else {
                Cow::Borrowed(req.id)
            })
    }

    pub fn get_default_lang<'a, T, U>(&self, request: T) -> Cow<'a, str>
    where
        T: Copy + Into<Request<'a, U>>,
        U: Borrow<FluentArgs<'a>>,
    {
        let req: Request<'a, U> = request.into();

        self.bundles
            .iter()
            .filter(|bundle| bundle.locales.contains(&DEFAULT_LANG))
            .find_map(|bundle| bundle.content(request))
            .map(|s| Cow::Owned(s.replace(['\u{2068}', '\u{2069}'], "")))
            .unwrap_or(if req.attr.is_some() {
                Cow::Borrowed("")
            } else {
                Cow::Borrowed(req.id)
            })
    }
}

impl<'a, T, U> Content<'a, T, U> for Translator
where
    T: Copy + Into<Request<'a, U>>,
    U: Borrow<FluentArgs<'a>>,
{
    fn content(&self, request: T) -> Option<String> {
        self.bundles
            .iter()
            .find_map(|bundle| bundle.content(request))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TextPurpose {
    Title,
    CardName,
    CardText,
    CardAbility,
    Dialog,
    Button,
    Result,
}

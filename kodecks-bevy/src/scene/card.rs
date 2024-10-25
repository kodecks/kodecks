use super::translator::Translator;
use crate::scene::translator::TextPurpose;
use bevy::{color::palettes::css, prelude::*};
use kodecks::{
    card::{CardSnapshot, Catalog},
    prelude::KeywordAbility,
    text::{parse_text, Section},
};
use kodecks_catalog::CATALOG;

#[derive(Debug)]
pub struct UICardInfo {
    pub snapshot: CardSnapshot,
}

impl UICardInfo {
    pub fn new(snapshot: CardSnapshot) -> Self {
        Self { snapshot }
    }

    pub fn related_abilities(&self, translator: &Translator) -> Vec<KeywordAbility> {
        let mut abilities = self
            .snapshot
            .computed
            .as_ref()
            .map(|attr| attr.abilities.as_ref())
            .unwrap_or_default()
            .to_vec();

        let safe_name = CATALOG[self.snapshot.archetype_id].safe_name;
        abilities.extend(translator.get_related_items(safe_name).abilities);
        abilities.sort();
        abilities.dedup();

        abilities
    }

    pub fn text_sections(&self, translator: &Translator, catalog: &Catalog) -> Vec<TextSection> {
        let safe_name = catalog[self.snapshot.archetype_id].safe_name;
        let id = format!("card-{safe_name}.text");
        let text = translator.get(&id);

        let abilities = self
            .snapshot
            .computed
            .as_ref()
            .map(|attr| attr.abilities.as_ref())
            .unwrap_or_default();

        let mut sections = abilities
            .iter()
            .flat_map(|ability| {
                let ability_name = format!("ability-{}", ability.to_string().to_lowercase());
                let ability_name = translator.get(&ability_name);
                vec![
                    TextSection::new(
                        ability_name,
                        TextStyle {
                            color: css::GOLD.into(),
                            ..translator.style(TextPurpose::CardText)
                        },
                    ),
                    TextSection::new("  ", translator.style(TextPurpose::CardText)),
                ]
            })
            .collect::<Vec<_>>();

        if !sections.is_empty() {
            sections.push(TextSection::new(
                "\n\n",
                TextStyle {
                    font_size: 10.0,
                    ..default()
                },
            ));
        }

        for section in parse_text(&text) {
            match section {
                Section::Text(text) => {
                    sections.push(TextSection::new(
                        text,
                        translator.style(TextPurpose::CardText),
                    ));
                }
                Section::Card(text) => {
                    sections.push(TextSection::new(
                        text,
                        TextStyle {
                            color: css::LIGHT_BLUE.into(),
                            ..translator.style(TextPurpose::CardText)
                        },
                    ));
                }
                Section::Keyword(ability) => {
                    sections.push(TextSection::new(
                        ability.to_string(),
                        TextStyle {
                            color: css::GOLD.into(),
                            ..translator.style(TextPurpose::CardText)
                        },
                    ));
                }
                Section::Number(n) => {
                    sections.push(TextSection::new(
                        n.to_string(),
                        TextStyle {
                            color: css::LIGHT_PINK.into(),
                            ..translator.style(TextPurpose::CardText)
                        },
                    ));
                }
            }
        }

        sections
    }
}

use unic_langid::{langid, LanguageIdentifier};

pub fn find_language(id: LanguageIdentifier) -> &'static Language {
    LANGUAGES
        .iter()
        .find(|lang| lang.id == id || lang.aliases.contains(&id))
        .or_else(|| {
            LANGUAGES.iter().find(|lang| {
                lang.id.matches(&id, true, true)
                    || lang
                        .aliases
                        .iter()
                        .any(|alias| alias.matches(&id, true, true))
            })
        })
        .unwrap_or(&LANGUAGES[0])
}

const LANGUAGES: &[Language] = &[
    Language {
        id: langid!("en-US"),
        name: "English",
        aliases: &[langid!("en")],
    },
    Language {
        id: langid!("ja-JP"),
        name: "Japanese",
        aliases: &[langid!("ja")],
    },
];

#[derive(Debug, Clone)]
pub struct Language {
    pub id: LanguageIdentifier,
    pub name: &'static str,
    pub aliases: &'static [LanguageIdentifier],
}

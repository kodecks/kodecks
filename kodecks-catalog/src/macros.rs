#[macro_export]
macro_rules! card_def {
    ( $struct:ident, $id:literal, $name:literal, $( $key:ident : $value:expr, )*) => {
        #[derive(Clone, Copy)]
        pub struct $struct;

        pub const ARCHETYPE: fn() -> &'static CardArchetype = || {
            static SAFE_NAME: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| kodecks::card::safe_name($name).unwrap());
            static CACHE: std::sync::LazyLock<CardArchetype> = std::sync::LazyLock::new(|| CardArchetype {
                id: tinystr::TinyAsciiStr::from_bytes_lossy($id.as_bytes()),
                name: $name,
                safe_name: &SAFE_NAME,
                attribute: CardAttribute {
                    $( $key : ($value).into(), )*
                    ..Default::default()
                },
                effect: || Box::new(CardDef),
            });
            &CACHE
        };
    };
}

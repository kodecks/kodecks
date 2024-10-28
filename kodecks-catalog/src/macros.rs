#[macro_export]
macro_rules! card_def {
    ( $struct:ident, $id:literal, $name:literal, $( $key:ident : $value:expr, )*) => {
        #[derive(Clone, Copy)]
        pub struct $struct;

        pub const ARCHETYPE: fn() -> &'static CardArchetype = || {
            static CACHE: std::sync::LazyLock<CardArchetype> = std::sync::LazyLock::new(|| CardArchetype {
                id: ArchetypeId::new($id),
                name: $name.to_string(),
                safe_name: kodecks::card::safe_name($name).unwrap(),
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

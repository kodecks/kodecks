use bpaf::Bpaf;
use kodecks::{card::safe_name, color::Color};
use kodecks_catalog::CATALOG;
use std::io::Write;
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::Path,
};

fn main() {
    let opts = options().run();
    if let Some(name) = opts.card_name {
        add_card(&name, opts.card_id);
    } else {
        show_stat();
    }
}

fn add_card(name: &str, id: Option<String>) {
    let safe_name = safe_name(name).unwrap();
    let id = id.unwrap_or_else(|| {
        safe_name
            .to_string()
            .replace('-', "")
            .chars()
            .take(4)
            .collect()
    });

    let asset_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../kodecks-bevy/assets");
    let card_asset_dir = asset_dir.join("cards").join(&safe_name);

    fs::create_dir_all(&card_asset_dir).unwrap();
    fs::write(card_asset_dir.join("image.main.png"), []).unwrap();

    let file_name = safe_name.replace('-', "_");

    for lang in &["en-US", "ja-JP"] {
        let file = asset_dir.join("locales").join(lang).join("cards.ftl");
        let mut file = fs::OpenOptions::new().append(true).open(file).unwrap();
        writeln!(file, "card-{safe_name} = ").unwrap();
    }

    let cards_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cards");
    let card_file = cards_dir.join(format!("{}.rs", file_name));
    fs::write(
        card_file,
        format!(
            r#"use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "{id}",
    "{name}",
    color: Color::GREEN,
    cost: 3,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 200,
);

impl Effect for CardDef {{}}
"#
        ),
    )
    .unwrap();

    let card_mod = cards_dir.join("mod.rs");
    let data = fs::read_to_string(&card_mod)
        .unwrap()
        .replace(
            "// card-modules\n",
            &format!("// card-modules\nmod {};\n", file_name),
        )
        .replace(
            "// card-entries\n",
            &format!(
                r#"// card-entries
    "{safe_name}" => {file_name}::ARCHETYPE,
    "{id}" => {file_name}::ARCHETYPE,
"#
            ),
        );
    fs::write(card_mod, data).unwrap();
    std::process::Command::new("cargo")
        .arg("fmt")
        .output()
        .unwrap();
}

fn show_stat() {
    let mut archetypes = HashSet::new();
    let mut colors = vec![(0, "Blue"), (0, "Green"), (0, "Yellow"), (0, "Red")];
    let mut costs = vec![];
    let mut powers = BTreeMap::new();

    CATALOG.iter().for_each(|card| {
        if card.attribute.is_token {
            return;
        }
        if archetypes.contains(&card.id) {
            return;
        }
        archetypes.insert(card.id);

        let color = card.attribute.color;
        colors[0].0 += color.contains(Color::BLUE) as i32;
        colors[1].0 += color.contains(Color::GREEN) as i32;
        colors[2].0 += color.contains(Color::YELLOW) as i32;
        colors[3].0 += color.contains(Color::RED) as i32;

        let cost = card.attribute.cost;
        if costs.len() <= cost as usize {
            costs.resize(cost as usize + 1, 0);
        }
        costs[cost as usize] += 1;

        if let Some(power) = card.attribute.power {
            *powers.entry(power).or_insert(0) += 1;
        }
    });

    println!("Color\tCount");
    println!("-----\t-----");
    for (count, name) in colors {
        if count > 0 {
            println!("{}\t{}", name, count);
        }
    }

    println!("\nCost\tCount");
    println!("----\t-----");
    for (cost, count) in costs.iter().enumerate() {
        println!("{}\t{}", cost, count);
    }

    println!("\nPower\tCount");
    println!("-----\t-----");
    for (power, count) in powers {
        println!("{}\t{}", power, count);
    }
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(positional("NAME"))]
    /// Add a new card
    card_name: Option<String>,

    #[bpaf(positional("ID"))]
    /// Specify the card ID
    card_id: Option<String>,
}

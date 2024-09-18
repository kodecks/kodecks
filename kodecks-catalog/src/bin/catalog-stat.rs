use std::collections::BTreeMap;

use kodecks::color::Color;
use kodecks_catalog::CATALOG;

fn main() {
    let mut colors = vec![(0, "Blue"), (0, "Green"), (0, "Yellow"), (0, "Red")];
    let mut costs = vec![];
    let mut powers = BTreeMap::new();

    CATALOG.iter().for_each(|card| {
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

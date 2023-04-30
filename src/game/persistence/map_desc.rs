use rand::Rng;

use crate::prelude::*;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MapDesc {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<u32>,
    pub areas: Vec<WorldArea>,
}

impl MapDesc {
    pub fn get_tile(&self, x: u32, y: u32) -> u32 {
        self.tiles[(x + y * self.width) as usize]
    }

    pub fn get_area(&self, x: u32, y: u32) -> Option<&WorldArea> {
        self.areas.iter().find(|a| a.world_position == (x, y))
    }
}

const ADVERBS: [&str; 10] = [
    "Ever", "Long", "True", "False", "Seldomly", "Greatly", "Abysmal", "Wondered", "Far", "Near",
];

const ADJECTIVES: [&str; 10] = [
    "Shining", "Dark", "New", "Old", "Great", "Small", "Big", "Little", "Red", "High",
];

const CITY_NOUNS: [&str; 10] = [
    "Haven", "Bastion", "Hammer", "Anvil", "Forge", "Hearth", "City", "", "Vale", "Valley",
];

const VILLAGE_NOUNS: [&str; 10] = [
    "Hollow", "Valley", "Cairn", "Bend", "Hole", "Pond", "Dale", "Meet", "Ford", "End",
];

const CITY: u32 = 4;
const VILLAGE: u32 = 3;

fn generate_village_name(rng: &mut StdRng) -> String {
    match rng.gen_range(0..=4) {
        0 => {
            format!(
                "The {} {} {}",
                choose(rng, &ADVERBS),
                choose(rng, &ADJECTIVES),
                choose(rng, &VILLAGE_NOUNS)
            )
        }
        1 => {
            format!(
                "{} {} of the {}",
                choose(rng, &ADVERBS),
                choose(rng, &ADJECTIVES),
                choose(rng, &VILLAGE_NOUNS)
            )
        }
        1 => {
            format!(
                "{} of {}",
                choose(rng, &VILLAGE_NOUNS),
                choose(rng, &ADJECTIVES)
            )
        }
        2 => {
            format!(
                "{} {}",
                choose(rng, &ADJECTIVES),
                choose(rng, &VILLAGE_NOUNS)
            )
        }
        3 => {
            format!("{} {}", choose(rng, &ADVERBS), choose(rng, &VILLAGE_NOUNS))
        }
        4 => {
            format!(
                "{} {} {}",
                choose(rng, &ADVERBS),
                choose(rng, &ADVERBS),
                choose(rng, &VILLAGE_NOUNS)
            )
        }
        idx => panic!("Bad village template {}", idx),
    }
}

fn generate_city_name(rng: &mut StdRng) -> String {
    match rng.gen_range(0..=4) {
        0 => {
            format!(
                "The {} {} {}",
                choose(rng, &ADVERBS),
                choose(rng, &ADJECTIVES),
                choose(rng, &CITY_NOUNS)
            )
        }
        1 => {
            format!(
                "{} {} of the {}",
                choose(rng, &ADVERBS),
                choose(rng, &ADJECTIVES),
                choose(rng, &CITY_NOUNS)
            )
        }
        1 => {
            format!(
                "{} of {}",
                choose(rng, &CITY_NOUNS),
                choose(rng, &ADJECTIVES)
            )
        }
        2 => {
            format!("{} {}", choose(rng, &ADJECTIVES), choose(rng, &CITY_NOUNS))
        }
        3 => {
            format!("{} {}", choose(rng, &ADVERBS), choose(rng, &CITY_NOUNS))
        }
        4 => {
            format!(
                "{} {} {}",
                choose(rng, &ADVERBS),
                choose(rng, &ADVERBS),
                choose(rng, &CITY_NOUNS)
            )
        }
        idx => panic!("Bad city template {}", idx),
    }
}

fn generate_city_population(rng: &mut StdRng, area: &mut WorldArea) {
    let population = rng.gen_range(80..=100);
    let upper_class = rng.gen_range(2..=population / 10);
    let middle_class = rng.gen_range(2..=population / 5);
    let lower_class = population - upper_class - middle_class;
    for _ in 0..upper_class {
        area.followers.push(Follower::new(rng.gen_range(20..30)));
    }
    for _ in 0..middle_class {
        area.followers.push(Follower::new(rng.gen_range(10..20)));
    }
    for _ in 0..lower_class {
        area.followers.push(Follower::new(rng.gen_range(1..10)));
    }
}

fn generate_village_population(rng: &mut StdRng, area: &mut WorldArea) {
    let population = rng.gen_range(30..=50);
    let middle_class = rng.gen_range(2..=population / 5);
    let lower_class = population - middle_class;
    for _ in 0..middle_class {
        area.followers.push(Follower::new(rng.gen_range(10..20)));
    }
    for _ in 0..lower_class {
        area.followers.push(Follower::new(rng.gen_range(1..10)));
    }
}

const PREFIX: [&str; 15] = [
    "Jarn", "Ax", "Tan", "Ev", "Be", "Log", "Bo", "Ko", "Ser", "Kor", "Al", "Kil", "Yet", "Nar",
    "So",
];

const SUFFIX: [&str; 16] = [
    "athan", "an", "os", "ex", "ra", "i", "u", "na", "ni", "a", "us", "or", "on", "athan", "en",
    "in",
];

fn generate_agent_name(rng: &mut StdRng) -> String {
    format!("{}{}", choose(rng, &PREFIX), choose(rng, &SUFFIX))
}

fn generate_area(rng: &mut StdRng, x: u32, y: u32, tile: u32) -> Option<WorldArea> {
    match tile {
        CITY => {
            let mut area = WorldArea::new(&generate_city_name(rng), x, y);
            generate_city_population(rng, &mut area);
            Some(area)
        }
        VILLAGE => {
            let mut area = WorldArea::new(&generate_village_name(rng), x, y);
            generate_village_population(rng, &mut area);
            Some(area)
        }
        _ => None,
    }
}

fn fill_neighbors(areas: &mut Vec<WorldArea>) {
    let mut neighbors = Vec::new();
    for _ in areas.iter() {
        let mut my_neighbors = Vec::new();
        for neighbor in areas.iter() {
            my_neighbors.push(neighbor.world_position);
        }
        neighbors.push(my_neighbors);
    }
    for (idx, mut neighbors) in neighbors.drain(..).enumerate() {
        neighbors.sort_by(|a, b| {
            let a_dist = (a.0 as isize - areas[idx].world_position.0 as isize).abs()
                + (a.1 as isize - areas[idx].world_position.1 as isize).abs();
            let b_dist = (b.0 as isize - areas[idx].world_position.0 as isize).abs()
                + (b.1 as isize - areas[idx].world_position.1 as isize).abs();
            a_dist.cmp(&b_dist)
        });
        areas[idx].nearest_neighbors = neighbors;
    }
}

pub fn generate_map(mut players: Vec<PlayerId>) -> MapDesc {
    players.sort();
    let seed = players
        .iter()
        .map(|p| p.0 as u64)
        .reduce(|a, b| (a << 32) ^ b)
        .unwrap_or(8675309);
    let mut rng = StdRng::seed_from_u64(seed as u64);
    let width = rng.gen_range(7..=10);
    let height = rng.gen_range(7..=10);
    let mut tiles = vec![0; width * height];
    let mut areas = Vec::new();
    let population_count = rng.gen_range((players.len() * 12)..(players.len() * 18));
    println!("Population count: {}", population_count);
    for _ in 0..population_count {
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);
        if tiles[x + y * width] == VILLAGE {
            tiles[x + y * width] = CITY;
        } else if tiles[x + y * width] == 0 {
            tiles[x + y * width] = VILLAGE;
        }
    }
    for idx in 0..(width * height) {
        if tiles[idx] == 0 {
            tiles[idx] = rng.gen_range(0..3);
        }
        if tiles[idx] == VILLAGE || tiles[idx] == CITY {
            let x = idx % width;
            let y = idx / width;
            if let Some(new_area) = generate_area(&mut rng, x as u32, y as u32, tiles[idx]) {
                areas.push(new_area);
            }
        }
    }
    fill_neighbors(&mut areas);
    let mut valid_agent_locations = areas
        .iter()
        .map(|a| a.world_position)
        .collect::<Vec<(u32, u32)>>();
    for player in players.iter() {
        for id in 0..=3 {
            let (x, y) = choose(&mut rng, &valid_agent_locations);
            println!("Agent {} at {}, {}", id, x, y);
            valid_agent_locations.retain(|p| *p != (x, y));
            let mut agent = Agent::new(
                generate_agent_name(&mut rng),
                AgentId {
                    player: *player,
                    agent: id,
                },
                (x, y),
                10 + id * 2,
            );
            agent.world_position = (x, y);
            areas
                .iter_mut()
                .find(|a| a.world_position == (x, y))
                .unwrap()
                .agents
                .push(agent);
        }
    }
    for _ in 0..players.len() {
        for _ in 0..4 {
            let (x, y) = choose(&mut rng, &valid_agent_locations);
            println!("Sign at {}, {}", x, y);
            valid_agent_locations.retain(|p| *p != (x, y));
            areas
                .iter_mut()
                .find(|a| a.world_position == (x, y))
                .unwrap()
                .followers
                .first_mut()
                .map(|follower| follower.sign_holder = true);
        }
    }
    MapDesc {
        width: width as u32,
        height: height as u32,
        tiles,
        areas,
    }
}

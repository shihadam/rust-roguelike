//! Group of functions relating to spawning entities
//! and spawning mineable items on the map.

use crate::prelude::*;

pub fn player_spawn_point(map: &Map) -> Point {
    let center = Point::new(MAP_WIDTH / 2, MAP_HEIGHT / 2);

    // get a vec of all the floor tiles
    let mut tmp = Vec::new();
    for idx in 0..map.tiles.len() {
        if map.tiles[idx] == TileType::Floor {
            tmp.push(idx);
        }
    }

    // find the closest tile to the center
    let mut closest_idx = tmp[0];
    for tile in &tmp {
        if DistanceAlg::Pythagoras.distance2d(center, map.index_to_point2d(*tile))
            < DistanceAlg::Pythagoras.distance2d(center, map.index_to_point2d(closest_idx))
        {
            closest_idx = *tile;
        }
    }

    map.index_to_point2d(closest_idx)
}

/// Finds spawn locations for enemies on a map.
pub fn enemy_spawns(map: &Map, rng: &mut RandomNumberGenerator) -> Vec<Point> {
    let safe_distance = 10;
    let num_spawns = 20;

    // get vec of all spawnable tiles
    // only tiles that are safe distance away from player spawn
    let mut spawnable = Vec::new();
    for idx in 0..map.tiles.len() {
        if map.tiles[idx] == TileType::Floor
            && DistanceAlg::Pythagoras.distance2d(map.player_spawn_point, map.index_to_point2d(idx))
                as i32
                > safe_distance
        {
            spawnable.push(map.index_to_point2d(idx));
        }
    }

    let mut spawns = Vec::new();
    for _ in 0..num_spawns {
        let idx = rng.random_slice_index(&spawnable).unwrap();
        spawns.push(spawnable[idx]);
        spawnable.remove(idx);
    }

    spawns
}

pub fn spawn_red_crystal(map: &mut Map, rng: &mut RandomNumberGenerator) {
    let num_spawns = 7;
    let mut spawnable = Vec::new();
    for idx in 0..map.tiles.len() {
        if map.tiles[idx] == TileType::Wall {
            spawnable.push(map.index_to_point2d(idx));
        }
    }

    let mut spawn_cores = Vec::new();
    for _ in 0..num_spawns {
        let idx = rng.random_slice_index(&spawnable).unwrap();
        spawn_cores.push(spawnable[idx]);
        spawnable.remove(idx);
    }

    let mut spawns = Vec::new();
    for core in spawn_cores {
        let prefabs = vec![
            // bar1
            vec![Point::new(1, 0)],
            // bar2
            vec![Point::new(-1, 0)],
            // vert1
            vec![Point::new(0, 1)],
            // vert2
            vec![Point::new(0, -1)],
            // box1
            vec![Point::new(1, 0), Point::new(1, -1), Point::new(0, -1)],
            // box2
            vec![Point::new(-1, 0), Point::new(-1, -1), Point::new(0, -1)],
            // box3
            vec![Point::new(1, 0), Point::new(1, 1), Point::new(0, 1)],
            // box4
            vec![Point::new(-1, 0), Point::new(-1, 1), Point::new(0, 1)],
        ];

        spawns.push(map.point2d_to_index(core));

        let choice = rng.range(0, prefabs.len());
        prefabs[choice].iter().for_each(|pt| {
            if map.in_bounds(core + *pt) {
                spawns.push(map.point2d_to_index(core + *pt))
            }
        });
    }

    spawns
        .iter()
        .for_each(|idx| map.tiles[*idx] = TileType::RedCrystal);
}

pub fn spawn_gold(map: &mut Map, rng: &mut RandomNumberGenerator) {
    let num_spawns = 9;
    let mut spawnable = Vec::new();

    for idx in 0..map.tiles.len() {
        if map.tiles[idx] == TileType::Wall {
            spawnable.push(map.index_to_point2d(idx));
        }
    }

    let mut spawn_cores = Vec::new();
    for _ in 0..num_spawns {
        let idx = rng.random_slice_index(&spawnable).unwrap();
        spawn_cores.push(spawnable[idx]);
        spawnable.remove(idx);
    }

    let mut spawns = Vec::new();
    for core in spawn_cores {
        let prefabs = vec![
            // horizontal
            vec![Point::new(1, 0), Point::new(-1, 0)],
            // vertical
            vec![Point::new(0, 1), Point::new(0, -1)],
            // right up L
            vec![Point::new(0, -1), Point::new(1, 0)],
            // left up L
            vec![Point::new(0, -1), Point::new(-1, 0)],
            // right down L
            vec![Point::new(0, 1), Point::new(1, 0)],
            // left down L
            vec![Point::new(0, 1), Point::new(-1, 0)],
            // up T
            vec![Point::new(1, 0), Point::new(-1, 0), Point::new(0, -1)],
            // down T
            vec![Point::new(1, 0), Point::new(-1, 0), Point::new(0, 1)],
            // right T
            vec![Point::new(0, 1), Point::new(0, -1), Point::new(1, 0)],
            // left T
            vec![Point::new(0, 1), Point::new(0, -1), Point::new(-1, 0)],
            // star
            vec![
                Point::new(0, 1),
                Point::new(0, -1),
                Point::new(-1, 0),
                Point::new(1, 0),
            ],
        ];

        spawns.push(map.point2d_to_index(core));

        let choice = rng.range(0, prefabs.len());
        prefabs[choice].iter().for_each(|pt| {
            if map.in_bounds(core + *pt) {
                spawns.push(map.point2d_to_index(core + *pt))
            }
        });
    }

    spawns
        .iter()
        .for_each(|idx| map.tiles[*idx] = TileType::Gold);
}

pub fn spawn_giant_gem(map: &mut Map, rng: &mut RandomNumberGenerator) {
    let min_distance = 30;

    let mut spawnable = Vec::new();
    for idx in 0..map.tiles.len() {
        if
        /*map.tiles[idx] != TileType::Floor
        &&*/
        DistanceAlg::Pythagoras.distance2d(map.player_spawn_point, map.index_to_point2d(idx))
            as i32
            > min_distance
        {
            spawnable.push(map.index_to_point2d(idx));
        }
    }

    let idx = rng.random_slice_index(&spawnable).unwrap();
    map.tiles[idx] = TileType::GiantGem;
}

/// Push the player into the ecs and spawn them onto the map
pub fn spawn_player(ecs: &mut World, position: Point) {
    ecs.push((
        Player,
        Name("Player".to_string()),
        position,
        Renderable {
            color: ColorPair::new(WHITE, BLACK),
            glyph: to_cp437('@'),
        },
        Stats {
            max_health: 20,
            health: 20,
            damage: 5,
        },
        MineRange(1),
    ));
}

pub fn spawn_enemies(ecs: &mut World, spawns: &[Point], rng: &mut RandomNumberGenerator) {
    for spawn in spawns {
        match rng.range(0, 3) {
            0 | 1 => spawn_cave_spider(ecs, *spawn),
            2 => spawn_goblin(ecs, *spawn),
            _ => (),
        }
    }
}

pub fn spawn_cave_spider(ecs: &mut World, position: Point) {
    ecs.push((
        Enemy,
        Name("Cave Spider".to_string()),
        position,
        Renderable {
            color: ColorPair::new(CHOCOLATE, BLACK),
            glyph: to_cp437('s'),
        },
        Stats {
            max_health: 5,
            health: 5,
            damage: 1,
        },
        RandomMovement {},
    ));
}

pub fn spawn_goblin(ecs: &mut World, position: Point) {
    ecs.push((
        Enemy,
        Name("Goblin".to_string()),
        position,
        Renderable {
            color: ColorPair::new(GREEN, BLACK),
            glyph: to_cp437('g'),
        },
        Stats {
            max_health: 15,
            health: 15,
            damage: 3,
        },
        TargetedMovement {},
    ));
}

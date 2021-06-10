//! Map storage and cellular automata generation.

use crate::prelude::*;

/// Tiletypes used in the map.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TileType {
    Wall,
    Floor,
    Gold,
    RedCrystal,
    GiantGem,
}

/// Map generation and storage struct.
pub struct Map {
    pub tiles: Vec<TileType>,
    pub player_spawn_point: Point,
    pub enemy_spawns: Vec<Point>,
}

/// BaseMap implementation for use with DijkstraMap.
impl BaseMap for Map {
    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let pos = self.index_to_point2d(idx);
        let default_cost = 1.0;

        if let Some(idx) = self.can_exit(pos, Point::new(1, 0)) {
            exits.push((idx, default_cost));
        }

        if let Some(idx) = self.can_exit(pos, Point::new(-1, 0)) {
            exits.push((idx, default_cost));
        }

        if let Some(idx) = self.can_exit(pos, Point::new(0, -1)) {
            exits.push((idx, default_cost));
        }

        if let Some(idx) = self.can_exit(pos, Point::new(0, 1)) {
            exits.push((idx, default_cost));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        DistanceAlg::Pythagoras.distance2d(self.index_to_point2d(idx1), self.index_to_point2d(idx2))
    }
}

/// Algorithm2D implmentation for index_to_point2d and point2d_to_index.
impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(MAP_WIDTH, MAP_HEIGHT)
    }
}

impl Map {
    pub fn new(rng: &mut RandomNumberGenerator) -> Self {
        let mut a_map = Self {
            tiles: vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize],
            player_spawn_point: Point::zero(),
            enemy_spawns: Vec::new(),
        };

        a_map.initialize(rng);
        for _ in 0..5 {
            a_map.step();
        }
        a_map.player_spawn_point = player_spawn_point(&a_map);
        a_map.enemy_spawns = enemy_spawns(&a_map, rng);
        spawn_gold(&mut a_map, rng);
        spawn_red_crystal(&mut a_map, rng);
        spawn_giant_gem(&mut a_map, rng);

        a_map
    }

    pub fn can_enter(&self, position: Point) -> bool {
        self.in_bounds(position) && self.tiles[self.point2d_to_index(position)] == TileType::Floor
    }

    pub fn can_exit(&self, position: Point, delta: Point) -> Option<usize> {
        let new_position = position + delta;

        match self.can_enter(new_position) {
            true => Some(self.point2d_to_index(new_position)),
            false => None,
        }
    }

    pub fn can_mine(&self, target_pos: Point) -> bool {
        self.in_bounds(target_pos)
            && self.tiles[self.point2d_to_index(target_pos)] != TileType::Floor
    }

    /// Iterate over every tile and set to Wall or Floor based
    /// on a set probability.
    fn initialize(&mut self, rng: &mut RandomNumberGenerator) {
        let chance_to_be_wall: f32 = 0.42;

        for t in self.tiles.iter_mut() {
            if rng.range::<f32>(0.0, 1.0) < chance_to_be_wall {
                *t = TileType::Wall;
            } else {
                *t = TileType::Floor;
            }
        }
    }

    /// Count the number of walls neighboring a map location.
    fn count_neighbor_walls(&self, x: i32, y: i32) -> i32 {
        let mut count = 0;

        for i in -1..=1 {
            for j in -1..=1 {
                if !(i == 0 && j == 0)
                    && self.in_bounds(Point::from_tuple((x + i, y + j)))
                    && self.tiles[self.point2d_to_index(Point::new(x + i, y + j))] == TileType::Wall
                {
                    count += 1;
                }
            }
        }

        count
    }

    /// Proceed one time step forwards. 
    /// ### Rules
    ///     * If Wall and < 3 neighbors, become Floor.
    ///     * If Floor and > 4 neighbors, become Wall.
    fn step(&mut self) {
        let mut new_map = self.tiles.clone();
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let neighbor_walls = self.count_neighbor_walls(x, y);
                let idx = self.point2d_to_index(Point::new(x, y));
                if self.tiles[idx] == TileType::Wall && neighbor_walls < 3 {
                    new_map[idx] = TileType::Floor;
                } else if self.tiles[idx] == TileType::Floor && neighbor_walls > 4 {
                    new_map[idx] = TileType::Wall;
                }
            }
        }
        self.tiles = new_map;
    }
}

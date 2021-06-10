//! Legion systems which are called in a schedule.

use crate::prelude::*;

/// Game states. Used to handle turns, loss, and win.
#[derive(Clone, Debug)]
pub enum TurnState {
    Input,
    Player,
    Enemy,
    GameOver,
    Victory,
}

/// Workaround for left_click registering twice per click in bracket-lib.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ClickLock(pub bool);

/// Game stats storage.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct GameStats {
    pub gold: i32,
    pub kills: i32,
    pub steps: i32,
    pub slain_by: Option<String>,
    pub has_gem: bool,
}

/// Input values grouping to reduce function argument counts.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct InputValues {
    pub key: Option<VirtualKeyCode>,
    pub mouse_pos: (i32, i32),
    pub left_click: bool,
}

/// Checks if target is in range of position.
pub fn in_range(pos: Point, target: Point, range: i32) -> bool {
    (pos.x - target.x).abs() <= range && (pos.y - target.y).abs() <= range
}

/// Handles player input.
#[system]
#[read_component(Point)]
#[read_component(Player)]
#[read_component(Enemy)]
#[read_component(MineRange)]
pub fn input(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] map: &Map,
    #[resource] input: &InputValues,
    #[resource] turnstate: &mut TurnState,
    #[resource] lock: &mut ClickLock,
) {
    let mut player = <(Entity, &Point, &MineRange)>::query().filter(component::<Player>());
    let mouse_pos = Point::from_tuple(input.mouse_pos);

    if input.left_click {
        // workaround left_click sensing mouse up.
        lock.0 = !lock.0;

        if !lock.0 {
            player.iter(ecs).for_each(|(entity, pos, mine_range)| {
                if map.can_mine(mouse_pos) && in_range(*pos, mouse_pos, mine_range.0) {
                    commands.push((
                        WantsToMine {
                            entity: *entity,
                            target: Point::new(mouse_pos.x, mouse_pos.y),
                        },
                        (),
                    ));
                    *turnstate = TurnState::Player;
                }
            });
        }
    } else if input.key != None {
        let delta = match input.key.unwrap() {
            VirtualKeyCode::A => Point::new(-1, 0),
            VirtualKeyCode::D => Point::new(1, 0),
            VirtualKeyCode::W => Point::new(0, -1),
            VirtualKeyCode::S => Point::new(0, 1),
            _ => Point::zero(),
        };

        let mut enemies = <(Entity, &Point)>::query().filter(component::<Enemy>());

        player.iter(ecs).for_each(|(player_entity, pos, _)| {
            let destination = *pos + delta;
            let mut attacking = false;

            enemies
                .iter(ecs)
                .filter(|(_, pos)| **pos == destination)
                .for_each(|(enemy_entity, _)| {
                    attacking = true;

                    commands.push((
                        WantsToAttack {
                            entity: *player_entity,
                            target: *enemy_entity,
                        },
                        (),
                    ));
                });

            if !attacking {
                commands.push((
                    WantsToMove {
                        entity: *player_entity,
                        destination,
                    },
                    (),
                ));
            }
        });
        *turnstate = TurnState::Player;
    }
}

/// Handles requests given by WantsToMine tag.
#[system]
#[read_component(WantsToMine)]
#[read_component(MineRange)]
#[write_component(Stats)]
#[read_component(Player)]
pub fn mining(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] map: &mut Map,
    #[resource] game_stats: &mut GameStats,
) {
    let mut entities = <(Entity, &WantsToMine)>::query();
    let (player_entity, player_stats) = <(Entity, &Stats)>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .find_map(|(entity, stats)| Some((entity, *stats)))
        .unwrap();

    entities.iter(ecs).for_each(|(flag, wants_to_mine)| {
        let idx = map.point2d_to_index(wants_to_mine.target);
        if map.can_mine(wants_to_mine.target) {
            match map.tiles[idx] {
                TileType::Gold => game_stats.gold += 1,
                TileType::RedCrystal => {
                    let mut new_stats = player_stats;
                    if player_stats.health < player_stats.max_health - 3 {
                        new_stats.health += 3;
                    } else if player_stats.health < player_stats.max_health {
                        new_stats.health = player_stats.max_health;
                    }
                    commands.add_component(*player_entity, new_stats);
                }
                TileType::GiantGem => game_stats.has_gem = true,
                _ => (),
            }
            map.tiles[idx] = TileType::Floor;
        }

        // remove messsage after processed
        commands.remove(*flag);
    });
}

/// Handles requests given by WantsToMove tag.
#[system]
#[read_component(WantsToMove)]
pub fn movement(ecs: &mut SubWorld, commands: &mut CommandBuffer, #[resource] map: &Map) {
    let mut entities = <(Entity, &WantsToMove)>::query();

    entities.iter(ecs).for_each(|(flag, wants_to_move)| {
        if map.can_enter(wants_to_move.destination) {
            commands.add_component(wants_to_move.entity, wants_to_move.destination);
        }

        // remove messsage after processed
        commands.remove(*flag);
    });
}

/// Handles requests given by WantsToAttack tag.
#[system]
#[write_component(Stats)]
#[read_component(WantsToAttack)]
#[read_component(Name)]
pub fn combat(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] game_log: &mut GameLog,
    #[resource] game_stats: &mut GameStats,
) {
    let mut attackers = <(Entity, &WantsToAttack)>::query();
    let flags: Vec<(Entity, Entity, Entity)> = attackers
        .iter(ecs)
        .map(|(flag, wants_to_attack)| (*flag, wants_to_attack.entity, wants_to_attack.target))
        .collect();

    flags.iter().for_each(|(flag, attacker, target)| {
        let (damage, attacker_name) = if let Ok(v) = ecs.entry_ref(*attacker) {
            if let (Ok(stats), Ok(name)) = (v.get_component::<Stats>(), v.get_component::<Name>()) {
                (stats.damage, name.0.clone())
            } else {
                (0, String::default())
            }
        } else {
            (0, String::default())
        };

        let target_name = if let Ok(v) = ecs.entry_ref(*target) {
            if let Ok(name) = v.get_component::<Name>() {
                name.0.clone()
            } else {
                String::default()
            }
        } else {
            String::default()
        };

        if let Ok(stats) = ecs.entry_mut(*target).unwrap().get_component_mut::<Stats>() {
            stats.health -= damage;
            game_log.log(format!(
                "{} attacked {} for {} dmg.",
                attacker_name, target_name, damage
            ));

            // if target dies and is not player, despawn
            if stats.health <= 0 {
                if target_name != "Player" {
                    commands.remove(*target);
                    game_log.log(format!("{} has been slain.", target_name));

                    if attacker_name == "Player" {
                        game_stats.kills += 1;
                    }
                } else {
                    game_stats.slain_by = Some(attacker_name);
                }
            }
        }

        // remove messsage after processed
        commands.remove(*flag);
    });
}

/// Renders map. Sent through a DrawBatch.
#[system]
#[write_component(Point)]
#[read_component(Player)]
#[read_component(MineRange)]
pub fn map_render(ecs: &mut SubWorld, #[resource] map: &Map, #[resource] input: &InputValues) {
    let mouse_pos = Point::from_tuple(input.mouse_pos);
    let mut draw_batch = DrawBatch::new();
    let (player_pos, player_range) = <(&Point, &MineRange)>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .find_map(|(pos, range)| Some((*pos, *range)))
        .unwrap();

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let idx = map.point2d_to_index(Point::new(x, y));
            match map.tiles[idx] {
                TileType::Floor => draw_batch.set(
                    Point::from_tuple((x, y)),
                    ColorPair::new(WHITE, BLACK),
                    to_cp437('.'),
                ),
                TileType::Wall => {
                    let color = {
                        if map.point2d_to_index(mouse_pos) == idx {
                            if in_range(player_pos, mouse_pos, player_range.0) {
                                ColorPair::new(CYAN, BLACK)
                            } else {
                                ColorPair::new(DARK_CYAN, BLACK)
                            }
                        } else {
                            ColorPair::new(BURLYWOOD, BLACK)
                        }
                    };

                    draw_batch.set(Point::new(x, y), color, to_cp437('#'))
                }
                TileType::Gold => {
                    let color = {
                        if map.point2d_to_index(mouse_pos) == idx {
                            ColorPair::new(GOLDENROD, BLACK)
                        } else {
                            ColorPair::new(GOLD, BLACK)
                        }
                    };

                    draw_batch.set(Point::new(x, y), color, to_cp437('#'))
                }
                TileType::RedCrystal => {
                    let color = {
                        if map.point2d_to_index(mouse_pos) == idx {
                            ColorPair::new(DARK_RED, BLACK)
                        } else {
                            ColorPair::new(RED, BLACK)
                        }
                    };

                    draw_batch.set(Point::new(x, y), color, to_cp437('#'))
                }
                TileType::GiantGem => {
                    let color = {
                        if map.point2d_to_index(mouse_pos) == idx {
                            ColorPair::new(DARK_VIOLET, BLACK)
                        } else {
                            ColorPair::new(VIOLET, BLACK)
                        }
                    };

                    draw_batch.set(Point::new(x, y), color, to_cp437('#'))
                }
            };
        }
    }

    draw_batch.submit(0).expect("Batch error");
}

/// Diplays all Renderable entities. Sent through a DrawBatch.
#[system]
#[read_component(Point)]
#[read_component(Renderable)]
pub fn entity_render(ecs: &SubWorld) {
    let mut draw_batch = DrawBatch::new();
    <(&Point, &Renderable)>::query()
        .iter(ecs)
        .for_each(|(position, renderable)| {
            draw_batch.set(*position, renderable.color, renderable.glyph);
        });
    draw_batch.submit(8000).expect("Batch error");
}

/// Handles TurnState switching.
#[system]
#[read_component(Player)]
#[read_component(Stats)]
pub fn cycle_turn(
    ecs: &mut SubWorld,
    #[resource] turnstate: &mut TurnState,
    #[resource] game_stats: &GameStats,
) {
    let current_state = turnstate.clone();
    let mut new_state = match current_state {
        TurnState::Player => TurnState::Enemy,
        TurnState::Enemy => TurnState::Input,
        _ => current_state,
    };

    // check game over
    <&Stats>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .for_each(|stats| {
            if stats.health <= 0 {
                new_state = TurnState::GameOver;
            }
        });

    // check victory
    if game_stats.has_gem {
        new_state = TurnState::Victory;
    }

    *turnstate = new_state;
}

/// Randomly moves entities with the RandomMovement tag.
#[system]
#[read_component(Point)]
#[read_component(Renderable)]
#[read_component(RandomMovement)]
#[read_component(Stats)]
#[read_component(Player)]
pub fn random_movement(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] rng: &mut RandomNumberGenerator,
) {
    let mut entities = <(Entity, &Point)>::query()
        .filter(component::<Renderable>() & component::<RandomMovement>());
    let mut positions = <(Entity, &Point)>::query().filter(component::<Stats>());
    let mut collided = false;

    entities.iter(ecs).for_each(|(entity, pos)| {
        let delta = match rng.range(0, 4) {
            0 => Point::new(-1, 0),
            1 => Point::new(1, 0),
            2 => Point::new(0, 1),
            _ => Point::new(0, -1),
        };
        let destination = *pos + delta;

        positions
            .iter(ecs)
            .filter(|(_, target_pos)| **target_pos == destination)
            .for_each(|(target, _)| {
                collided = true;
                // if collided target is a player then attack
                if ecs
                    .entry_ref(*target)
                    .unwrap()
                    .get_component::<Player>()
                    .is_ok()
                {
                    commands.push((
                        WantsToAttack {
                            entity: *entity,
                            target: *target,
                        },
                        (),
                    ));
                }
            });

        if !collided {
            commands.push((
                WantsToMove {
                    entity: *entity,
                    destination,
                },
                (),
            ));
        }
    });
}

/// Moves entities with TargetedMovement towards Player.
#[system]
#[read_component(Point)]
#[read_component(Renderable)]
#[read_component(Stats)]
#[read_component(Player)]
#[read_component(TargetedMovement)]
pub fn targeted_movement(ecs: &mut SubWorld, commands: &mut CommandBuffer, #[resource] map: &Map) {
    let mut entities = <(Entity, &Point)>::query()
        .filter(component::<Renderable>() & component::<TargetedMovement>());
    let mut positions = <(Entity, &Point)>::query().filter(component::<Stats>());
    let player_pos = <&Point>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .last()
        .unwrap();
    let player_idx = map.point2d_to_index(*player_pos);
    let mut collided = false;

    // create the dijkstra map to the player
    let dijkstra_map = DijkstraMap::new(MAP_WIDTH, MAP_HEIGHT, &[player_idx], map, 1024.0);

    // move each entity to the player
    entities.iter(ecs).for_each(|(entity, pos)| {
        let entity_idx = map.point2d_to_index(*pos);

        if let Some(destination) = DijkstraMap::find_lowest_exit(&dijkstra_map, entity_idx, map) {
            let distance = DistanceAlg::Pythagoras.distance2d(*pos, *player_pos);
            let destination = if distance > 1.2 {
                map.index_to_point2d(destination)
            } else {
                *player_pos
            };

            positions
                .iter(ecs)
                .filter(|(_, target_pos)| **target_pos == destination)
                .for_each(|(target, _)| {
                    collided = true;
                    // if collided target is a player then attack
                    if ecs
                        .entry_ref(*target)
                        .unwrap()
                        .get_component::<Player>()
                        .is_ok()
                    {
                        commands.push((
                            WantsToAttack {
                                entity: *entity,
                                target: *target,
                            },
                            (),
                        ));
                    }
                });

            if !collided {
                commands.push((
                    WantsToMove {
                        entity: *entity,
                        destination,
                    },
                    (),
                ));
            }
        }
    });
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn in_range_test() {
        let p1 = Point::new(0, 0);
        let p2 = Point::new(1, 1);
        let range = 1;
        assert!(in_range(p1, p2, range));
    }
}

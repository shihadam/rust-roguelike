//! Combat log and stats display.

use crate::prelude::*;

/// Log of entries.
pub struct GameLog {
    pub entries: Vec<String>,
}

impl GameLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn log(&mut self, msg: String) {
        self.entries.push(msg);
    }
}

/// System to render the UI. Sent through a DrawBatch.
#[system]
#[read_component(Player)]
#[read_component(Stats)]
pub fn ui_render(
    ecs: &mut SubWorld,
    #[resource] game_log: &mut GameLog,
    #[resource] game_stats: &GameStats,
) {
    let ui_x = SCREEN_WIDTH - (SCREEN_WIDTH - MAP_WIDTH);
    let width = SCREEN_WIDTH - MAP_WIDTH;
    let mut draw_batch = DrawBatch::new();

    let stats = <&Stats>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .last()
        .unwrap();

    // ui border
    draw_batch.draw_box(
        Rect::with_exact(MAP_WIDTH, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1),
        ColorPair::new(WHITE, BLACK),
    );

    draw_batch.draw_box(
        Rect::with_exact(0, MAP_HEIGHT, MAP_WIDTH - 1, SCREEN_HEIGHT - 1),
        ColorPair::new(WHITE, BLACK),
    );

    // hp text
    draw_batch.print(
        Point::new(ui_x + 2, 2),
        format!("HP: {}/{}", stats.health, stats.max_health),
    );

    // hp bar
    draw_batch.bar_horizontal(
        Point::new(ui_x + 2, 4),
        width - 4,
        stats.health,
        stats.max_health,
        ColorPair::new(RED, BLACK),
    );

    // stats
    draw_batch.print(Point::new(ui_x + 2, 6), format!("Damage: {}", stats.damage));
    draw_batch.print(
        Point::new(ui_x + 2, 8),
        format!("Gold: {}", game_stats.gold),
    );
    draw_batch.print(
        Point::new(ui_x + 2, 10),
        format!("Kills: {}", game_stats.kills),
    );

    // output log messages
    let mut entries = game_log.entries.clone();
    for n in 1..=6 {
        if let Some(msg) = entries.pop() {
            draw_batch.print(Point::new(2, SCREEN_HEIGHT - 2 * n), msg);
        }
    }

    draw_batch.submit(0).unwrap();
}

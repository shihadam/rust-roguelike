//! A Roguelike game using bracket-lib and legion ecs.

mod components;
mod map;
mod schedule;
mod spawner;
mod systems;
mod ui;

/// Grouping of imports and globals for convienience.
mod prelude {

    pub use bracket_lib::prelude::*;
    pub use legion::systems::CommandBuffer;
    pub use legion::world::SubWorld;
    pub use legion::*;

    pub const SCREEN_WIDTH: i32 = 95;
    pub const SCREEN_HEIGHT: i32 = 60;
    pub const TILE_WIDTH: i32 = 12;
    pub const TILE_HEIGHT: i32 = 12;
    pub const MAP_WIDTH: i32 = 75;
    pub const MAP_HEIGHT: i32 = 45;

    pub use crate::components::*;
    pub use crate::map::*;
    pub use crate::schedule::*;
    pub use crate::spawner::*;
    pub use crate::systems::*;
    pub use crate::ui::*;
}

use prelude::*;

/// Holds ecs, resources, and schedules neccessary for the game.
struct State {
    ecs: World,
    resources: Resources,
    input_systems: Schedule,
    player_systems: Schedule,
    enemy_systems: Schedule,
}

impl State {
    fn new() -> Self {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let map = Map::new(&mut rng);
        let mut game_log = GameLog::new();
        let game_stats = GameStats::default();
        let lock = ClickLock(true);
        let input_values = InputValues::default();

        // initial messages in combat log.
        game_log.log("Welcome to Dwarf Game. You are a Dwarf.".to_string());
        game_log.log("WASD to move around. Click on tiles to mine them.".to_string());
        game_log.log("Find the GIANT GEM to prove your worth and win!".to_string());

        // spawn entities
        spawn_player(&mut ecs, map.player_spawn_point);
        spawn_enemies(&mut ecs, &map.enemy_spawns, &mut rng);

        // insert resources
        resources.insert(map);
        resources.insert(TurnState::Input);
        resources.insert(rng);
        resources.insert(game_log);
        resources.insert(game_stats);
        resources.insert(true);
        resources.insert(lock);
        resources.insert(input_values);

        Self {
            ecs,
            resources,
            input_systems: build_input_scheduler(),
            player_systems: build_player_scheduler(),
            enemy_systems: build_enemy_scheduler(),
        }
    }

    /// Restarts the game. New map generated and all stats reset.
    fn restart(&mut self) {
        self.ecs = World::default();
        self.resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let map = Map::new(&mut rng);
        let mut game_log = GameLog::new();
        let game_stats = GameStats::default();
        let lock = ClickLock(true);
        let input_values = InputValues::default();

        // initial messages in combat log
        game_log.log("Welcome to Dwarf Game. You are a Dwarf.".to_string());
        game_log.log("WASD to move around. Click on tiles to mine them.".to_string());
        game_log.log("Find the GIANT GEM to prove your worth and win!".to_string());

        // spawn entities
        spawn_player(&mut self.ecs, map.player_spawn_point);
        spawn_enemies(&mut self.ecs, &map.enemy_spawns, &mut rng);

        // insert resources
        self.resources.insert(map);
        self.resources.insert(TurnState::Input);
        self.resources.insert(rng);
        self.resources.insert(game_log);
        self.resources.insert(game_stats);
        self.resources.insert(lock);
        self.resources.insert(input_values);
    }

    /// Displays GameOver screen. Restarts game upon SPACEBAR press.
    fn game_over(&mut self, ctx: &mut BTerm) {
        let half = SCREEN_HEIGHT / 2;
        ctx.print_centered(half - 5, "You failed to retreive the GIANT GEM.");
        ctx.print_centered(
            half - 3,
            "You exit the cave in shame, a disgrace to Dwarfkind.",
        );
        ctx.print_color_centered(half, RED, BLACK, "Press SPACEBAR to play again ... ");

        // print game stats
        let stats = self.resources.get::<GameStats>().unwrap().clone();
        ctx.print_centered(half + 3, format!("Gold Collected: {}", stats.gold));
        ctx.print_centered(half + 5, format!("Enemies Slain: {}", stats.kills));
        ctx.print_centered(half + 7, format!("Slain By: {}", stats.slain_by.unwrap()));

        // restart game if key pressed
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.restart();
        }
    }

    /// Displays Victory screen. Restarts game upon SPACEBAR press.
    fn victory(&mut self, ctx: &mut BTerm) {
        let half = SCREEN_HEIGHT / 2;
        ctx.print_color_centered(
            half - 5,
            GREEN,
            BLACK,
            "Congradulations! You found the GIANT GEM!",
        );
        ctx.print_centered(half - 3, "You are now a dishtinguished Dwarf.");
        ctx.print_color_centered(half, RED, BLACK, "Press SPACEBAR to play again ... ");

        // print game stats
        let stats = self.resources.get::<GameStats>().unwrap().clone();
        ctx.print_centered(half + 3, format!("Gold Collected: {}", stats.gold));
        ctx.print_centered(half + 5, format!("Enemies Slain: {}", stats.kills));

        // restart game if key pressed
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.restart();
        }
    }
}

/// Bracket-lib GameState. Neccessary for game to run, supplies the tick() function, called once per frame.
impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        let input_values = InputValues {
            key: ctx.key,
            mouse_pos: ctx.mouse_pos(),
            left_click: ctx.left_click,
        };

        self.resources.insert(input_values);

        let turnstate = self.resources.get::<TurnState>().unwrap().clone();
        // println!("{:?}", turnstate);
        match turnstate {
            TurnState::Input => self
                .input_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::Player => self
                .player_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::Enemy => self
                .enemy_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::GameOver => {
                self.game_over(ctx);
            }
            TurnState::Victory => {
                self.victory(ctx);
            }
        }

        render_draw_buffer(ctx).expect("Render error");
    }
}

/// Game loop.
fn main() -> BError {
    // build BTerm
    let mut ctx: BTerm = BTermBuilder::simple(SCREEN_WIDTH, SCREEN_HEIGHT)
        .unwrap()
        .with_title("Dwarf Game")
        .with_tile_dimensions(TILE_WIDTH, TILE_HEIGHT)
        .build()?;

    // add cool scanlines and screen burn effect.
    ctx.with_post_scanlines(true);

    // run main loop
    main_loop(ctx, State::new())
}

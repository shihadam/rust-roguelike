# Dwarf Roguelike
Created by Adam Shih

Dwarf Roguelike is a roguelike game written in rust, using the bracket-lib game engine,
and legion ecs. It is a simple game, mine resources to
increase your stats, kill enemies, and find the "Giant Gem" to win.

## Usage
```
cargo run
```
From the project directory, run this command. It will build and run the binary for you.

## Game mechanics
### Monsters
* Spiders: Represented by an 's' these monsters move randomly, die in one hit and do 1 damage.
* Goblins: Represented by a 'g' are the more dangerous monster. They constantly move towards the
player and deal 3 damage per hit.

### Ores
* Gold: Mining this gives one gold per block to the player. It currently has no function other
than increasing the number in the UI.
* Red Crystal: Mining this adds 3 health to the player up to their max health per block.

![image](images/screenshot.png)

## References
https://bfnightly.bracketproductions.com/rustbook/  
https://pragprog.com/titles/hwrust/hands-on-rust/

## License
[MIT](https://github.com/shihadam/rust-roguelike/blob/main/LICENSE)

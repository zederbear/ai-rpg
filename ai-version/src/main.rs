use rand::Rng;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;
use crossterm::{
    execute,
    style::{Color, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode},
    ExecutableCommand,
};


#[derive(Debug)]
enum ClassType {
    MartialArtist,
    QiCultivator,
    Assassin,
}

#[derive(Debug)]
struct Player {
    name: String,
    class_type: ClassType,
    health: i32,
    attack: i32,
    defense: i32,
    qi: i32,
    cultivation_level: u32,
    gold: i32,
    bandits_defeated: u32,
    cultivation_speed: f32, // Add a field for cultivation speed bonus
    qi_pills: u32, // Track the number of Qi pills in inventory
}

#[derive(Debug)]
struct Enemy {
    name: String,
    health: i32,
    attack: i32,
    defense: i32,
    is_boss: bool, // Track if the enemy is the final boss
}

#[derive(Debug)]
struct Quest {
    description: String,
    reward: i32,
    completed: bool,
}

#[derive(Debug)]
struct Npc {
    name: String,
    quest: Quest,
}

const BUFFER_SIZE: usize = 5;
struct ConsoleBuffer {
    messages: Vec<String>,
}

impl ConsoleBuffer {
    fn new() -> Self {
        ConsoleBuffer { messages: Vec::new() }
    }

    fn add_message(&mut self, message: String) {
        if self.messages.len() >= BUFFER_SIZE {
            self.messages.remove(0);
        }
        self.messages.push(message);
    }

    fn display(&self) -> crossterm::Result<()> {
        use crossterm::cursor;

        // Move cursor to appropriate position
        // Adjust the Y coordinate as needed
        let start_line = 15;
        execute!(io::stdout(), cursor::MoveTo(0, start_line))?;

        for message in &self.messages {
            // Clear the line before writing
            execute!(io::stdout(), Clear(ClearType::CurrentLine))?;
            write!(io::stdout(), "{}\n", message)?;
        }

        io::stdout().flush()?;
        Ok(())
    }

    fn clear(&mut self) {
        self.messages.clear();
    }
}


fn main() -> crossterm::Result<()> {
    // Enter the alternate screen
    execute!(io::stdout(), EnterAlternateScreen)?;
    execute!(io::stdout(), crossterm::cursor::Hide)?;

    // Display the game intro
    display_intro()?;

    // Create the player and NPC
    let mut player = create_player();
    let mut console_buffer = ConsoleBuffer::new();
    console_buffer.add_message(format!(
        "Welcome, {}! Prepare for your adventure!",
        player.name
    ));

    let mut npc1 = Npc {
        name: String::from("Wise Elder"),
        quest: Quest {
            description: String::from(
                "Defeat 3 bandits to prove your worth as a martial artist.",
            ),
            reward: 100,
            completed: false,
        },
    };

    // Game loop variables
    let mut game_running = true;
    let mut needs_redraw = true; // Flag to track when player info needs to be redrawn

    // Main game loop
    while game_running {
        // Only redraw the screen if there's a change that requires it
        if needs_redraw {
            clear_screen()?;
            display_player_info(&player)?; // Display updated player stats
            console_buffer.display()?;     // Display messages from the buffer
            needs_redraw = false;          // Reset the redraw flag after drawing
        }

        // Use a selection menu to choose the location or action
        let location_options = ["In the wilds", "At a village"];
        let location_choice = select_option(&location_options)?;

        match location_choice {
            0 => {
                // Explore the wilds
                let outcome = explore_wilds(&mut player, &mut npc1, &mut console_buffer)?;
                console_buffer.add_message(outcome);
                needs_redraw = true; // Set to true because player stats might change during exploration
            }
            1 => {
                // Perform village actions
                let outcome = village_actions(&mut player, &mut game_running, &mut npc1)?;
                console_buffer.add_message(outcome);
                needs_redraw = true; // Set to true because player stats might change in the village
            }
            _ => console_buffer.add_message("Invalid location, please try again.".to_string()),
        }
    }

    // Show the cursor again before exiting
    execute!(io::stdout(), crossterm::cursor::Show)?;
    // Leave the alternate screen
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}


fn display_intro() -> crossterm::Result<()> {
    // ASCII title
    execute!(io::stdout(), cursor::MoveTo(0, 0))?;
    execute!(io::stdout(), SetForegroundColor(Color::Green))?;
    write!(
        io::stdout(),
        r#"
    █▀▄▀█ █░█ █▀█ █ █▀▄▀█
    █░▀░█ █▄█ █▀▄ █ █░▀░█
            "#
    )?;
    execute!(io::stdout(), SetForegroundColor(Color::Reset))?;

    // Game instructions
    write!(
        io::stdout(),
        r#"
Welcome to Cultivation Quest!

In this game, you play as a martial artist or Qi cultivator on a journey to reach immortality.
Here are the main mechanics:

1. **Classes**: Choose from different classes with unique attributes.
2. **Exploration**: Travel through wilds and villages to battle enemies and complete quests.
3. **Qi and Cultivation**: Train your Qi to reach higher cultivation levels and become more powerful.
4. **Breakthroughs**: With enough Qi, attempt breakthroughs to ascend to the next level.
5. **Final Goal**: Reach level 5 and defeat the Ancient Demon Lord. Then, attempt the final breakthrough to level 6 to become an immortal!

**Important**: Each breakthrough has a success chance, which decreases at higher levels. If you fail the final breakthrough to level 6, you will be struck down by the wrath of heaven and earth!

Press Enter to begin your adventure!
            "#
    )?;

    io::stdout().flush()?;

    // Wait for the player to press Enter
    let mut enter = String::new();
    io::stdin()
        .read_line(&mut enter)
        .expect("Failed to read line");
    Ok(())
    
}


fn create_player() -> Player {
    execute!(io::stdout(), crossterm::cursor::Show).unwrap(); // Show cursor for name entry

    print!("Enter your name: ");
    let mut name = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut name).expect("Failed to read line");
    let name = name.trim_end().to_string();

    execute!(io::stdout(), crossterm::cursor::Hide).unwrap(); // Hide cursor after name entry

    // Use `select_option` for class selection
    let class_options = ["Martial Artist", "Qi Cultivator", "Assassin"];
    let class_choice = select_option(&class_options).expect("Failed to select option");

    let class_type = match class_choice {
        0 => ClassType::MartialArtist,
        1 => ClassType::QiCultivator,
        2 => ClassType::Assassin,
        _ => ClassType::MartialArtist,
    };

    let cultivation_level = 1;
    let (base_health, attack, defense, qi) = match class_type {
        ClassType::MartialArtist => (120, 18, 12, 5),
        ClassType::QiCultivator => (80, 8, 6, 25),
        ClassType::Assassin => (100, 15, 10, 10),
    };

    Player {
        name,
        cultivation_level,
        class_type,
        health: base_health,
        attack,
        defense,
        qi,
        gold: 50,
        bandits_defeated: 0,
        cultivation_speed: 1.0,
        qi_pills: 0,
    }
}

fn generate_boss() -> Enemy {
    // Define stats for the final boss, significantly stronger than regular enemies
    Enemy {
        name: String::from("Ancient Demon Lord"),
        health: 500,
        attack: 30,
        defense: 15,
        is_boss: true,
    }
}

fn explore_wilds(
    player: &mut Player,
    npc: &mut Npc,
    console_buffer: &mut ConsoleBuffer,
) -> crossterm::Result<String> {
    let mut rng = rand::thread_rng();

    if player.cultivation_level >= 5 {
        let mut boss = generate_boss();
        console_buffer.add_message(
            "A powerful aura fills the air... The Ancient Demon Lord appears!".to_string(),
        );
        battle(player, &mut boss, console_buffer)?;
        if player.health > 0 {
            console_buffer.add_message(
                "You have defeated the final boss and brought peace to the land!".to_string(),
            );
            std::process::exit(0); // End the game upon defeating the boss
        } else {
            println!("Game Over. You have died.");
            std::process::exit(0);
        }
    } else {
        // Introduce a random chance to meet an NPC or an enemy
        let encounter_chance: u32 = rng.gen_range(1..=100);
        if encounter_chance <= 70 {
            // 70% chance to encounter an enemy
            let mut enemy = generate_enemy(player);
            console_buffer.add_message(format!("A wild {} appears!", enemy.name));

            let encounter_options = ["Fight", "Run"];
            let action_choice = select_option(&encounter_options)?;

            match action_choice {
                0 => {
                    battle(player, &mut enemy, console_buffer)?;
                    if player.health > 0 {
                        console_buffer.add_message(format!(
                            "You defeated the {} and gained 20 gold!",
                            enemy.name
                        ));
                        player.gold += 20;

                        if enemy.name == "Bandit" {
                            player.bandits_defeated += 1;
                            console_buffer.add_message(format!(
                                "You have defeated {} bandits so far.",
                                player.bandits_defeated
                            ));
                        }

                        if rng.gen_bool(0.25) {
                            player.qi_pills += 1;
                            console_buffer.add_message(
                                "You found a Qi pill! Your cultivation speed increases by 1%."
                                    .to_string(),
                            );
                            player.cultivation_speed += 0.05;
                        }
                        Ok("Battle completed.".to_string())
                    } else {
                        println!("Game Over. You have died.");
                        std::process::exit(0);
                    }
                }
                1 => Ok("You chose to run away.".to_string()),
                _ => Ok("Invalid action.".to_string()),
            }
        } else {
            // 30% chance to meet an NPC
            console_buffer.add_message(format!(
                "You encounter {} in the wilds.",
                npc.name
            ));

            let npc_interaction = talk_to_npc(player, npc)?;
            console_buffer.add_message(npc_interaction);

            Ok("You had an encounter in the wilds.".to_string())
        }
    }
}


fn village_actions(
    player: &mut Player,
    game_running: &mut bool,
    npc: &mut Npc,
) -> crossterm::Result<String> {
    clear_screen()?;
    display_player_info(player)?;

    // Use `select_option` for village actions
    let actions = [
        "Rest at a village",
        "Buy techniques",
        "Talk to NPC",
        "Train Qi",
        "Attempt Breakthrough",
        "Quit",
    ];
    let action_choice = select_option(&actions)?;

    match action_choice {
        0 => {
            player.health = 100;
            Ok("You rested and recovered health.".to_string())
        }
        1 => buy_gear(player),
        2 => {
            let npc_interaction = talk_to_npc(player, npc)?;
            Ok(npc_interaction)
        }
        3 => {
            train_qi(player)?;
            Ok("You trained your Qi.".to_string())
        }
        4 => {
            let breakthrough_message = attempt_breakthrough(player)?;
            Ok(breakthrough_message)
        }
        5 => {
            *game_running = false;
            Ok("Thank you for playing! Goodbye.".to_string())
        }
        _ => Ok("Invalid choice.".to_string()),
    }
}


fn train_qi(player: &mut Player) -> crossterm::Result<()> {
    clear_screen()?;
    display_question("Training Qi. Press 'Enter' to stop.")?;

    // Display the meditating figure
    display_meditating_figure()?;

    let mut qi_amount = player.qi;

    loop {
        // Apply the cultivation speed multiplier
        qi_amount += (1.0 * player.cultivation_speed) as i32;

        // Display updated Qi level
        execute!(io::stdout(), cursor::MoveTo(0, 18))?; // Position below the ASCII art
        execute!(io::stdout(), Clear(ClearType::CurrentLine))?;
        write!(io::stdout(), "Qi level: {}", qi_amount)?;
        io::stdout().flush()?;

        // Adjust sleep duration based on cultivation speed
        sleep(Duration::from_millis((100.0 / player.cultivation_speed) as u64));

        // Check for 'Enter' key press to stop training
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.code == KeyCode::Enter {
                    break;
                }
            }
        }
    }

    // Update player's Qi level after training
    player.qi = qi_amount;

    // Display training completion message
    execute!(io::stdout(), cursor::MoveTo(0, 20))?;
    write!(
        io::stdout(),
        "Training stopped. Your Qi level is now {}.",
        qi_amount
    )?;
    io::stdout().flush()?;

    // Remove the extra wait for Enter
    // Return immediately to the main menu
    Ok(())
}

fn attempt_breakthrough(player: &mut Player) -> crossterm::Result<String> {
    let (qi_needed, success_chance) = match player.cultivation_level {
        1 => (100, 0.9),   // 90% chance of success for level 1
        2 => (300, 0.7),   // 70% chance of success for level 2
        3 => (900, 0.5),   // 50% chance of success for level 3
        4 => (2000, 0.4),  // 40% chance of success for level 4
        5 => (5000, 0.3),  // 30% chance of success for level 5
        6 => (10000, 0.1), // 10% chance of success for level 6 (immortality)
        _ => (u32::MAX, 0.0),
    };

    // Clear the area where we will display the info
    execute!(io::stdout(), cursor::MoveTo(0, 10))?;
    execute!(io::stdout(), Clear(ClearType::FromCursorDown))?;

    // Show Qi requirement and success chance to the player
    write!(
        io::stdout(),
        "Attempting to break through to Level {}.\nThis requires {} Qi and has a {:.0}% chance of success.\n",
        player.cultivation_level + 1,
        qi_needed,
        success_chance * 100.0
    )?;
    io::stdout().flush()?;

    // Prompt the user
    write!(io::stdout(), "Do you want to proceed? ")?;
    io::stdout().flush()?;

    // Use the select_yes_no function
    let proceed = select_yes_no()?;

    if proceed {
        if player.qi >= qi_needed as i32 {
            let mut rng = rand::thread_rng();
            if rng.gen_bool(success_chance) {
                // Successful breakthrough
                player.cultivation_level += 1;
                player.qi = 0;
                if player.cultivation_level == 6 {
                    return Ok("Incredible! You have transcended mortal bounds and become a Cultivation Immortal!".to_string());
                } else {
                    return Ok(format!("Congratulations! You have successfully ascended to Cultivation Level {}!", player.cultivation_level));
                }
            } else {
                // Failed breakthrough
                if player.cultivation_level == 5 {
                    return Ok("Your attempt to reach immortality has failed. The heavens tremble... You are struck by the wrath of heaven and earth, your mortal body unable to withstand the fury. You die a hero, but ultimately remain a mortal.".to_string());
                } else {
                    let qi_loss = (player.qi as f32 * 0.3) as i32; // Lose 30% of Qi for levels 1-5
                    player.qi -= qi_loss;
                    return Ok(format!("Breakthrough attempt failed. You lost {} Qi. Try again after further training.", qi_loss));
                }
            }
        } else {
            return Ok("Not enough Qi to attempt a breakthrough.".to_string());
        }
    } else {
        return Ok("You chose not to attempt the breakthrough at this time.".to_string());
    }
}

fn generate_enemy(player: &Player) -> Enemy {
    let mut rng = rand::thread_rng();
    let enemy_type = rng.gen_range(1..=3);

    // Scale enemy stats based on player's cultivation level
    let scale_factor = player.cultivation_level as i32;
    
    match enemy_type {
        1 => Enemy {
            name: String::from("Bandit"),
            health: 40 + (10 * scale_factor),
            attack: 7 + (2 * scale_factor),
            defense: 3 + (scale_factor),
            is_boss: false,
        },
        2 => Enemy {
            name: String::from("Rogue Cultivator"),
            health: 60 + (15 * scale_factor),
            attack: 12 + (3 * scale_factor),
            defense: 5 + (2 * scale_factor),
            is_boss: false,
        },
        3 => Enemy {
            name: String::from("Shadow Assassin"),
            health: 50 + (12 * scale_factor),
            attack: 15 + (3 * scale_factor),
            defense: 4 + (2 * scale_factor),
            is_boss: false,
        },
        _ => unreachable!(),
    }
}

fn battle(
    player: &mut Player,
    enemy: &mut Enemy,
    battle_buffer: &mut ConsoleBuffer,
) -> crossterm::Result<()> {
    let mut rng = rand::thread_rng();

    while player.health > 0 && enemy.health > 0 {
        clear_screen()?;

        // Display player and enemy info, and then buffer messages
        display_player_info(player)?;
        display_enemy_info(enemy)?;
        battle_buffer.display()?;

        let action_options = ["Attack", "Use Qi", "Defend"];
        let action_choice = select_option(&action_options)?;

        match action_choice {
            0 => {
                let damage = player.attack - enemy.defense;
                let action_message = format!(
                    "You attack the enemy for {} damage!",
                    if damage > 0 { damage } else { 1 }
                );
                battle_buffer.add_message(action_message);
                enemy.health -= if damage > 0 { damage } else { 1 };
            }
            1 => {
                if player.qi >= 10 {
                    let damage = ((player.cultivation_level as i32 * 15) - enemy.defense).max(1);
                    let action_message =
                        format!("You unleash a powerful Qi attack for {} damage!", damage);
                    battle_buffer.add_message(action_message);
                    enemy.health -= damage;
                    player.qi -= 10;
                } else {
                    battle_buffer.add_message("Not enough Qi!".to_string());
                }
            }
            2 => {
                battle_buffer.add_message("You brace yourself for the enemy's attack!".to_string());
            }
            _ => {
                battle_buffer.add_message("Invalid action, you lose your turn!".to_string());
            }
        }

        if enemy.health <= 0 {
            if enemy.is_boss {
                write!(
                    io::stdout(),
                    "Congratulations! You have defeated the Ancient Demon Lord!"
                )?;
                io::stdout().flush()?;
                std::process::exit(0);
            }
            player.gold += 20;
            battle_buffer.add_message(format!("You have defeated the {}!", enemy.name));
            break;
        }

        // Enemy's turn
        let enemy_action: u32 = rng.gen_range(1..=2);
        match enemy_action {
            1 => {
                let damage = (enemy.attack - player.defense / 2).max(1);
                battle_buffer.add_message(format!("The enemy attacks you for {} damage!", damage));
                player.health -= damage;
            }
            2 => {
                battle_buffer.add_message("The enemy braces itself!".to_string());
            }
            _ => {}
        }

        if player.health <= 0 {
            write!(
                io::stdout(),
                "You have been defeated by the {}...\nGame Over. You have died.",
                enemy.name
            )?;
            io::stdout().flush()?;
            std::process::exit(0);
        }
    }
    Ok(())
}

fn buy_gear(player: &mut Player) -> crossterm::Result<String> {
    display_player_info(player);

    let gear_options = ["Iron Fist Gloves (30 gold)", "Qi Enhancing Necklace (20 gold)"];
    let gear_choice = select_option(&gear_options)?;

    match gear_choice {
        0 if player.gold >= 30 => {
            player.attack += 5;
            player.gold -= 30;
            Ok("You bought Iron Fist Gloves! Attack increased.".to_string())
        }
        1 if player.gold >= 20 => {
            player.qi += 10;
            player.gold -= 20;
            Ok("You bought a Qi Enhancing Necklace! Qi increased.".to_string())
        }
        _ => Ok("Not enough gold or invalid choice.".to_string()),
    }
}


fn talk_to_npc(player: &mut Player, npc: &mut Npc) -> crossterm::Result<String> {
    if !npc.quest.completed {
        if npc.quest.description == "Defeat 3 bandits to prove your worth as a martial artist."
            && player.bandits_defeated >= 3
        {
            npc.quest.completed = true;
            player.gold += npc.quest.reward;
            Ok(format!(
                "Congratulations! Quest completed. You received {} gold!",
                npc.quest.reward
            ))
        } else {
            Ok(format!("{}: '{}'", npc.name, npc.quest.description))
        }
    } else {
        Ok(format!(
            "{}: You have already completed my quest.",
            npc.name
        ))
    }
}

fn clear_screen() -> crossterm::Result<()> {
    execute!(
        io::stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;
    Ok(())
}

fn display_question(question: &str) -> crossterm::Result<()> {
    use std::time::Duration;
    use crossterm::{style::Color::Yellow, cursor};

    execute!(io::stdout(), SetForegroundColor(Color::Yellow))?;

    // Move to the top of the screen
    execute!(io::stdout(), cursor::MoveTo(0, 0))?;

    // Clear current line and write the question
    execute!(io::stdout(), Clear(ClearType::CurrentLine))?;

    for ch in question.chars() {
        write!(io::stdout(), "{}", ch)?;
        io::stdout().flush()?;
        sleep(Duration::from_millis(10)); // Delay for typewriter effect
    }

    execute!(io::stdout(), SetForegroundColor(Color::Reset))?;
    Ok(())
}

fn display_player_info(player: &Player) -> crossterm::Result<()> {
    use crossterm::{cursor, style::Color::Blue};

    execute!(io::stdout(), SetForegroundColor(Blue))?;

    // Move to the starting position
    execute!(io::stdout(), cursor::MoveTo(0, 0))?;

    // Use write! instead of println!
    write!(io::stdout(), "====================\n")?;
    write!(io::stdout(), "Name: {}\n", player.name)?;
    write!(io::stdout(), "Health: {}\n", player.health)?;
    write!(io::stdout(), "Attack: {}\n", player.attack)?;
    write!(io::stdout(), "Defense: {}\n", player.defense)?;
    write!(io::stdout(), "Qi level: {}\n", player.qi)?;
    write!(
        io::stdout(),
        "Cultivation level: {}\n",
        player.cultivation_level
    )?;
    write!(io::stdout(), "Gold: {}\n", player.gold)?;
    write!(io::stdout(), "====================\n")?;

    execute!(io::stdout(), SetForegroundColor(Color::Reset))?;
    io::stdout().flush()?;
    Ok(())
}

fn display_enemy_info(enemy: &Enemy) -> crossterm::Result<()> {
    use crossterm::style::Color::Red;

    execute!(io::stdout(), SetForegroundColor(Red))?;

    // Move cursor to appropriate position
    // Adjust the Y coordinate as needed
    execute!(io::stdout(), cursor::MoveTo(0, 9))?;

    write!(io::stdout(), "====================\n")?;
    write!(io::stdout(), "Enemy Name: {}\n", enemy.name)?;
    write!(io::stdout(), "Health: {}\n", enemy.health)?;
    write!(io::stdout(), "Attack: {}\n", enemy.attack)?;
    write!(io::stdout(), "Defense: {}\n", enemy.defense)?;
    write!(io::stdout(), "====================\n")?;

    execute!(io::stdout(), SetForegroundColor(Color::Reset))?;
    io::stdout().flush()?;
    Ok(())
}

fn select_option(options: &[&str]) -> crossterm::Result<usize> {
    let mut selected = 0;

    // Initial drawing of the options
    print_options(options, selected)?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Up => {
                        selected = if selected > 0 { selected - 1 } else { options.len() - 1 };
                        print_options(options, selected)?; // Update display
                    }
                    KeyCode::Down => {
                        selected = if selected < options.len() - 1 { selected + 1 } else { 0 };
                        print_options(options, selected)?; // Update display
                    }
                    KeyCode::Enter => return Ok(selected),
                    _ => {}
                }
            }
        }
    }
}

fn print_options(options: &[&str], selected: usize) -> crossterm::Result<()> {
    use crossterm::cursor;

    // Move cursor to a fixed position to overwrite previous options
    // Adjust the Y coordinate as needed
    let options_start_line = 20;
    execute!(io::stdout(), cursor::MoveTo(0, options_start_line))?;

    for (i, option) in options.iter().enumerate() {
        // Clear the line before writing
        execute!(io::stdout(), Clear(ClearType::CurrentLine))?;
        write!(
            io::stdout(),
            "{} {}\n",
            if i == selected { "●" } else { "◯" },
            option
        )?;
    }

    io::stdout().flush()?;
    Ok(())
}

fn select_yes_no() -> crossterm::Result<bool> {
    use crossterm::style::{Attribute, SetAttribute};

    let options = ["Yes", "No"];
    let mut selected = 0;

    loop {
        // Display options
        execute!(io::stdout(), cursor::MoveTo(0, 12))?; // Adjust the position as needed

        // Clear the line
        execute!(io::stdout(), Clear(ClearType::CurrentLine))?;

        for (i, option) in options.iter().enumerate() {
            if i == selected {
                execute!(
                    io::stdout(),
                    SetForegroundColor(Color::Blue),
                    SetAttribute(Attribute::Underlined)
                )?;
                write!(io::stdout(), "{} ", option)?;
                execute!(
                    io::stdout(),
                    SetAttribute(Attribute::Reset),
                    SetForegroundColor(Color::Reset)
                )?;
            } else {
                write!(io::stdout(), "{} ", option)?;
            }
        }
        io::stdout().flush()?;

        // Wait for input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if selected < options.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        // Return true if 'Yes' selected, false if 'No' selected
                        return Ok(selected == 0);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn display_meditating_figure() -> crossterm::Result<()> {
    use crossterm::{cursor, style::Color::Green};

    // Define the ASCII art as a raw string literal
    let art = r#"
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣤⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⣿⣿⢿⣧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣤⣴⣿⣤⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠻⣿⣿⣿⠟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣴⣿⣦⣈⣉⣁⣴⣷⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⣀⣀⡀⠀⠀⢀⣼⣿⣿⣿⣿⣿⣿⣿⣿⣧⠀⠀⠀⣀⣀⣀⠀⠀⠀
⠀⠀⠀⢸⣿⣿⣿⣿⣶⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣶⣿⣿⣿⣿⠂⠀⠀
⠀⠀⠀⠀⠈⠙⠛⠿⣿⣿⡿⠁⢹⣿⣿⣿⣿⣿⡏⠙⢿⣿⣿⠿⠛⠉⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠀⠀⢸⣿⣿⣿⣿⣿⡇⠀⠀⠉⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⣀⣀⣀⣀⣠⣿⣿⣿⣿⣿⣿⣿⣷⣄⣀⣀⣀⡀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⢰⣿⣿⣿⣿⣿⠛⠿⢿⣿⣿⣿⠿⠟⢻⣿⣿⣿⣿⣷⡄⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠈⠻⣿⣿⣿⣿⣷⣶⣤⣄⣁⠀⠐⠶⢿⣿⣿⣿⣿⠟⠁⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠈⠛⠻⢿⣿⣿⣿⣿⣿⣿⣷⣶⣤⣌⡉⠛⠁⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⣶⡦⠄⠀⠉⠉⠉⠙⠛⠻⠿⣿⣷⡀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠁⠀⠀⠀⠀⠀⠀⠀
    "#;

    // Set the color for the ASCII art
    execute!(io::stdout(), SetForegroundColor(Color::Green))?;

    // Move the cursor to a suitable position
    execute!(io::stdout(), cursor::MoveTo(0, 2))?;

    // Print the ASCII art
    write!(io::stdout(), "{}\n", art)?;

    // Reset the color
    execute!(io::stdout(), SetForegroundColor(Color::Reset))?;
    io::stdout().flush()?;
    Ok(())
}

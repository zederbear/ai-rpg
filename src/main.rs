use rand::Rng;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;
use crossterm::{
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
    event::{self, Event, KeyCode},
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
}

#[derive(Debug)]
struct Enemy {
    name: String,
    health: i32,
    attack: i32,
    defense: i32,
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

    fn display(&self) {
        for message in &self.messages {
            println!("{}", message);
        }
    }
}

fn main() -> crossterm::Result<()> {
    let mut player = create_player();
    let mut console_buffer = ConsoleBuffer::new();
    console_buffer.add_message(format!("Welcome, {}! Prepare for your adventure!", player.name));

    let mut npc1 = Npc {
        name: String::from("Wise Elder"),
        quest: Quest {
            description: String::from("Defeat 3 bandits to prove your worth as a martial artist."),
            reward: 100,
            completed: false,
        },
    };

    let mut game_running = true;
    while game_running {
        clear_screen()?;
        console_buffer.display();
        display_question("Where are you? 1) In the wilds 2) At a village");
        let mut location = String::new();
        io::stdin().read_line(&mut location).expect("Failed to read line");
        let location: u32 = match location.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                console_buffer.add_message("Invalid input, please enter a number.".to_string());
                continue;
            }
        };

        match location {
            1 => {
                clear_screen()?;
                let outcome = explore_wilds(&mut player, &mut npc1)?;
                console_buffer.add_message(outcome);
            }
            2 => {
                clear_screen()?;
                let outcome = village_actions(&mut player, &mut game_running)?;
                console_buffer.add_message(outcome);
            }
            _ => console_buffer.add_message("Invalid location, please try again.".to_string()),
        }
    }
    Ok(())
}

fn create_player() -> Player {
    print!("Enter your name: ");
    let mut name = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut name).expect("Failed to read line");
    let name = name.trim_end().to_string();

    display_question("Choose your class: 1) Martial Artist 2) Qi Cultivator 3) Assassin");
    let mut class_choice = String::new();
    io::stdin().read_line(&mut class_choice).expect("Failed to read line");
    let class_choice: u32 = class_choice.trim().parse().expect("Please enter a number");

    let class_type = match class_choice {
        1 => ClassType::MartialArtist,
        2 => ClassType::QiCultivator,
        3 => ClassType::Assassin,
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
    }
}

fn explore_wilds(player: &mut Player, npc: &mut Npc) -> crossterm::Result<String> {
    let mut rng = rand::thread_rng();
    let event = rng.gen_range(1..=3);

    match event {
        1 => {
            let mut enemy = generate_enemy();
            println!("A wild {} appears!", enemy.name);
            battle(player, &mut enemy)?;
            if player.health > 0 {
                Ok(format!("You defeated the {} and gained 20 gold!", enemy.name))
            } else {
                println!("Game Over. You have died.");
                std::process::exit(0);
            }
        }
        2 => talk_to_npc(player, npc),
        3 => {
            player.gold += 10;
            Ok("You found medicinal herbs!".to_string())
        }
        _ => Ok("Nothing happened during your exploration.".to_string()),
    }
}

fn village_actions(player: &mut Player, game_running: &mut bool) -> crossterm::Result<String> {
    display_question("1) Rest at a village 2) Buy techniques 3) Train Qi 4) Attempt Breakthrough 5) Quit");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).expect("Failed to read line");
    let choice: u32 = choice.trim().parse().expect("Please enter a number");

    match choice {
        1 => {
            player.health = 100;
            Ok("You rested and recovered health.".to_string())
        }
        2 => buy_gear(player),
        3 => {
            train_qi(player)?;
            Ok("You trained your Qi.".to_string())
        }
        4 => {
            attempt_breakthrough(player)?;
            Ok("You attempted a breakthrough.".to_string())
        }
        5 => {
            *game_running = false;
            Ok("Thank you for playing! Goodbye.".to_string())
        }
        _ => Ok("Invalid choice.".to_string()),
    }
}

fn train_qi(player: &mut Player) -> crossterm::Result<()> {
    display_question("Training Qi. Press 'Enter' to stop.");
    let mut qi_amount = player.qi;

    loop {
        qi_amount += 1;
        print!("\rQi level: {}", qi_amount);
        io::stdout().flush().unwrap();
        sleep(Duration::from_millis(100));

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.code == KeyCode::Enter {
                    break;
                }
            }
        }
    }
    println!("\nTraining stopped. Your Qi level is now {}.", qi_amount);
    player.qi = qi_amount;
    Ok(())
}

fn attempt_breakthrough(player: &mut Player) -> crossterm::Result<()> {
    let qi_needed = match player.cultivation_level {
        1 => 100,
        2 => 300,
        3 => 900,
        _ => u32::MAX,
    };
    if player.qi >= qi_needed as i32 {
        player.cultivation_level += 1;
        player.qi = 0;
        println!("Congratulations! You have ascended to Cultivation Level {}!", player.cultivation_level);
    } else {
        println!("Not enough Qi to attempt a breakthrough.");
    }
    Ok(())
}

fn generate_enemy() -> Enemy {
    let mut rng = rand::thread_rng();
    let enemy_type = rng.gen_range(1..=3);

    match enemy_type {
        1 => Enemy {
            name: String::from("Bandit"),
            health: 40,
            attack: 7,
            defense: 3,
        },
        2 => Enemy {
            name: String::from("Rogue Cultivator"),
            health: 60,
            attack: 12,
            defense: 5,
        },
        3 => Enemy {
            name: String::from("Shadow Assassin"),
            health: 50,
            attack: 15,
            defense: 4,
        },
        _ => unreachable!(),
    }
}

fn battle(player: &mut Player, enemy: &mut Enemy) -> crossterm::Result<()> {
    let mut rng = rand::thread_rng();

    while player.health > 0 && enemy.health > 0 {
        clear_screen()?;
        println!("Player Health: {} | Enemy Health: {}", player.health, enemy.health);
        display_question("Choose an action: 1) Attack 2) Use Qi 3) Defend");
        let mut action = String::new();
        io::stdin().read_line(&mut action).expect("Failed to read line");
        let action: u32 = action.trim().parse().expect("Please enter a number");

        match action {
            1 => {
                let damage = player.attack - enemy.defense;
                println!("You attack the enemy for {} damage!", if damage > 0 { damage } else { 1 });
                enemy.health -= if damage > 0 { damage } else { 1 };
            }
            2 => {
                if player.qi >= 10 {
                    println!("You unleash a powerful Qi attack!");
                    let damage = (player.cultivation_level as i32 * 20) - enemy.defense;
                    println!("You deal {} damage to the enemy with your Qi!", if damage > 0 { damage } else { 1 });
                    enemy.health -= if damage > 0 { damage } else { 1 };
                    player.qi -= 10;
                } else {
                    println!("Not enough Qi!");
                }
            }
            3 => println!("You brace yourself for the enemy's attack!"),
            _ => println!("Invalid action, you lose your turn!"),
        }

        if enemy.health <= 0 {
            println!("You have defeated the {}!", enemy.name);
            player.gold += 20;
            break;
        }

        let enemy_action: u32 = rng.gen_range(1..=2);
        match enemy_action {
            1 => {
                let damage = enemy.attack - player.defense / 2;
                println!("The enemy attacks you for {} damage!", if damage > 0 { damage } else { 1 });
                player.health -= if damage > 0 { damage } else { 1 };
            }
            2 => println!("The enemy braces itself!"),
            _ => {}
        }

        if player.health <= 0 {
            println!("You have been defeated by the {}...", enemy.name);
            println!("Game Over. You have died.");
            std::process::exit(0);
        }
    }
    Ok(())
}

fn buy_gear(player: &mut Player) -> crossterm::Result<String> {
    display_question("1) Buy Iron Fist Gloves (30 gold) 2) Buy Qi Enhancing Necklace (20 gold)");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).expect("Failed to read line");
    let choice: u32 = choice.trim().parse().expect("Please enter a number");

    match choice {
        1 if player.gold >= 30 => {
            player.attack += 5;
            player.gold -= 30;
            Ok("You bought Iron Fist Gloves! Attack increased.".to_string())
        }
        2 if player.gold >= 20 => {
            player.qi += 10;
            player.gold -= 20;
            Ok("You bought a Qi Enhancing Necklace! Qi increased.".to_string())
        }
        _ => Ok("Not enough gold or invalid choice.".to_string()),
    }
}

fn talk_to_npc(player: &mut Player, npc: &mut Npc) -> crossterm::Result<String> {
    if !npc.quest.completed {
        if npc.quest.description == "Defeat 3 bandits to prove your worth as a martial artist." && player.bandits_defeated >= 3 {
            npc.quest.completed = true;
            player.gold += npc.quest.reward;
            Ok(format!("Congratulations! Quest completed. You received {} gold!", npc.quest.reward))
        } else {
            Ok(format!("{}: '{}'", npc.name, npc.quest.description))
        }
    } else {
        Ok(format!("{}: You have already completed my quest.", npc.name))
    }
}

fn clear_screen() -> crossterm::Result<()> {
    execute!(io::stdout(), Clear(ClearType::All), crossterm::cursor::MoveTo(0, 0))
}

fn display_question(question: &str) {
    execute!(io::stdout(), SetForegroundColor(Color::Yellow), Print(question), SetForegroundColor(Color::Reset)).unwrap();
    println!();
}

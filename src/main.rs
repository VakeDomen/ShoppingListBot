use teloxide::{prelude::*, utils::command::BotCommands};
use std::collections::HashMap;
use std::error::Error;
use std::env;
use once_cell::sync::Lazy;
use std::sync::Mutex;

use dotenv::dotenv;

static SHOPPING_LIST: Lazy<Mutex<HashMap<ChatId, Vec<String>>>> = Lazy::new(|| {
    match serde_any::from_file("shopping_list.json") {
        Ok(hm) => Mutex::new(hm),
        Err(_) => Mutex::new(HashMap::new())
    }
});

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this text.")]
    Help,
    #[command(description = "Add item to shopping list")]
    Add(String),
    #[command(description = "Remove item from shopping list after it's bought: /remove <item number>")]
    Remove(i8),
    #[command(description = "List all items to be bought")]
    List,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("TELOXIDE_TOKEN").expect("$TELOXIDE_TOKEN is not set");
    env::set_var("TELOXIDE_TOKEN", token);
    pretty_env_logger::init();
    let bot = Bot::from_env().auto_send();
    println!("Running telegram bot!");
    teloxide::commands_repl(bot, answer, Command::ty()).await;
}

async fn answer(
    bot: AutoSend<Bot>,
    message: Message,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Help           => { bot.send_message(message.chat.id, Command::descriptions().to_string()).await? },
        Command::Add(item)    => { bot.send_message(message.chat.id, add_to_list(&bot, message, item)).await? },
        Command::Remove(item_index)     => { bot.send_message(message.chat.id, remove_from_list(&bot, message, item_index)).await? },
        Command::List           => { bot.send_message(message.chat.id, display_list(&bot, message)).await? },
    };
    Ok(())
}

fn display_list(
    _: &AutoSend<Bot>,
    message: Message,
)  -> String  {
    let mut subs = SHOPPING_LIST.lock().unwrap();
    match subs.get_mut(&message.chat.id) {
        Some(items) =>  {
            let mut out = "".to_string();
            for (i, x) in items.iter().enumerate() {
                out = format!("{}\n{}\t\t{}", out, i, x);
            }
            out
        },
        None => format!("Not subbed to anything..."),
    }
}

fn remove_from_list(
    _: &AutoSend<Bot>,
    message: Message,
    item_index: i8,
) -> String {
    let mut list = SHOPPING_LIST.lock().unwrap();
    let resp = match list.get_mut(&message.chat.id) {
        Some(v) =>  {
            v.remove(item_index as usize);
            println!("Removed sopping item: {:?}", item_index);
            println!("New state: {:?}", list);
            format!("Successfully removed item from list.")
        },
        None => format!("Not subbed to anything..."),
    };
    match serde_any::to_file("shopping_list.json", &*list) {
        Ok(_) => {();},
        Err(e) => {println!("Error saving subscirbers: {:?}", e);}
    };
    resp
}

fn add_to_list(
    _: &AutoSend<Bot>,
    message: Message,
    item: String,
) -> String {
    let mut list = SHOPPING_LIST.lock().unwrap();
    list.entry(message.chat.id).or_insert(Vec::new());
    let resp = match list.get_mut(&message.chat.id) {
        Some(v) =>  {
            if v.iter().find(|&x| *x == *item) == None {
                v.push(item);
                println!("New item: {:?}", list);
                format!("Successfully added item to shopping list.")
            } else {
                println!("Existing item: {:?}", list);
                format!("You already have the item on the list.")
            }
        },
        None => format!("Something is not right..."),
    };
    match serde_any::to_file("shopping_list.json", &*list) {
        Ok(_) => {();},
        Err(e) => {println!("Error saving shopping list: {:?}", e);}
    };
    resp
}


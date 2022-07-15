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
    Remove(String),
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
        Command::Add(items)    => { bot.send_message(message.chat.id, add_to_list(&bot, message, items)).await? },
        Command::Remove(items)     => { bot.send_message(message.chat.id, remove_from_list(&bot, message, items)).await? },
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
    items: String,
) -> String {
    let mut list = SHOPPING_LIST.lock().unwrap();

    if items.eq("all") {
        match list.get_mut(&message.chat.id) {
            Some(v) => v.clear(),
            None => ()
        }
        return "Removed all items from the shopping list".to_string();
    }
        // parse tasks
    let ids: Vec<usize> = items
        .split_whitespace()
        .map(|id_str| id_str.parse::<usize>())
        .take_while(|x|x.is_ok())
        .map(|x|x.ok().unwrap())
        .collect();
        
    let resp = match list.get_mut(&message.chat.id) {
        Some(v) =>  {
            let mut out = String::new();
            for id in ids {   
                v.remove(id as usize);
                println!("Removed sopping item: {:?}", id);
                out = format!("{}Successfully removed item from list.\n", out);
            }
            out
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
    items: String,
) -> String {
    let mut list = SHOPPING_LIST.lock().unwrap();
    list.entry(message.chat.id).or_insert(Vec::new());    
    let resp = match list.get_mut(&message.chat.id) {
        Some(v) =>  {
            let mut out = String::new();
            for item_slice in items.split(",") {
                let item = item_slice.trim().to_string();
                if v.iter().find(|&x| *x == *item) == None {
                    v.push(item.clone());
                    out = format!("{}Successfully added item({}) to shopping list.\n", out, item);
                } else {
                    out = format!("{}You already have the item on the list.\n", out);
                }
            }
            out
        },
        None => format!("Something is not right..."),
    };
    match serde_any::to_file("shopping_list.json", &*list) {
        Ok(_) => {();},
        Err(e) => {println!("Error saving shopping list: {:?}", e);}
    };
    resp
}


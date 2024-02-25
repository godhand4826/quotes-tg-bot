use clap::{arg, Command};
use dotenvy::dotenv;
use rand::{rngs::ThreadRng, Rng};
use std::sync::{Arc, Mutex};
use std::{env, fs};
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    // parsing arguments...
    let cmd = Command::new("quotes-tg-bot")
        .args(&[
            arg!(-f --quote_file <FILE> "a required file for quotes (one line per quote)"),
            arg!(-t --api_token <TOKEN> "a requited telegram bot api token"),
            arg!(-l --log_level <LEVEL> "log level: trace, debug, info(default), warn, error"),
        ])
        .before_help("A simple telegram bot that replies with a random quote")
        .after_help(
            "Additionally, the program reads environment variables such as QUOTE_FILE, API_TOKEN, LOG_LEVEL, and also supports loading configuration from a .env file."
        );

    let args = cmd.get_matches();
    let arg_quote_file = args.get_one::<String>("quote_file").map(|s| s.to_string());
    let arg_api_token = args.get_one::<String>("api_token").map(|s| s.to_string());
    let arg_log_level = args.get_one::<String>("log_level").map(|s| s.to_string());

    // loading environment variables...
    let _ = dotenv();
    let env_quote_file = env::var("QUOTE_FILE").ok();
    let env_api_token = env::var("API_TOKEN").ok();
    let env_log_level = env::var("LOG_LEVEL").ok();

    // configuring...
    let api_token = arg_api_token
        .or(env_api_token)
        .expect("api token is not set");
    let log_level = arg_log_level
        .or(env_log_level)
        .unwrap_or("info".to_string())
        .to_string();
    let quote_file = arg_quote_file
        .or(env_quote_file)
        .unwrap_or("quotes.txt".to_string());

    // initializing...
    let _ = pretty_env_logger::formatted_builder()
        .default_format()
        .parse_filters(&log_level)
        .init();

    log::info!("loading quotes from file: {}", quote_file);
    let quotes: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(
        fs::read_to_string(quote_file.clone())
            .expect(format!("could not read quote file: {}", quote_file).as_str())
            .lines()
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_owned())
            .collect(),
    ));
    log::info!("{} quotes loaded", quotes.lock().unwrap().len());

    let bot = Bot::new(api_token);
    let handler = Update::filter_message().endpoint(
        |bot: Bot, quotes: Arc<Mutex<Vec<String>>>, msg: Message| async move {
            let reply = match msg.text() {
                Some("/quotes") => Some({
                    let index = ThreadRng::default().gen_range(0..quotes.lock().unwrap().len());
                    quotes.lock().unwrap()[index].to_string()
                }),
                Some("/help") => Some(
                    "/help - show support commands
/quotes - replies a random, hopefully interesting, quote"
                        .to_string(),
                ),
                _ => None,
            };

            if let Some(reply) = reply {
                log::info!(
                    "replying to message with ID {:?} and content: {:?}",
                    msg.id.0,
                    reply
                );
                bot.send_message(msg.chat.id, reply)
                    .reply_to_message_id(msg.id)
                    .await?;
            }

            respond(())
        },
    );

    // start
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![quotes])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

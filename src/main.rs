use regex::Regex;
use reqwest::Client;
use serde_json::Value;
use serenity::client::{ClientBuilder, Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::gateway::{GatewayIntents, Ready};
use std::env;

struct Handler {
    re: Regex,
    client: Client,
}

impl Handler {
    fn new() -> Self {
        Self {
            re: Regex::new(r"https://(?:[^\s/.]+\.)*(spotify\.com|music\.apple\.com|youtube\.com|youtu\.be|tidal\.com|music\.amazon\.[^\s/.]+|pandora\.com|soundcloud\.com|deezer\.com)/\S+").unwrap(),
            client: Client::new(),
        }
    }
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author != **ctx.cache.current_user() {
            if let Some(c) = self.re.captures(&msg.content) {
                // https://www.notion.so/d0ebe08a5e304a55928405eb682f6741
                let res = self
                    .client
                    .get("https://api.song.link/v1-alpha.1/links")
                    .query(&[("url", &c[0])])
                    .send()
                    .await
                    .unwrap();

                if res.status().is_success() {
                    let value = res.json::<Value>().await.unwrap();

                    // Wrap in <> to disable auto-embed
                    let content = format!("<{}>", value["pageUrl"].as_str().unwrap());

                    msg.reply(&ctx.http, content).await.unwrap();
                }
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env if it exists
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").unwrap();
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = ClientBuilder::new(&token, intents)
        .event_handler(Handler::new())
        .await
        .unwrap();

    client.start().await.unwrap();
}

mod basic;
mod clips;
mod data;
mod gpt;
mod helper;
mod mods;
mod reminder;

use dashmap::DashMap;
use data::{UserData, VoiceUser};
use std::{env, sync::Arc};
use tokio::sync::RwLock;

pub use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, data::Data, Error>;

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");
    let token = env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let data = data::Data::load();

    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::GUILD_MESSAGE_REACTIONS
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_VOICE_STATES;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            // Check and create a user account before each command
            pre_command: |ctx: Context<'_>| {
                Box::pin(async move {
                    data::Data::check_or_create_user(ctx).await.unwrap();
                })
            },
            // Save all data after running a command
            post_command: |ctx: Context<'_>| {
                Box::pin(async move {
                    ctx.data().save().await;
                })
            },
            commands: vec![
                register(),
                basic::ping(),
                basic::uwu(),
                basic::wallet(),
                basic::claim_bonus(),
                basic::voice_status(),
                basic::info(),
                basic::leaderboard(),
                basic::buy_tickets(),
                clips::submit_clip(),
                clips::server_clips(),
                clips::my_clips(),
                clips::next_clip(),
                mods::give_creds(),
                mods::take_creds(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|_ctx, _ready, _framework| {
            Box::pin(async move {
                let users = data.users.clone();
                let voice_users = data.voice_users.clone();
                background_task(users, voice_users);
                Ok(data)
            })
        })
        .build();

    let client = serenity::Client::builder(&token, intents)
        .activity(serenity::ActivityData {
            name: "Coding Rust".to_string(),
            kind: serenity::ActivityType::Custom,
            state: Some("Phase2 - Development".to_string()),
            url: None,
        })
        .status(serenity::OnlineStatus::Online)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, data::Data, Error>,
    data: &data::Data,
) -> Result<(), Error> {
    let gen_chat = env::var("GENERAL").expect("Failed to load GENERAL channel id");
    let bot_chat = env::var("BOT_CMD").expect("Failed to load BOT_CMD channel id");
    let sub_chat = env::var("SUBMIT").expect("Failed to load SUBMIT channel id");
    let prof_bid = env::var("PROFESSOR").expect("Failed to load PROFESSOR bot id");

    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}\n\n", data_about_bot.user.name);
        }

        serenity::FullEvent::Message { new_message } => {
            if new_message.author.id.to_string() == prof_bid {
                return Ok(());
            }

            let channel_id = new_message.channel_id.get().to_string();
            if channel_id != gen_chat && channel_id != bot_chat && channel_id != sub_chat {
                return Ok(());
            }

            let mut do_gpt = new_message.mentions_me(&ctx.http).await.unwrap_or(false);
            let mut messages = vec![new_message.content.clone()];
            let mut referenced_message = &new_message.referenced_message;

            // println!("{:?}", messages);

            while let Some(msg) = referenced_message {
                messages.push(msg.content.clone());

                if msg.mentions_me(&ctx.http).await.unwrap_or(false) {
                    do_gpt = true;
                    break;
                }
                referenced_message = &msg.referenced_message;
            }

            let doodle = messages[0].to_lowercase().contains("draw")
                || messages[0].to_lowercase().contains("doodle");
            let randomstyle = messages[0].to_lowercase().contains("style");

            if do_gpt && doodle {
                let doodle_url: String = gpt::generate_doodle(&mut messages, randomstyle).await;
                new_message.reply(&ctx.http, doodle_url).await?;
            }

            if do_gpt && !doodle {
                let reading: String = gpt::generate_text(&mut messages).await;
                new_message.reply(&ctx.http, reading).await?;
            }
        }

        serenity::FullEvent::VoiceStateUpdate { old: _, new } => {
            let voice_users = &data.voice_users;

            // Someone left the channel
            if new.channel_id.is_none() {
                voice_users.remove(&new.user_id);
                return Ok(());
            }

            let mut user = voice_users
                .entry(new.user_id)
                .or_insert(data::VoiceUser::new());
            user.update_mute(new.self_mute || new.mute);
            user.update_deaf(new.self_deaf || new.deaf);
        }
        _ => {}
    }
    Ok(())
}

fn background_task(
    users: Arc<DashMap<serenity::UserId, Arc<RwLock<UserData>>>>,
    voice_users: Arc<DashMap<serenity::UserId, VoiceUser>>,
) {
    tokio::spawn(async move {
        loop {
            {
                // How long should someone be in voice for creds
                const CRED_TIME: i64 = 30;
                // How much creds to award
                const REWARD_CREDITS: i32 = 50;
                // How much xp to award
                const REWARD_XP: i32 = 30;

                // Check time
                let now = chrono::Utc::now();

                for mut x in voice_users.iter_mut() {
                    let (id, vu) = x.pair_mut();
                    let joined = vu.joined;

                    let user_data = users.get_mut(id);
                    if user_data.is_none() {
                        return;
                    }
                    let user_data = user_data.unwrap();

                    if let Some(last) = vu.last_reward {
                        if (now - last).num_minutes() >= CRED_TIME {
                            // Give user credits
                            let mut user_data = user_data.write().await;
                            user_data.add_creds(REWARD_CREDITS);
                            user_data.update_xp(REWARD_XP);
                            vu.last_reward = Some(now);
                        }
                    }

                    if (now - joined).num_minutes() >= CRED_TIME {
                        // Give user credits
                        let mut user_data = user_data.write().await;
                        user_data.add_creds(REWARD_CREDITS);
                        user_data.update_xp(REWARD_XP);
                        vu.last_reward = Some(now);
                    }
                }
            }
            // Sleep for a while before the next iteration
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });
}

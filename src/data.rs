use crate::serenity;
use chrono::prelude::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use serenity::Color;
use std::collections::HashMap;

use std::env;
use std::fs;

use tokio::sync::Mutex;

pub const NUMBER_EMOJS: [&str; 10] = [
    "\u{0030}\u{FE0F}\u{20E3}",
    "\u{0031}\u{FE0F}\u{20E3}",
    "\u{0032}\u{FE0F}\u{20E3}",
    "\u{0033}\u{FE0F}\u{20E3}",
    "\u{0034}\u{FE0F}\u{20E3}",
    "\u{0035}\u{FE0F}\u{20E3}",
    "\u{0036}\u{FE0F}\u{20E3}",
    "\u{0037}\u{FE0F}\u{20E3}",
    "\u{0038}\u{FE0F}\u{20E3}",
    "\u{0039}\u{FE0F}\u{20E3}",
];

// General Structures
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ClipData {
    title: String,
    link: String,
    date: DateTime<Utc>,
    rating: Option<u8>,
}

impl ClipData {
    pub fn new(title: String, link: String) -> Self {
        ClipData {
            title,
            link,
            date: Utc::now(),
            rating: None,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct WishData {
    small_pity: i32,
    big_pity: i32,
    wishes: i32,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ItemData {
    name: String,
    desc: String,
    effect: i32,
    cost: i32,
}

// Event Structures
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PokeData {
    name: String,
    desc: String,
    nickname: Option<String>,
    sprite: String,
    health: Option<i32>,
    types: String,
}

impl PokeData {
    pub fn get_name(&self) -> String {
        return self.name.clone();
    }
    pub fn get_desc(&self) -> String {
        return self.desc.clone();
    }
    // pub fn get_nickname(&self) -> Option<String> {
    //     return self.nickname.clone();
    // }
    pub fn get_sprite(&self) -> String {
        return self.sprite.clone();
    }
    // pub fn get_health(&self) -> Option<i32> {
    //     return self.health.clone();
    // }
    pub fn get_types(&self) -> String {
        return self.types.clone();
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    name: String,
    buddy: i32,
    team: Vec<PokeData>,
    store: Vec<ItemData>,
}

// User profile
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    level: i32,
    xp: i32,

    creds: i32,
    rolls: i32,
    daily_count: i32,
    bonus_count: i32,
    last_daily: DateTime<Utc>,

    pub submits: Vec<Option<ClipData>>,
    wish: WishData,

    event: EventData,
    bag: Vec<ItemData>,
}

impl UserData {
    pub fn update_level(&mut self) {
        self.level = self.level + 1;
    }

    pub fn update_xp(&mut self, xp: i32) -> bool {
        if xp < 0 {
            return false;
        }

        self.xp += xp;
        let xp_cap = self.get_level() * 80;

        if self.xp > xp_cap {
            self.xp -= xp_cap;
            self.update_level();
            self.add_wishes(3);

            return true;
        }
        return false;
    }

    pub fn update_daily(&mut self) {
        self.last_daily = Utc::now();
        self.daily_count += 1;
    }

    pub fn add_rolls(&mut self, roll: i32) -> bool {
        if roll < 1 {
            return false;
        }

        self.rolls += roll;
        return true;
    }

    pub fn check_daily(&self) -> bool {
        let diff = Utc::now() - self.last_daily;
        return diff.num_hours() >= 24;
    }

    pub fn add_bonus(&mut self) {
        if self.bonus_count == 3 {
            self.bonus_count = 3;
        } else {
            self.bonus_count += 1;
        }
    }

    pub fn reset_bonus(&mut self) {
        self.bonus_count = 0;
    }

    pub fn check_claim(&self) -> bool {
        if self.bonus_count == 3 {
            return true;
        } else {
            return false;
        }
    }

    pub fn add_creds(&mut self, creds: i32) -> bool {
        if creds < 0 {
            return false;
        }

        self.creds += creds;
        return true;
    }

    pub fn sub_creds(&mut self, creds: i32) -> bool {
        if creds < 0 {
            return false;
        }
        self.creds -= creds;
        return true;
    }

    pub fn add_wishes(&mut self, wishes: i32) -> bool {
        if wishes < 1 {
            return false;
        }

        self.wish.wishes = wishes;
        return true;
    }

    pub fn get_creds(&self) -> i32 {
        return self.creds;
    }

    pub fn get_luck(&self) -> String {
        if self.daily_count == 0 {
            return "---".to_string();
        }

        let average = self.rolls / self.daily_count;
        let luck: String;
        if average < 6 {
            luck = "Horrible".to_string();
        } else if average >= 6 && average < 9 {
            luck = "Below Average".to_string();
        } else if average >= 9 && average < 12 {
            luck = "Average".to_string();
        } else if average >= 12 && average < 15 {
            luck = "Above Average".to_string();
        } else {
            luck = "Blessed".to_string();
        }

        return luck;
    }

    pub fn get_bonus(&self) -> i32 {
        return self.bonus_count;
    }

    pub fn get_level(&self) -> i32 {
        return self.level;
    }

    pub fn add_submit(&mut self, new_submit: ClipData) -> bool {
        for i in 0..3 {
            let s = self.submits.get_mut(i);
            if let Some(s) = s {
                if s.is_none() {
                    *s = Some(new_submit);
                    return true;
                }
            } else {
                self.submits.push(Some(new_submit));
                return true;
            }
        }
        false
    }

    // pub fn get_submit_index(&self, clip_id: usize) -> Option<usize> {
    //     // cycles through self.submits, get the index
    //     // associated with the clip id
    //     if self.submits.len() <= 0 {
    //         return None;
    //     }
    //     for i in 0..self.submits.len() {
    //         if self.submits[i].id == clip_id {
    //             return Some(i);
    //         }
    //     }
    //     return None;
    // }

    pub fn remove_submit(&mut self, submit_index: usize) -> bool {
        let res = self.submits.remove(submit_index);
        return res.is_some();
    }

    pub fn get_submissions(&self) -> Vec<String> {
        let mut submissions: Vec<String> = vec![];
        for (id, clip) in self.submits.iter().enumerate() {
            if let Some(clip) = clip {
                let clip_string = format!(
                    "{} - {} {}",
                    NUMBER_EMOJS[id],
                    clip.date.date_naive(),
                    clip.title
                );
                submissions.push(clip_string);
            }
        }
        submissions
    }

    // pub fn update_small_pity(&mut self, small_pity: i32) -> bool {
    //     if small_pity < 0 {
    //         return false;
    //     }
    //     self.wish.small_pity = small_pity;
    //     return true;
    // }

    // pub fn update_big_pity(&mut self, big_pity: i32) -> bool {
    //     if big_pity < 0 {
    //         return false;
    //     }
    //     self.wish.big_pity = big_pity;
    //     return true;
    // }

    // pub fn update_wishes(&mut self, wish_count: i32) -> bool {
    //     if wish_count < 0 {
    //         return false;
    //     }
    //     self.wish.wishes = wish_count;
    //     return true;
    // }
}

#[derive(Default, Debug)]
pub struct VoiceUser {
    pub joined: DateTime<Utc>,
    pub mute: Option<DateTime<Utc>>,
    pub deaf: Option<DateTime<Utc>>,
}

impl VoiceUser {
    pub fn new() -> VoiceUser {
        return VoiceUser {
            joined: Utc::now(),
            mute: None,
            deaf: None,
        };
    }
    pub fn update_mute(&mut self, b: bool) {
        if b {
            self.mute = Some(Utc::now());
        } else {
            self.mute = None;
        }
    }
    pub fn update_deaf(&mut self, b: bool) {
        if b {
            self.deaf = Some(Utc::now());
        } else {
            self.deaf = None;
        }
    }
}
/// User data, which is stored and accessible in all command invocations
#[derive(Default)]
pub struct Data {
    /// Persistent data of users
    pub users: Mutex<HashMap<serenity::UserId, UserData>>,
    /// Duration of users in voice channel, updates by events
    pub voice_users: Mutex<HashMap<serenity::UserId, VoiceUser>>,
    pub meme: Vec<String>,
    pub ponder: Vec<String>,
    pub pong: Vec<String>,
    pub d20f: Vec<String>,
    pub gpt_key: String,
    pub pokedex: Vec<PokeData>,
}

impl Data {
    pub async fn check_or_create_user<'a>(
        &self,
        ctx: crate::Context<'a>,
    ) -> Result<(), crate::Error> {
        let user_id = ctx.author().id;
        {
            let mut data = self.users.lock().await;
            if data.contains_key(&user_id) {
                return Ok(());
            }

            data.insert(user_id, Default::default());
        }
        self.save().await;
        ctx.send(
            poise::CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .title("Account Created!")
                    .description(format!("{}", ctx.author().name))
                    .image(
                        "https://gifdb.com/images/high/anime-girl-okay-sign-b5zlye5h8mnjhdg2.gif",
                    )
                    .thumbnail(ctx.author().avatar_url().unwrap())
                    .color(Color::new(16119285)),
            ),
        )
        .await?;

        Ok(())
    }
    /// Attempts to save the data to a file
    ///
    /// Make sure that the Mutex is unlocked before calling this function
    pub async fn save(&self) {
        let users = self.users.lock().await;
        let encoded = serde_json::to_string(&users.clone()).unwrap();
        fs::write("data.json", encoded).expect("Failed to write binary save file");
    }

    /// Attempts to load the Data from a file, otherwise return a default
    pub fn load() -> Data {
        let data = fs::read_to_string("data.json").ok();
        let users: HashMap<serenity::UserId, UserData> = if let Some(file) = data {
            serde_json::from_str(&file).expect("Old data format?")
        } else {
            HashMap::default()
        };

        let meme = read_lines("reference/meme.txt");
        let ponder = read_lines("reference/ponder.txt");
        let pong = read_lines("reference/pong.txt");
        let d20f = read_lines("reference/d20.txt");

        let gpt_key = env::var("API_KEY").expect("missing DISCORD_TOKEN");

        // EVENT DATA ////////////////////////////////////////////////////////////////////////////////////////
        let poke_string = read_lines("event/pokemon.txt");
        let mut pokedex = Vec::new();
        let missing_no = PokeData {
            name: "MissingNo.".to_string(),
            desc: "????????????".to_string(),
            types: "Bird/Normal".to_string(),
            sprite: "https://archives.bulbagarden.net/media/upload/9/98/Missingno_RB.png"
                .to_string(),
            nickname: None,
            health: None,
        };
        pokedex.push(missing_no);

        let mut poke_counter = 1;
        for poke_line in poke_string {
            let line_split: Vec<&str> = poke_line.split("=").collect();

            let name: String = line_split
                .get(0)
                .expect(format!("Failed to load Name for No. {}", poke_counter).as_str())
                .to_string();
            let desc: String = line_split
                .get(1)
                .expect(format!("Failed to load Description for No. {}", poke_counter).as_str())
                .to_string();
            let types: String = line_split
                .get(2)
                .expect(format!("Failed to load typing for No. {}", poke_counter).as_str())
                .to_string();
            let sprite: String = line_split
                .get(3)
                .expect(format!("Failed to load Sprite for No. {}", poke_counter).as_str())
                .to_string();

            let pokemon_info = PokeData {
                name: name,
                desc: desc,
                types: types,
                sprite: sprite,
                nickname: None,
                health: None,
            };

            pokedex.push(pokemon_info);
            poke_counter += 1;
        }
        // EVENT DATA ////////////////////////////////////////////////////////////////////////////////////////

        return Data {
            users: Mutex::new(users),
            voice_users: Mutex::new(HashMap::new()),
            meme,
            ponder,
            pong,
            d20f,
            gpt_key,
            pokedex,
        };
    }
}

fn read_lines(filename: &str) -> Vec<String> {
    let lines: Vec<String> = fs::read_to_string(filename)
        .unwrap()
        .lines()
        .map(String::from)
        .collect();

    println!("{}: loaded {} lines", filename, lines.len());

    return lines;
}

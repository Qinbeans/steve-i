#[macro_use]
extern crate diesel;
extern crate dotenv;

use std::env;
use serenity::async_trait;
use serenity::model::{
    id::GuildId,
    voice::VoiceState, 
    gateway::Ready,
    guild::{
        Guild,
        Member,
        GuildUnavailable
    },
    user::User,
};
use serenity::client::{
    Client,
    Context, 
    EventHandler,
    bridge::gateway::GatewayIntents
};
use serenity::framework::standard::{
    StandardFramework,
    macros::group
};
use dotenv::dotenv;
use songbird::SerenityInit;
use rspotify::{ClientCredsSpotify, Credentials};
//future mutex
use futures::lock::Mutex;
use std::sync::Arc;
use tokio::sync::RwLock;

use std::collections::HashMap;

use db::{init,models,schema};
mod db;

use commands::{
    misc::*,
    music::*
};
mod commands;

use log::{log::{logf,dbgf}, error::errf};
mod log;

mod spotify;

#[group]
#[commands(prefix,steve, deafen, join, leave, mute, play, undeafen, unmute, skip, pause, stop, resume, list, boom, spotify, status)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler{
    async fn ready(&self, ctx: Context, ready: Ready) {
        let guilds = ready.guilds.iter().map(|gp|
                match gp{
                    serenity::model::guild::GuildStatus::Offline(g) => g.id,
                    serenity::model::guild::GuildStatus::OnlineGuild(g) => g.id,
                    serenity::model::guild::GuildStatus::OnlinePartialGuild(g) => g.id,
                    other => panic!("{:?}", other)
                }
            ).collect::<Vec<GuildId>>();
        dbgf(&format!("{} guilds cached", guilds.len()), "Steve");
        let data_map = ctx.data.read().await;
        let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;
        let mut user_cache = data_map.get::<Users>().unwrap().write().await;
        let mut guilduser_cache = data_map.get::<GuUs>().unwrap().write().await;
        let db = data_map.get::<Database>().unwrap().lock().await;

        for user in models::get_users(&db){
            user_cache.insert(user.id,user);
        }
        for guild in models::get_guilds(&db){
            if !guild_cache.contains_key(&guild.id){
                guild_cache.insert(guild.id,guild);
            }
        }
        for guilduser in models::get_guildusers(&db){
            guilduser_cache.push(guilduser);
        }

        for raw_guild in guilds{
            let gid = raw_guild.0 as i64;
            if let Some(guild) = raw_guild.to_guild_cached(&ctx).await{
                let mut owner: serenity::model::user::User;
                let oid = guild.owner_id.0 as i64;
                dbgf(&format!("{}",guild.name), &format!("{}",guild.id));
                for (user_id,member) in guild.members.to_owned(){
                    let uid = user_id.0 as i64;
                    if uid == oid{
                        owner = member.user.clone();
                        dbgf(&format!("{} is the owner", owner.name), &format!("{}",guild.id));
                    }
                    if !user_cache.contains_key(&uid){
                        //insert user
                        user_cache.insert(uid,
                            models::User::new(&db,
                                uid,
                                Some(member.user.to_owned().name),
                                Some(member.user.to_owned().tag()),
                                Some("".to_string()),
                            )
                        );
                    }
                }
                if !guild_cache.contains_key(&gid){
                    //insert guild
                    //each insert saves
                    guild_cache.insert(gid,
                        models::Guild::new(&db,
                            gid,
                            Some(guild.name), 
                            Some("!".to_string()), 
                            Some(oid), 
                            Some("".to_string())
                        )
                    );
                }
                if guilduser_cache.len() == 0{
                    for (user_id,_) in guild.members.to_owned(){
                        let uid = user_id.0 as i64;
                        guilduser_cache.push(models::GuildUser::new(&db,uid,gid));
                    }
                }
            }
        }
        logf(&format!("{} is connected!", ready.user.name), "Steve");
    }
    async fn guild_create(&self, ctx: Context, guild: Guild, _: bool){
        let data_map = ctx.data.read().await;
        let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;
        let mut user_cache = data_map.get::<Users>().unwrap().write().await;
        let mut guilduser_cache = data_map.get::<GuUs>().unwrap().write().await;
        let db = data_map.get::<Database>().unwrap().lock().await;
        let gid = guild.id.0 as i64;
        //add guild to db
        //if guild_cache does not contain guild
        if !guild_cache.contains_key(&gid){
            //find owner id
            let oid_obj = guild.owner_id;
            if let Some(owner) = guild.members.get(&oid_obj){
                let oid = oid_obj.0 as i64;
                //insert owner into cache
                dbgf(&format!("Owner: {}",owner.user.name), &format!("{}",guild.id));
                user_cache.insert(oid,
                    models::User::new(&db,
                        oid,
                        Some(owner.user.to_owned().name),
                        Some(owner.user.to_owned().tag()),
                        Some("".to_string())
                    )
                );
            }
            guild_cache.insert(gid,
                models::Guild::new(&db,
                    gid,Some(guild.name.to_owned()),
                    Some("!".to_string()),
                    Some(guild.owner_id.0 as i64),
                    Some("".to_string())
                )
            );
        }
        dbgf(&format!("Added/Created {}",guild.name), &format!("{}",guild.id));
        for (user, member) in guild.members.to_owned(){
            let uid = user.0 as i64;
            if !user_cache.contains_key(&uid){
                //insert user
                user_cache.insert(uid,
                    models::User::new(&db,
                        uid,
                        Some(member.user.to_owned().name),
                        Some(member.user.to_owned().tag()),
                        Some("".to_string())
                    )
                );
                guilduser_cache.push(models::GuildUser::new(&db,uid,gid));
            }
        }
    }
    async fn guild_delete(&self, ctx: Context, _: GuildUnavailable, guild: Option<Guild>){
        let data_map = ctx.data.read().await;
        let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;
        let mut user_cache = data_map.get::<Users>().unwrap().write().await;
        let guilduser_cache = data_map.get::<GuUs>().unwrap().write().await;
        let db = data_map.get::<Database>().unwrap().lock().await;
        if let Some(g) = guild{
            let gid = g.id.0 as i64;
            logf(&format!("Deleted {}",g.name), &format!("{}",g.id));
            for guilduser in guilduser_cache.iter(){
                if guilduser.guild_id == gid{
                    let uid = guilduser.user_id;
                    let user = user_cache.get(&uid).unwrap();
                    user.remove(&db);
                    user_cache.remove(&uid);
                }
            }
            // delete guild from cache
            logf(&format!("Not a guild: {}",g.name), &format!("{}",g.id));
            let guild = guild_cache.get(&gid).unwrap();
            guild.remove(&db);
            guild_cache.remove(&gid);
        }
    }
    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, new_member: Member) {
        let data_map = ctx.data.read().await;
        let mut user_cache = data_map.get::<Users>().unwrap().write().await;
        let mut guilduser_cache = data_map.get::<GuUs>().unwrap().write().await;
        let db = data_map.get::<Database>().unwrap().lock().await;
        let gid = guild_id.0 as i64;
        let uid = new_member.user.id.0 as i64;
        //insert user
        user_cache.insert(uid,
            models::User::new(&db,
                uid,
                Some(new_member.user.to_owned().name),
                Some(new_member.user.to_owned().tag()),
                Some("".to_string())
            )
        );
        //insert guilduser
        guilduser_cache.push(models::GuildUser::new(&db,uid,gid));
    }
    async fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, _: Option<Member>) {
        let data_map = ctx.data.read().await;
        let mut user_cache = data_map.get::<Users>().unwrap().write().await;
        let mut guilduser_cache = data_map.get::<GuUs>().unwrap().write().await;
        let db = data_map.get::<Database>().unwrap().lock().await;
        let gid = guild_id.0 as i64;
        let uid = user.id.0 as i64;
        //remove guil(duser
        for i in 0..guilduser_cache.len(){
            if guilduser_cache[i].guild_id == gid && guilduser_cache[i].user_id == uid{
                guilduser_cache.remove(i);
                if let Some(user_db) = user_cache.get(&uid){
                    user_db.remove(&db);
                }
                user_cache.remove(&uid);
            }
        }
    }
    /**
     * Makes sure the bot exits when everyone else exits
     */
    async fn voice_state_update(&self, ctx: Context, _: Option<GuildId>, old: Option<VoiceState>, _: VoiceState){
        let data_map = ctx.data.read().await;
        let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;
        let current_user = ctx.cache.current_user().await;
        let mut guild_name = "None";
        let current_id = current_user.id.0;
        if let Some(o_channel) = old{
            if let Some(o_channel_id) = o_channel.channel_id{
                if let Some(guild_id) = o_channel.guild_id{
                    let i_guild_id = guild_id.0 as i64;
                    let c_guild_raw = guild_cache.get_mut(&i_guild_id);
                    if let Some(c_guild) = c_guild_raw{
                        if let Some(c_guild_name) = c_guild.name.as_ref(){
                            guild_name = &c_guild_name;
                        }
                    }
                    // get guild name
                    let m_g_channels = guild_id.channels(&ctx.http).await;
                    let manager = songbird::get(&ctx).await
                        .expect("Songbird Voice client placed in at initialisation.").clone();
                    if let Err(e) = m_g_channels{
                        errf(&format!("{}",e),guild_name);
                    }else if let Ok(r_g_channels) = m_g_channels{
                        if let Some(raw_members) = r_g_channels.get(&o_channel_id){
                            //passes
                            let members = raw_members.members(&ctx.cache).await;
                            if let Err(e) = members{
                                errf(&format!("{}",e), guild_name);
                            }else if let Ok(r_members) = members{
                                let mem_count = r_members.len();
                                for mem in r_members{
                                    if mem.user.id.0 == current_id{
                                        if mem_count <= 1{
                                            let has_handler = manager.get(guild_id).is_some();
                                            if has_handler{
                                                if let Err(e) = manager.remove(guild_id).await{
                                                    errf(&format!("{}",e), guild_name);
                                                    return;
                                                }
                                                logf(&format!("'{}' has empty chat '{}'",guild_id.name(&ctx.cache).await.unwrap(),raw_members.name), guild_name);
                                            }else{
                                                logf(&format!("No in guild '{}'",guild_id.name(&ctx.cache).await.unwrap()), guild_name);
                                            }
                                        }
                                        return;
                                    }
                                }
                            }
                        }else{
                            return;
                        }
                    }
                }else{
                    errf("Channel not in guild", guild_name);
                }
            }else{
                errf("No channel id found", guild_name);
            }
        }else{
            errf("No old channel found", guild_name);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let framework = StandardFramework::new()
        .configure(|c| c
            .dynamic_prefix(|ctx, msg| Box::pin(async move{
                let guild_id = msg.guild_id.unwrap().0 as i64;
                let def_pfx = env::var("PREFIX").expect("Steve has no prefix!");
                let data_map = ctx.data.read().await;
                let guild_cache = data_map.get::<Guilds>().unwrap().write().await;
                let guild_raw = guild_cache.get(&guild_id);
                let mut guild_name = "None";
                if let Some(guild) = guild_raw{
                    if let Some(tmp) = guild.name.as_ref(){
                        guild_name = tmp;
                    }
                    if let Some(prefix) = guild.prefix.as_ref(){
                        logf(&format!("prefix {} changed!", prefix), guild_name);
                    }else{
                        logf(&format!("prefix {}", def_pfx), guild_name);
                    }
                }else{
                    errf("No guild found", &format!("{}",guild_name));
                }
                if guild_cache.contains_key(&guild_id){
                    let guild = guild_cache.get(&guild_id).unwrap();
                    let g_pfx = guild.prefix.as_ref().unwrap();//circumvent move
                    if guild.prefix.is_some(){
                        logf(&format!("prefix is {}...", g_pfx), guild_name);
                        return Some(g_pfx.to_string());
                    }
                }
                Some(def_pfx)
            }))
        )
        .unrecognised_command(unknown_command)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);
    let token = env::var("DISCORD_TOKEN").expect("Steve is confused...What token?");
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .intents(GatewayIntents::all())
        .register_songbird()
        .await
        .expect("Steve is lost...Steve flatlined");
    {
        let mut data = client.data.write().await;
        let conn = init::establish_connection();
        logf("Connecting to Spotify...", "Spotify");
        let creds_raw = Credentials::from_env();
        if let Some(creds) = creds_raw{
            let mut s_client = ClientCredsSpotify::new(creds);
            let res = s_client.request_token().await;
            if let Ok(_) = res{
                logf("Connected to Spotify!", "Spotify");
                data.insert::<Database>(Arc::new(Mutex::new(conn)));
                data.insert::<Users>(Arc::new(RwLock::new(HashMap::default())));
                data.insert::<Guilds>(Arc::new(RwLock::new(HashMap::default())));
                data.insert::<GuUs>(Arc::new(RwLock::new(Vec::new())));
                data.insert::<SpotifyClient>(Arc::new(Mutex::new(s_client)));
                data.insert::<Query>(Arc::new(RwLock::new(HashMap::default())));
            }else{
                errf("No token received!", "Spotify");
                return;
            }
        }else{
            errf("No credentials found!", "Spotify");
            return;
        }
    }
    if let Err(why) = client.start_shards(5).await {
        errf(&format!("Steve is hurt!! Call an ambulance stat!! He has {:?}!!",why), "Steve");
    }
}
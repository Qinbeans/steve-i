use serenity::model::{
   prelude::UserId,
   channel::Message
};
use serenity::utils::Colour;
use serenity::framework::standard::{
   CommandResult,
   Args,
   macros::{
       help,
       hook,
       command
   },
   CommandGroup,
   HelpOptions,
   help_commands
};
use diesel::mysql::MysqlConnection;
use serenity::client::Context;
use serenity::prelude::*;
use std::collections::{
    HashMap,
    HashSet
};
//future mutex
use futures::lock::Mutex;
use std::sync::Arc;
use tokio::sync::RwLock;
use rspotify::ClientCredsSpotify;
use crate::db::models;

pub struct Database;

impl TypeMapKey for Database {
    type Value = Arc<Mutex<MysqlConnection>>;
}

pub struct Users;

impl TypeMapKey for Users {
    type Value = Arc<RwLock<HashMap<i64,models::User>>>;
}

pub struct Guilds;

impl TypeMapKey for Guilds {
    type Value = Arc<RwLock<HashMap<i64,models::Guild>>>;
}

pub struct GuUs;

impl TypeMapKey for GuUs {
    type Value = Arc<RwLock<Vec<models::GuildUser>>>;
}

pub struct SpotifyClient;

impl TypeMapKey for SpotifyClient {
    type Value = Arc<Mutex<ClientCredsSpotify>>;
}

const VALID_PFX: [&'static str; 17] = ["~","+","-","=","$","#","@","/","\\","?",">","*","&","^","%","|","!"];

fn find(tar:String, src:Vec<&str>) -> bool{
    if src.iter().any(|&x| x == tar){
        return true;
    }
    false
}

#[help]
#[embed_success_colour("#ff00ea")]
#[embed_error_colour("#ff0000")]
#[individual_command_tip = "Try `<prefix><command>`"]
#[command_not_found_text = "Whats is `{}`?"]
#[max_levenshtein_distance(3)]
pub async fn my_help(ctx: &Context, msg: &Message, args: Args, help_options: &'static HelpOptions, groups: &[&'static CommandGroup], owners: HashSet<UserId>) -> CommandResult {
    let _ = help_commands::with_embeds(ctx,msg,args,help_options,groups,owners).await;
    Ok(())
}
#[hook]
pub async fn unknown_command(ctx: &Context, msg: &Message, unknown_command_name: &str){
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(format!("{}, what are you trying to say?",unknown_command_name));
            e.color(Colour::RED)
        })
    }).await.unwrap();
}

#[command]
#[description("Am I alive?")]
#[usage("steve")]
#[only_in("guilds")]
pub async fn steve(ctx: &Context, msg: &Message) -> CommandResult{
    msg.channel_id.say(&ctx.http, "STEEEEEVVVVVVEEEEE").await?;
    Ok(())
}

#[command]
#[description("Change prefix")]
#[usage("prefix <new prefix|blank>")]
#[only_in("guilds")]
#[owner_privilege(true)]
pub async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult{
    let data_map = ctx.data.read().await;
    let db = data_map.get::<Database>().unwrap().lock().await;
    let pfx = args.single::<String>().unwrap_or(format!("+"));
    if !find(pfx.to_owned(),VALID_PFX.to_vec()){
        msg.reply(ctx, "Invalid prefix").await?;
        return Ok(());
    }
    //this is more so to prevent that panick message
    if msg.guild_id.is_none(){
        msg.reply(ctx, "This command can only be used in a guild").await?;
        return Ok(());
    }
    let raw_gid = msg.guild_id;
    if raw_gid.is_none(){
        msg.reply(ctx, "This command can only be used in a guild").await?;
        return Ok(());
    }
    let gid = raw_gid.unwrap().0 as i64;
    let raw_guild = models::get_guild_by_id(&db, gid);
    if raw_guild.is_none(){
        msg.reply(ctx, "This command can only be used in a guild").await?;
        return Ok(());
    }
    let guild = raw_guild.unwrap();
    guild.set_prefix(&db,pfx.to_owned());
    msg.channel_id.send_message(&ctx.http,|m| {
        m.embed(|e| {
            e.title("Prefix Changed");
            e.description(format!("Prefix changed to `{}`",pfx));
            e.color(Colour::BLITZ_BLUE)
        })
    }).await?;
    Ok(())
}
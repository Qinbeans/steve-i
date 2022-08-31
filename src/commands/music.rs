// #[macro]
use serenity::builder::{
    CreateActionRow,
    CreateButton,
    CreateComponents
};
use core::time::Duration;
use serenity::model::{
    channel::Message,
    application::component::ButtonStyle
};
use serenity::client::Context;
use serenity::framework::standard::{
    CommandResult,
    Args,
    macros::command
};
use serenity::{
    Result as SerenityResult
};
use serenity::utils::Colour;
use songbird::input::restartable::Restartable;
use youtube_dl::{YoutubeDlOutput,YoutubeDl,SearchOptions};
use rspotify::ClientCredsSpotify;
use crate::log::{log::*, error::errf};
use crate::commands::misc::{
    Guilds,
    SpotifyClient,
    Database
};
use crate::spotify::{
    result::{
        QueryResult,
        QueryType,
        get_playlist,
        get_song
    },
};

const MAX_QUERY: usize = 5;
const MAX_LABEL_LENGTH: usize = 17;

//##############################START##################################
// Grabbed from:
//https://github.com/serenity-rs/songbird/blob/current/examples/serenity/voice/src/main.rs

#[command]
#[description("I won't hear anything.")]
#[usage("deafen")]
#[only_in("guilds")]
pub async fn deafen(ctx: &Context, msg: &Message) -> CommandResult {
    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let mut guild_name = "None";
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        if let Some(g_name) = c_guild.name.as_ref() {
            guild_name = g_name;
        }
    }

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            errf("Not in a voice channel", guild_name);
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        },
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_deaf() {
        errf("Already deafened", guild_name);
        check_msg(msg.channel_id.say(&ctx.http, "Already deafened").await);
    } else {
        if let Err(e) = handler.deafen(true).await {
            errf(&format!("Failed: {}",e), guild_name);
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }
        logf("Deafened", guild_name);
        check_msg(msg.channel_id.say(&ctx.http, "Deafened").await);
    }

    Ok(())
}

#[command]
#[description("Allows me to join.")]
#[usage("join")]
#[only_in("guilds")]
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let mut guild_name = "None";
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        if let Some(g_name) = c_guild.name.as_ref() {
            guild_name = g_name;
        }
    }
    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            errf("Not in a voice channel", guild_name);
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let _handler = manager.join(guild_id, connect_to).await;

    Ok(())
}

#[command]
#[description("Tells me to leave")]
#[usage("leave")]
#[only_in("guilds")]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let mut guild_name = "None";
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        if let Some(g_name) = c_guild.name.as_ref() {
            guild_name = g_name;
        }
    }

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            errf(&format!("Failed: {}",e), guild_name);
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }
        logf("Left", guild_name);
        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        errf("Not in a voice channel", guild_name);
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[description("Mutes me for whatever reason")]
#[usage("mute")]
#[only_in("guilds")]
pub async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let mut guild_name = "None";
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        if let Some(g_name) = c_guild.name.as_ref() {
            guild_name = g_name;
        }
    }

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            errf("Not in a voice channel", guild_name);
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        },
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        errf("Already muted", guild_name);
        check_msg(msg.channel_id.say(&ctx.http, "Already muted").await);
    } else {
        if let Err(e) = handler.mute(true).await {
            errf(&format!("Failed: {}",e), guild_name);
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        logf("Muted", guild_name);
        check_msg(msg.channel_id.say(&ctx.http, "Now muted").await);
    }

    Ok(())
}

fn create_button(name:String, id: usize) -> CreateButton {
    //resize the text to fit the button
    let mut button: CreateButton = CreateButton::default();
    button.label(name)
        .style(ButtonStyle::Success)
        .custom_id(id);
    button
}

fn button_builder(names: Vec<String>) -> CreateActionRow {
    let mut row: CreateActionRow = CreateActionRow::default();
    for (i,name) in names.iter().enumerate() {
        logf(&format!("{}:{}/{}",i,name,name.len()), "NONE");
        let resize = format!("{}...",special_truncate(name));
        row.add_button(create_button(resize,i));
    }
    row
}

fn special_truncate(name: &str) -> String {
    match name.char_indices().nth(MAX_LABEL_LENGTH) {
        None => name.to_string(),
        Some((idx, _)) => format!("{}...",&name[..idx]),
    }
}

#[command]
#[description("I'll play some songs for you")]
#[usage("p(lay) <song url|playlist url>")]
#[aliases("p")]
#[only_in("guilds")]
pub async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut guild_name = "None".to_string();
    let url = args.rest().to_string();
    let main_msg_res = msg.channel_id.say(&ctx.http, "Loading...").await;
    if main_msg_res.as_ref().err().is_some() {
        errf("Can't reach", &guild_name);
        return Ok(());
    }
    let mut main_msg = main_msg_res.ok().unwrap();
    if url.len() == 0 {
        errf("No query provided", &guild_name);
        //edit main_msg
        let _ = main_msg.edit(&ctx.http, |m| {
            m.content("").embed(|e| {
                e.title("Error")
                    .description("No query provided")
                    .color(Colour::RED)
            })
        }).await;
        return Ok(());
    }

    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;
    let db = &mut data_map.get::<Database>().unwrap().lock().await;
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        //copy name of guild
        guild_name = c_guild.get_name().to_string();
        c_guild.empty_query_results().update(db);
    }

    let mut searchable = false;
    if !url.starts_with("http") {
        searchable = true;
    }
    
    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);
    
    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            errf("Not in a voice channel", &guild_name);
            let _ = main_msg.edit(&ctx.http, |m| {
                m.content("").embed(|e| {
                    e.title("Error")
                        .description("Not in a voice channel")
                        .color(Colour::RED)
                })
            }).await;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let _handler = manager.join(guild_id, connect_to).await;

    let mut err: String;
    let handler_lock_raw = manager.get(guild_id);
    if handler_lock_raw.is_none() {
        errf("Not in a voice channel to play in", &guild_name);
        let _ = main_msg.edit(&ctx.http, |m| {
            m.content("").embed(|e| {
                e.title("Error")
                    .description("Not in a voice channel to play in")
                    .color(Colour::RED)
            })
        }).await;
        return Ok(());
    }
    let handler_lock = handler_lock_raw.unwrap();
    let mut handler = handler_lock.lock().await;
    if searchable{
        //search option
        let search = SearchOptions::youtube(&url).with_count(MAX_QUERY);
        let output = YoutubeDl::search_for(&search).run();
        if output.as_ref().err().is_some() { // need to borrow output
            err = format!("Failed getting search");
            errf(&err, &guild_name);
        }else{
            let results_raw = output.ok().unwrap();
            //get search results
            let results: (Option<Vec<String>>,Option<Vec<String>>) = match results_raw {
                YoutubeDlOutput::Playlist(pl_ref) => {
                    //deref pl
                    let pl = *pl_ref;
                    if pl.entries.is_none(){
                        ()
                    }
                    let items = pl.entries.unwrap();
                    //vector of songs need to become vector of strings
                    let mut urls: Vec<String> = Vec::new();
                    let mut values: Vec<String> = Vec::new();
                    for item in items {
                        values.push(item.title.to_owned());
                        let url_raw = item.webpage_url;
                        if url_raw.is_none() {
                            err = format!("No url found for: {}",item.title);
                            errf(&err, &guild_name);
                            break;
                        }
                        //copy url from String to &str
                        urls.push(url_raw.unwrap());
                    }
                    (Some(urls),Some(values))
                }
                _ => (None, None)
            };
            let mut joined: String = "".to_string();
            if results.1.is_none() {
                err = format!("No results found for: {}",&url);
                errf(&err, &guild_name);
                return Ok(());
            }
            let q = results.1.unwrap();
            for (i,url) in q.iter().enumerate() {
                joined = format!("{}{}: {}\n",joined,i+1,url);
            }
            let dbg = format!("DEBUG\n{}",joined);
            dbgf(&dbg, &guild_name);
            //embed
            let _ = main_msg.edit(&ctx.http, |m| {
                m.content("").embed(|e| {
                    e.title("Search results");
                    e.description(joined);
                    e.color(Colour::BLITZ_BLUE)
                }).components(|c| {
                    c.add_action_row(button_builder(q))
                })
            }).await;
            let mci = match main_msg.await_component_interaction(ctx.shard.as_ref()).timeout(Duration::from_secs(15)).collect_limit(1).await {
                Some(mci) => mci,
                None => {
                    err = format!("No interaction");
                    errf(&err, &guild_name);
                    let _ = main_msg.edit(&ctx.http, |m| {
                        m.content("").embed(|e| {
                            e.title("Timeout")
                                .description("Took too long")
                                .color(Colour::ORANGE)
                        }).set_components(CreateComponents::default())
                    }).await;
                    return Ok(());
                }
            };
            let res = &mci.data.custom_id.parse::<usize>();
            if res.is_err() {
                err = format!("Failed parsing custom id");
                errf(&err, &guild_name);
                let _ = main_msg.edit(&ctx.http, |m| {
                    m.content("").embed(|e| {
                        e.title("Error")
                            .description("Failed parsing custom id")
                            .color(Colour::RED)
                    }).set_components(CreateComponents::default())
                }).await;
                return Ok(());
            }
            if results.0.is_none() {
                err = format!("No queries found");
                errf(&err, &guild_name);
                let _ = main_msg.edit(&ctx.http, |m| {
                    m.content("").embed(|e| {
                        e.title("Error")
                            .description("No queries found")
                            .color(Colour::RED)
                    }).set_components(CreateComponents::default())
                }).await;
                return Ok(());
            }
            let urls = results.0.unwrap();
            if *res.as_ref().ok().unwrap() == urls.len() {
                let _ = main_msg.delete(ctx).await;
                return Ok(());
            }
            let url = urls[*res.as_ref().ok().unwrap()].to_owned();
            let source = match Restartable::ytdl(url, true).await {
                Ok(source) => source,
                Err(e) => {
                    errf(&format!("Err invlaid link {:?}", e), &guild_name);
                    let _ = main_msg.edit(&ctx.http, |m| {
                        m.content("").embed(|e| {
                            e.title("Error")
                                .description("Error sourcing ffmpeg")
                                .color(Colour::RED)
                        }).set_components(CreateComponents::default())
                    }).await;
                    return Ok(());
                },
            };
            handler.enqueue_source(source.into());
        }
    }else{
        if url.contains("youtube") && url.contains("playlist"){
            let output = YoutubeDl::new(&url)
                        .flat_playlist(true)
                        .socket_timeout("5")
                        .run();
            if let Ok(yt) = output {
                match yt {
                    YoutubeDlOutput::Playlist(yt_pl) => {
                        let entries = yt_pl.entries.unwrap_or(vec![]);
                        for entry in entries {
                            let source = match Restartable::ytdl(entry.url.unwrap(), true).await {
                                Ok(source) => source,
                                Err(e) => {
                                    errf(&format!("Err not searchable {:?}", e), &guild_name);

                                    let _ = main_msg.edit(&ctx.http, |m| {
                                        m.embed(|e| {
                                            e.title("Error")
                                                .description("Errer not searchable")
                                                .color(Colour::RED)
                                        })
                                    }).await;
                                    return Ok(());
                                },
                            };
                            handler.enqueue_source(source.into());
                        }                            
                    }
                    YoutubeDlOutput::SingleVideo(yt_vid) => {
                        errf(&format!("Err unreasonable dead end with {:?}",yt_vid.url.unwrap()), &guild_name);
                        let _ = main_msg.edit(&ctx.http, |m| {
                            m.embed(|e| {
                                e.title("Error")
                                    .description("Uh oh dead en")
                                    .color(Colour::RED)
                            })
                        }).await;
                    }
                }
            }
        }else{
            let source = match Restartable::ytdl(url, true).await {
                Ok(source) => source,
                Err(e) => {
                    errf(&format!("Err invlaid link {:?}", e), &guild_name);
                    let _ = main_msg.edit(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title("Error")
                                .description("Errer sourcing ffmpeg")
                                .color(Colour::RED)
                        }).set_components(CreateComponents::default())
                    }).await;
                    return Ok(());
                },
            };
            handler.enqueue_source(source.into());
        }
    }
    if !handler.queue().is_empty(){
        logf("Queued", &guild_name);
        let _ = main_msg.edit(&ctx.http, |m| {
            m.content("").embed(|e| {
                e.title("Queued")
                    .color(Colour::BLITZ_BLUE)
            }).set_components(CreateComponents::default())
        }).await;
        return Ok(());
    }
    if let Err(e) = handler.queue().resume(){
        errf(&format!("Failed {:?}", e), &guild_name);
        let _ = main_msg.edit(&ctx.http, |m| {
            m.content("").embed(|e| {
                e.title("Error")
                    .description("Failed to resume")
                    .color(Colour::RED)
            }).set_components(CreateComponents::default())
        }).await;
    }else if !searchable{
        logf("Playing...", &guild_name);
        let _ = main_msg.edit(&ctx.http, |m| {
            m.content("").embed(|e| {
                e.title("Playing...")
                    .color(Colour::BLITZ_BLUE)
            }).set_components(CreateComponents::default())
        }).await;
    }
    Ok(())
}

#[command]
#[description("I will be able to hear again")]
#[usage("undeafen")]
#[only_in("guilds")]
pub async fn undeafen(ctx: &Context, msg: &Message) -> CommandResult {
    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let mut guild_name = "None";
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        if let Some(g_name) = c_guild.name.as_ref() {
            guild_name = g_name;
        }
    }

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.deafen(false).await {
            errf(&format!("Failed {:?}", e), guild_name);
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }
        logf("Undeafened", guild_name);
        check_msg(msg.channel_id.say(&ctx.http, "Undeafened").await);
    } else {
        errf("Not in a voice channel to play in", guild_name);
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to undeafen in").await);
    }

    Ok(())
}

#[command]
#[description("I will continue to sing my heart out")]
#[usage("unmute")]
#[only_in("guilds")]
pub async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let mut guild_name = "None";
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        if let Some(g_name) = c_guild.name.as_ref() {
            guild_name = g_name;
        }
    }
    
    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.mute(false).await {
            errf(&format!("Failed {:?}", e), guild_name);
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }
        logf("Unmuted", guild_name);
        check_msg(msg.channel_id.say(&ctx.http, "Unmuted").await);
    } else {
        errf("Not in a voice channel to play in", guild_name);
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to unmute in").await);
    }

    Ok(())
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        logf(&format!("Checking message status {:?}",why), "None");
    }
}
//###############################END###################################


#[command]
#[description("Stops music")]
#[usage("stop")]
#[only_in("guilds")]
pub async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let mut guild_name = "None";
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        if let Some(g_name) = c_guild.name.as_ref() {
            guild_name = g_name;
        }
    }
    
    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        logf("Stopping", guild_name);
        check_msg(msg.channel_id.send_message(&ctx.http,|m|{
            m.tts(false);
            m.embed(|e| {
                e.title("Steve is quiet");
                e.description("Steve shut up, but here was last song:");
                e.image((*handler.queue().current().unwrap().metadata()).thumbnail.as_ref().unwrap());
                e.color(Colour::TEAL);
                e
            });

            m
        }).await);
        handler.stop();
    }else{
        errf("Not in a voice channel to play in", guild_name);
        check_msg(msg.channel_id.send_message(&ctx.http,|m|{
            m.tts(false);
            m.embed(|e| {
                e.title("Banana");
                e.description("Steve not in channel");
                e.color(Colour::RED);
                e
            });

            m
        }).await);
    }
    Ok(())
}

#[command]
#[description("Pauses music")]
#[usage("pause")]
#[only_in("guilds")]
pub async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let mut guild_name = "None";
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        if let Some(g_name) = c_guild.name.as_ref() {
            guild_name = g_name;
        }
    }
    
    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        if handler.queue().is_empty(){
            errf("Nothing to pause", guild_name);
            check_msg(msg.channel_id.send_message(&ctx.http,|m|{
                m.tts(false);
                m.embed(|e| {
                    e.title("Uh oh :)");
                    e.description("Queue is empty");
                    e.color(Colour::RED);
                    e
                });

                m
            }).await);
        }else{
            if let Err(e) = handler.queue().pause(){
                errf(&format!("Failed {:?}", e), guild_name);
                check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
            }
            else{
                logf("Paused", guild_name);
                check_msg(msg.channel_id.send_message(&ctx.http,|m|{
                    m.tts(false);
                    m.embed(|e| {
                        e.title("Steve pause :)");
                        e.description("Monkey pausing:");
                        e.image((*handler.queue().current().unwrap().metadata()).thumbnail.as_ref().unwrap());
                        e.color(Colour::TEAL);
                        e
                    });

                    m
                }).await);
            }
        }
    }else{
        errf("Not in a voice channel to play in", guild_name);
        check_msg(msg.channel_id.send_message(&ctx.http,|m|{
            m.tts(false);
            m.embed(|e| {
                e.title("Banana");
                e.description("Steve not in channel");
                e.color(Colour::RED);
                e
            });

            m
        }).await);
    }
    Ok(())
}

#[command]
#[description("Skips current song")]
#[usage("skip")]
#[only_in("guilds")]
pub async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let mut guild_name = "None";
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        if let Some(g_name) = c_guild.name.as_ref() {
            guild_name = g_name;
        }
    }
    
    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        if handler.queue().is_empty(){
            errf("Nothing to skip", guild_name);
            check_msg(msg.channel_id.send_message(&ctx.http,|m|{
                m.tts(false);
                m.embed(|e| {
                    e.title("Uh oh :)");
                    e.description("Queue is empty");
                    e.color(Colour::RED);
                    e
                });

                m
            }).await);
        }else{
            let thum = (*handler.queue().current().unwrap().metadata()).thumbnail.as_ref().unwrap().clone();
            if let Err(e) = handler.queue().skip(){
                errf(&format!("Failed {:?}", e), guild_name);
                check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
            }
            else{
                logf("Skipped", guild_name);
                check_msg(msg.channel_id.send_message(&ctx.http,|m|{
                    m.tts(false);
                    m.embed(|e| {
                        e.title("Steve skip :)");
                        e.description("Skipped:");
                        e.color(Colour::TEAL);
                        e.image(thum);
                        e
                    });

                    m
                }).await);
            }
        }
    }else{
        errf("Not in a voice channel to play in", guild_name);
        check_msg(msg.channel_id.send_message(&ctx.http,|m|{
            m.tts(false);
            m.embed(|e| {
                e.title("Banana");
                e.description("Steve not in channel");
                e.color(Colour::RED);
                e
            });

            m
        }).await);
    }
    Ok(())
}

#[command]
#[description("Resumes paused music")]
#[usage("resume")]
#[only_in("guilds")]
pub async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let mut guild_name = "None";
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        if let Some(g_name) = c_guild.name.as_ref() {
            guild_name = g_name;
        }
    }
    
    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        if handler.queue().is_empty(){
            errf("Nothing to resume", guild_name);
            check_msg(msg.channel_id.send_message(&ctx.http,|m|{
                m.tts(false);
                m.embed(|e| {
                    e.title("Uh oh :)");
                    e.description("Queue is empty");
                    e.color(Colour::RED);
                    e
                });

                m
            }).await);
        }else{
            if let Err(e) = handler.queue().resume(){
                errf(&format!("Failed {:?}", e), guild_name);
                check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
            }
            else{
                logf("Resumed", guild_name);
                check_msg(msg.channel_id.send_message(&ctx.http,|m|{
                    m.tts(false);
                    m.embed(|e| {
                        e.title("Steve resume :)");
                        e.description("Gummybears");
                        e.image((*handler.queue().current().unwrap().metadata()).thumbnail.as_ref().unwrap());
                        e.color(Colour::TEAL);
                        e
                    });

                    m
                }).await);
            }
        }
    }else{
        errf("Not in a voice channel to play in", guild_name);
        check_msg(msg.channel_id.send_message(&ctx.http,|m|{
            m.tts(false);
            m.embed(|e| {
                e.title("Banana");
                e.description("Steve not in channel");
                e.color(Colour::RED);
                e
            });

            m
        }).await);
    }
    Ok(())
}

#[command]
#[description("Lists out music in queue")]
#[usage("list")]
#[only_in("guilds")]
pub async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let mut guild_name = "None";
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        if let Some(g_name) = c_guild.name.as_ref() {
            guild_name = g_name;
        }
    }
    
    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        if handler.queue().is_empty(){
            errf("Nothing to list", guild_name);
            check_msg(msg.channel_id.send_message(&ctx.http,|m|{
                m.tts(false);
                m.embed(|e| {
                    e.title("Uh oh :)");
                    e.description("Queue is empty");
                    e.color(Colour::RED);
                    e
                });

                m
            }).await);
        }else{
            let mut list: String = "".to_owned();
            let qlist = handler.queue().current_queue();
            let mut ind = 1;
            for el in qlist{
                if ind >= 5 {
                    break;
                }
                list = format!("{}{:<2}:{:>100}\n",list,ind,(*el.metadata()).title.clone().unwrap().as_str());
                ind += 1;
            }
            logf("Logging list...", guild_name);
            check_msg(msg.channel_id.send_message(&ctx.http,|m|{
                m.tts(false);
                m.embed(|e| {
                    e.title("Steve content");
                    e.description(list);
                    e.color(Colour::TEAL);
                    e
                });

                m
            }).await);
        }
    }else{
        errf("Not in a voice channel to play in", guild_name);
        check_msg(msg.channel_id.send_message(&ctx.http,|m|{
            m.tts(false);
            m.embed(|e| {
                e.title("Banana");
                e.description("Steve not in channel");
                e.color(Colour::RED);
                e
            });

            m
        }).await);
    }
    Ok(())
}

#[command]
#[description("Explodes the queue")]
#[usage("boom")]
#[only_in("guilds")]
pub async fn boom(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let mut guild_name = "None";
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        if let Some(g_name) = c_guild.name.as_ref() {
            guild_name = g_name;
        }
    }

    let manager = songbird::get(ctx).await
    .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {        
        let mut handler = handler_lock.lock().await;
        handler.stop();
        if handler.queue().is_empty() {
            errf("Queue is empty", guild_name);
            check_msg(msg.reply(ctx, "Steve cannot kaboom :(").await);
        }
        while !handler.queue().is_empty(){
            handler.queue().dequeue(0);
        }
    }
    logf("Boom", guild_name);
    check_msg(msg.reply(ctx, "KABOOM!!!").await);

    Ok(())
}

#[allow(dead_code)]
async fn parse(tp: &str, query: &str, client: &ClientCredsSpotify) -> Result<QueryResult, String>{
    //get ClientCredsSpotify from data
    match tp{
        "p" => {
            get_playlist(&query.to_owned(), &client.to_owned(), true).await
        },
        "s" => {
            get_song(&query.to_owned(), &client.to_owned(), true).await
        },
        _ => {
            let err = format!("Invalid query type: {}", tp);
            errf(&err, "Spotify");
            return Err(err);
        }
    }
}

#[command]
#[description("Searches spotify for playlists or songs")]
#[usage("spotify <p(laylist) | s(ong)> <query>")]
#[only_in("guilds")]
pub async fn spotify(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // split args into query and type
    let data_map = ctx.data.read().await;
    let tp = args.single::<String>().unwrap();
    let query = args.rest();
    let client = data_map.get::<SpotifyClient>().unwrap().lock().await;
    let resp = parse(&tp, &query, &client).await;
    if let Ok(res) = resp {
        match res.tp {
            QueryType::Playlist => {
                check_msg(msg.channel_id.send_message(&ctx.http,|m|{
                    m.tts(false);
                    m.embed(|e| {
                        e.title(format!("Playlists for \"{}\" found: ",query));
                        e.description(res.result);
                        e.color(Colour::MEIBE_PINK);
                        e
                    });
    
                    m
                }).await);
            },
            QueryType::Song => {
                check_msg(msg.channel_id.send_message(&ctx.http,|m|{
                    m.tts(false);
                    m.embed(|e| {
                        e.title(format!("Songs for \"{}\" found: ",query));
                        e.description(res.result);
                        e.color(Colour::MAGENTA);
                        e
                    });
    
                    m
                }).await);
            }
        }
    }else if let Err(er) = resp {
        check_msg(msg.channel_id.send_message(&ctx.http,|m|{
            m.tts(false);
            m.embed(|e| {
                e.title(format!("Error: {}",er));
                e.color(Colour::RED);
                e
            });
            m
        }).await);
    }
    Ok(())
}

#[command]
#[description("Shows status of current song and place in queue")]
#[usage("status")]
#[only_in("guilds")]
pub async fn status(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).unwrap().id;
    let err: &str;
    let manager = songbird::get(ctx).await
    .expect("Songbird Voice client placed in at initialisation.").clone();
    let handler_lock_raw = manager.get(guild_id);
    if handler_lock_raw.is_none() {
        err = "Could not get handler";
        errf(err, "Songbird");
        check_msg(msg.channel_id.say(&ctx.http,err).await);
        return Ok(())
    }
    let handler_lock = handler_lock_raw.unwrap();
    let handler = handler_lock.lock().await;
    let curr = handler.queue().current_queue();
    let metadata = curr[0].metadata();
    let info_raw = curr[0].get_info().await;
    if info_raw.is_err() {
        err = "Could not get info for track info";
        errf(err, "Songbird");
        check_msg(msg.channel_id.say(&ctx.http,err).await);
        return Ok(());
    }
    let info = info_raw.unwrap();
    let position = info.position;
    let title = &metadata.title;
    let thumbnail = &metadata.thumbnail;
    let duration_raw = &metadata.duration;
    if title.is_none() || thumbnail.is_none() || duration_raw.is_none() {
        err = "No metadata";
        errf(&err, "Songbird");
        check_msg(msg.channel_id.say(&ctx.http,err).await);
        return Ok(());
    }
    let duration = duration_raw.unwrap();
    let output = format!("{}:{} min / {}:{} min - {}", position.as_secs()/60, position.as_secs()%60, duration.as_secs()/60, duration.as_secs()%60, title.to_owned().unwrap());
    dbgf(&output, "Songbird");
    check_msg(msg.channel_id.send_message(&ctx.http,|m|{
        m.tts(false);
        m.embed(|e| {
            e.title(output);
            e.thumbnail(thumbnail.to_owned().unwrap());
            e.color(Colour::MAGENTA);
            e
        });
        m
    }).await);
    Ok(())
}
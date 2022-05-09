use serenity::model::channel::Message;
use serenity::client::Context;
use serenity::framework::standard::{
    CommandResult,
    Args,
    macros::command
};
use serenity::{
    Result as SerenityResult
};
use chrono::Utc;
use serenity::utils::Colour;
use songbird::input::restartable::Restartable;
use youtube_dl::{YoutubeDlOutput,YoutubeDl,SearchOptions};
use rspotify::ClientCredsSpotify;
use crate::log::{log::logf, log::dbgf, error::errf};
use crate::commands::misc::{
    Guilds,
    SpotifyClient,
    Query,
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

    let guild = msg.guild(&ctx.cache).await.unwrap();
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

    let guild = msg.guild(&ctx.cache).await.unwrap();
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

    let guild = msg.guild(&ctx.cache).await.unwrap();
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

    let guild = msg.guild(&ctx.cache).await.unwrap();
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

#[command]
#[description("I'll play some songs for you")]
#[usage("p(lay) <song url|playlist url>")]
#[aliases("p")]
#[only_in("guilds")]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut guild_name = "None".to_string();
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            errf("No url provided", &guild_name);
            check_msg(msg.channel_id.say(&ctx.http, "Must provide a URL to a video or audio").await);

            return Ok(());
        },
    };

    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;
    let mut query_cache = data_map.get::<Query>().unwrap().write().await;
    let db = data_map.get::<Database>().unwrap().lock().await;
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;
    let gid = i64::from(guild_id);
    let mut queries: Option<Vec<String>> = None;
    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
        //copy name of guild
        guild_name = c_guild.get_name().to_string();
        queries = c_guild.get_query_results();
        c_guild.empty_query_results().update(&db);
        if query_cache.contains_key(&gid) {
            if let Some(q) = query_cache.get(&i64::from(guild_id)) {
                //if the difference between time is 5 min or more, wipe it and return
                let now = Utc::now();
                let diff = now.signed_duration_since(*q);
                if diff.num_minutes() >= MAX_QUERY as i64 {
                    query_cache.remove(&gid);
                    c_guild.empty_query_results().update(&db);
                    logf("Query cache wiped", &guild_name);
                    check_msg(msg.channel_id.say(&ctx.http, "Took to long to respond.  Try again!").await);
                    return Ok(());
                }
            }
        }
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
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let _handler = manager.join(guild_id, connect_to).await;

    let mut err: String;

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if searchable{
            // check if there are already searches in guild
            if let Some(urls) = queries {
                //url needs to be a number
                if let Ok(num) = url.parse::<i64>() {
                    //check if the number is in the queries
                    if urls.len() + 1 > num as usize {
                        let query = urls[(num - 1) as usize].to_string();
                        //play youtube video
                        let source = match Restartable::ytdl(query,true).await{
                            Ok(source) => source,
                            Err(e) => {
                                err = format!("Failed: {}",e);
                                errf(&err, &guild_name);
                                check_msg(msg.channel_id.say(&ctx.http, err).await);
                                return Ok(());
                            }
                        };
                        handler.enqueue_source(source.into());
                    }
                    errf(&format!("Out of range: {}", url), &guild_name);
                }
                errf(&format!("Not a number: {}", url), &guild_name);
            }else{
                //search option
                let search = SearchOptions::youtube(&url).with_count(MAX_QUERY);
                let output = YoutubeDl::search_for(&search).run();
                if let Ok(results_raw) = output {
                    let results: (Option<Vec<String>>,Option<Vec<String>>) = match results_raw {
                        YoutubeDlOutput::Playlist(pl_ref) => {
                            //deref pl
                            let pl = *pl_ref;
                            if let Some(items) = pl.entries {
                                //vector of songs need to become vector of strings
                                let mut urls: Vec<String> = Vec::new();
                                let mut values: Vec<String> = Vec::new();
                                for item in items {
                                    values.push(item.title.to_owned());
                                    if let Some(url) = item.webpage_url{
                                        //copy url from String to &str
                                        urls.push(url);
                                    }else{
                                        err = format!("No url found for: {}",item.title);
                                        errf(&err, &guild_name);
                                    }
                                }
                                (Some(urls),Some(values))
                            }else{
                                (None, None)
                            }
                        }
                        _ => (None, None)
                    };
                    let mut joined: String = "".to_string();
                    if let Some(c_guild) = guild_cache.get_mut(&i64::from(guild_id)) {
                        c_guild.set_query_results(&results.0).update(&db);
                    } 
                    if let Some(q) = results.1 {
                        for (i,url) in q.iter().enumerate() {
                            joined = format!("{}{}: {}\n",joined,i+1,url);
                        }
                    } else {
                        err = format!("No results found for: {}",&url);
                        errf(&err, &guild_name);
                        return Ok(());
                    }
                    let dbg = format!("DEBUG\n{}",joined);
                    dbgf(&dbg, &guild_name);
                    //embed
                    msg.channel_id.send_message(&ctx.http,|m| {
                        m.embed(|e| {
                            e.title("Search results");
                            e.description(joined);
                            e.color(Colour::BLITZ_BLUE)
                        })
                    }).await?;
                }else if let Err(e) = output {
                    err = format!("Failed: {}",e);
                    errf(&err, &guild_name);
                }
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

                                        check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                                        return Ok(());
                                    },
                                };
                                handler.enqueue_source(source.into());
                            }                            
                        }
                        YoutubeDlOutput::SingleVideo(yt_vid) => {
                            errf(&format!("Err unreasonable dead end with {:?}",yt_vid.url.unwrap()), &guild_name);
                            check_msg(msg.channel_id.say(&ctx.http, "Uh oh dead end").await);
                        }
                    }
                }
            }else{
                let source = match Restartable::ytdl(url, true).await {
                    Ok(source) => source,
                    Err(e) => {
                        errf(&format!("Err invlaid link {:?}", e), &guild_name);

                        check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                        return Ok(());
                    },
                };
                handler.enqueue_source(source.into());
            }
        }
        
        if handler.queue().is_empty() {
            if let Err(e) = handler.queue().resume(){
                errf(&format!("Failed {:?}", e), &guild_name);
                check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
            }else if !searchable{
                logf("Playing...", &guild_name);
                check_msg(msg.channel_id.say(&ctx.http, "Playing song").await);
            }
        }else{
            logf("Queued", &guild_name);
            check_msg(msg.channel_id.say(&ctx.http, "Queued").await);
        }
        //make anew
    } else {
        //make anew
        errf("Not in a voice channel to play in", &guild_name);
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to play in").await);
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

    let guild = msg.guild(&ctx.cache).await.unwrap();
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

    let guild = msg.guild(&ctx.cache).await.unwrap();
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

    let guild = msg.guild(&ctx.cache).await.unwrap();
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

    let guild = msg.guild(&ctx.cache).await.unwrap();
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

    let guild = msg.guild(&ctx.cache).await.unwrap();
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

    let guild = msg.guild(&ctx.cache).await.unwrap();
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

    let guild = msg.guild(&ctx.cache).await.unwrap();
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

    let guild = msg.guild(&ctx.cache).await.unwrap();
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
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let err: &str;
    let manager = songbird::get(ctx).await
    .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let curr = handler.queue().current_queue();
        let metadata = curr[0].metadata();
        if let Ok(info) = curr[0].get_info().await {
            let position = info.position;
            let length = info.play_time;
            if let Some(title) = &metadata.title {
                let title = title.to_owned();
                if let Some(thumbnail) = &metadata.thumbnail {
                    let thumbnail = thumbnail.to_owned();
                    let output = format!("{}/{} - {}", position.as_secs_f64()/60.0, length.as_secs_f64()/60.0, title);
                    dbgf(&output, "Songbird");
                    check_msg(msg.channel_id.send_message(&ctx.http,|m|{
                        m.tts(false);
                        m.embed(|e| {
                            e.title(output);
                            e.thumbnail(thumbnail);
                            e.color(Colour::MAGENTA);
                            e
                        });
                        m
                    }).await);
                }
            }
            err = "No metadata";
            errf(&err, "Songbird");
            check_msg(msg.channel_id.say(&ctx.http,err).await);
            return Ok(());
        }
        err = "Could not get info for track info";
        errf(err, "Songbird");
        check_msg(msg.channel_id.say(&ctx.http,err).await);
        return Ok(());
    }
    err = "Could not get handler";
    errf(err, "Songbird");
    check_msg(msg.channel_id.say(&ctx.http,err).await);
    Ok(())
}
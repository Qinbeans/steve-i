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
use serenity::utils::Colour;
use songbird::input::restartable::Restartable;
use youtube_dl::{YoutubeDlOutput,YoutubeDl};
use crate::log::{log::logf, error::errf};
use crate::commands::misc::Guilds;

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
#[only_in("guilds")]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut guild_name = "None";
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            errf("No url provided", guild_name);
            check_msg(msg.channel_id.say(&ctx.http, "Must provide a URL to a video or audio").await);

            return Ok(());
        },
    };

    let mut searchable: bool = false;

    if !url.starts_with("http") {
        searchable = true;
    }

    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;
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

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if searchable{
            let source = match Restartable::ytdl_search(url, true).await {
                Ok(source) => source,
                Err(e) => {
                    errf(&format!("Err not searchable {:?}", e), guild_name);
                    check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                    return Ok(());
                },
            };
            handler.enqueue_source(source.into());
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
                                        errf(&format!("Err not searchable {:?}", e), guild_name);

                                        check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                                        return Ok(());
                                    },
                                };
                                handler.enqueue_source(source.into());
                            }                            
                        }
                        YoutubeDlOutput::SingleVideo(yt_vid) => {
                            errf(&format!("Err unreasonable dead end with {:?}",yt_vid.url.unwrap()), guild_name);
                            check_msg(msg.channel_id.say(&ctx.http, "Uh oh dead end").await);
                        }
                    }
                }
            }else{
                let source = match Restartable::ytdl(url, true).await {
                    Ok(source) => source,
                    Err(e) => {
                        errf(&format!("Err invlaid link {:?}", e), guild_name);

                        check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                        return Ok(());
                    },
                };
                handler.enqueue_source(source.into());
            }
        }
        
        if handler.queue().is_empty() {
            if let Err(e) = handler.queue().resume(){
                errf(&format!("Failed {:?}", e), guild_name);
                check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
            }else{
                logf("Playing...", guild_name);
                check_msg(msg.channel_id.say(&ctx.http, "Playing song").await);
            }
        }else{
            logf("Queued", guild_name);
            check_msg(msg.channel_id.say(&ctx.http, "Queued").await);
        }
        //make anew
    } else {
        //make anew
        errf("Not in a voice channel to play in", guild_name);
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
#[description("I'll play some songs for you")]
#[usage("p(lay) <song url|playlist url>")]
#[only_in("guilds")]
pub async fn p(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut guild_name = "None";
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            errf("No url provided", guild_name);
            check_msg(msg.channel_id.say(&ctx.http, "Must provide a URL to a video or audio").await);

            return Ok(());
        },
    };

    let mut searchable: bool = false;

    if !url.starts_with("http") {
        searchable = true;
    }

    let data_map = ctx.data.read().await;
    let mut guild_cache = data_map.get::<Guilds>().unwrap().write().await;

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;
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

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if searchable{
            let source = match Restartable::ytdl_search(url, true).await {
                Ok(source) => source,
                Err(e) => {
                    errf(&format!("Err not searchable {:?}", e), guild_name);
                    check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                    return Ok(());
                },
            };
            handler.enqueue_source(source.into());
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
                                        errf(&format!("Err not searchable {:?}", e), guild_name);

                                        check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                                        return Ok(());
                                    },
                                };
                                handler.enqueue_source(source.into());
                            }                            
                        }
                        YoutubeDlOutput::SingleVideo(yt_vid) => {
                            errf(&format!("Err unreasonable dead end with {:?}",yt_vid.url.unwrap()), guild_name);
                            check_msg(msg.channel_id.say(&ctx.http, "Uh oh dead end").await);
                        }
                    }
                }
            }else{
                let source = match Restartable::ytdl(url, true).await {
                    Ok(source) => source,
                    Err(e) => {
                        errf(&format!("Err invlaid link {:?}", e), guild_name);

                        check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                        return Ok(());
                    },
                };
                handler.enqueue_source(source.into());
            }
        }
        
        if handler.queue().is_empty() {
            if let Err(e) = handler.queue().resume(){
                errf(&format!("Failed {:?}", e), guild_name);
                check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
            }else{
                logf("Playing...", guild_name);
                check_msg(msg.channel_id.say(&ctx.http, "Playing song").await);
            }
        }else{
            logf("Queued", guild_name);
            check_msg(msg.channel_id.say(&ctx.http, "Queued").await);
        }
        //make anew
    } else {
        //make anew
        errf("Not in a voice channel to play in", guild_name);
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to play in").await);
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
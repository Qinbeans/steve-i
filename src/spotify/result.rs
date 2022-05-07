use rspotify::{
    ClientError,
    ClientCredsSpotify,
    clients::base::BaseClient
};
use rspotify_model::{
    enums::{
        types::SearchType,
        country::Country,
        misc::Market
    },
    search::SearchResult,
};
use rspotify_http::HttpError;
use async_recursion::async_recursion;

use crate::log::{log::logf, error::errf};

pub enum QueryType{
    Playlist,
    Song
}

pub struct QueryResult{
    pub result: String,
    pub tp: QueryType
}

#[async_recursion]
pub async fn get_playlist(query: &str, client: &ClientCredsSpotify, attempt: bool) -> Result<QueryResult, String>{
    let res = client.search(query,&SearchType::Playlist,Some(&Market::Country(Country::UnitedStates)),None,Some(5),None).await;
    let mut err = format!("No playlists found: {}", query);
    if let Ok(good_res) = res {
        match good_res{
            SearchResult::Playlists(playlists) => {
                let mut result: String = "".to_string();
                for pl in playlists.items{
                    if let Some(name) = pl.owner.display_name{
                        result += &format!("{} by {}\n",pl.name,name);
                    }else{
                        result += &format!("{}\n",pl.name);
                    }
                }
                logf(&format!("Results: {}",result), "Playlist");
                return Ok(QueryResult {
                    result,
                    tp: QueryType::Playlist
                });
            },
            _ => {
                errf(&err, "Playlist");
                return Err(err);
            }
        }
    }
    //check if the error is from outdated token
    if attempt{
        if let Err(e) = res {
            match e{
                ClientError::Http(http_raw) => {
                    //deref to http error
                    match *http_raw {
                        HttpError::StatusCode(code) => {
                            if code.status() == 401 {
                                errf("Attempting again, token out of date...", "Playlist");
                                let resp = client.refresh_token().await;
                                if let Ok(_) = resp {
                                    return get_playlist(query, client, false).await;
                                }
                                err = format!("Failed to refresh token: {}", query);
                                errf(&err, "Playlist");
                                return Err(err);
                            }
                        },
                        _ => {
                            errf(&err, "Playlist");
                            return Err(err);
                        }
                    }
                },
                _ => {
                    errf(&err, "Playlist");
                    return Err(err);
                }
            }
        }
    }
    errf(&err, "Playlist");
    return Err(err);
}

#[async_recursion]
pub async fn get_song(query: &str, client: &ClientCredsSpotify, attempt: bool) -> Result<QueryResult, String>{
    let res = client.search(query,&SearchType::Track,Some(&Market::Country(Country::UnitedStates)),None,Some(5),None).await;
    let mut err = format!("No Song found: {}", query);
    if let Ok(good_res) = res {
        match good_res{
            SearchResult::Tracks(song) => {
                let mut result: String = "".to_string();
                for pl in song.items{
                    let name = &pl.artists[0].name;
                    result += &format!("{} by {}\n",pl.name,name);
                }
                logf(&format!("Results: {}",result), "Song");
                return Ok(QueryResult {
                    result,
                    tp: QueryType::Song
                });
            },
            _ => {
                errf(&err, "Song");
                return Err(err);
            }
        }
    }
    if attempt{
        if let Err(e) = res {
            match e{
                ClientError::Http(http_raw) => {
                    //deref to http error
                    match *http_raw {
                        HttpError::StatusCode(code) => {
                            if code.status() == 401 {
                                errf("Attempting again, token out of date...", "Playlist");
                                let resp = client.refresh_token().await;
                                if let Ok(_) = resp {
                                    return get_song(query, client, false).await;
                                }
                                err = format!("Failed to refresh token: {}", query);
                                errf(&err, "Playlist");
                                return Err(err);
                            }
                        },
                        _ => {
                            errf(&err, "Playlist");
                            return Err(err);
                        }
                    }
                },
                _ => {
                    errf(&err, "Playlist");
                    return Err(err);
                }
            }
        }
    }
    errf(&err, "Song");
    return Err(err);
}
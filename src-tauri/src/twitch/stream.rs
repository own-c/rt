use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::util;

use super::{
    main,
    query::{GraphQLQuery, GraphQLResponse, UseLiveQuery, UseLiveResponse},
};

const USHER_API: &str = "https://usher.ttvnw.net/api/channel/hls";

#[derive(Serialize, Debug)]
pub struct LiveNow {
    pub username: String,
    pub started_at: String,
}

pub async fn fetch_live_now(usernames: Vec<String>) -> Result<HashMap<String, LiveNow>> {
    let mut query: Vec<UseLiveQuery> = Vec::new();

    for username in usernames {
        if username.is_empty() {
            continue;
        }

        query.push(UseLiveQuery::new(&username));
    }

    let response: Vec<UseLiveResponse> = match main::send_query(query).await {
        Ok(data) => data,
        Err(err) => {
            return Err(anyhow!("Failed to fetch UseLive: {err}"));
        }
    };

    let mut live_now: HashMap<String, LiveNow> = HashMap::new();

    for obj in response {
        if obj.data.user.stream.is_none() {
            continue;
        }

        let stream = obj.data.user.stream.unwrap();
        let username = obj.data.user.login;

        let live = LiveNow {
            username: username.clone(),
            started_at: stream.created_at,
        };

        live_now.insert(username, live);
    }

    Ok(live_now)
}

#[tauri::command]
pub async fn fetch_stream_playback(username: &str, backup: bool) -> Result<String, String> {
    if username.is_empty() {
        return Err(String::from("No username provided"));
    }

    let gql = GraphQLQuery::playback_query(username, backup);

    let response: GraphQLResponse = match main::send_query(gql).await {
        Ok(response) => response,
        Err(err) => {
            return Err(format!("Failed to fetch stream info: {err}"));
        }
    };

    let Some(stream_playback) = response.data.stream_playback_access_token else {
        return Err(String::from("No stream playback access token found"));
    };

    let signature = stream_playback.signature;
    let value = stream_playback.value;

    let url = playlist_url(username, backup, &signature, &value);

    Ok(url)
}

fn playlist_url(username: &str, backup: bool, signature: &str, token: &str) -> String {
    let mut url = format!("{USHER_API}/{username}.m3u8");

    let random_number = util::random_number(1_000_000, 10_000_000);

    if backup {
        url.push_str(&format!("?platform=ios&supported_codecs=h264&player=twitchweb&fast_bread=true&p={random_number}&sig={signature}&token={token}"));
    } else {
        url.push_str(&format!("?platform=web&supported_codecs=av1,h265,h264&allow_source=true&player=twitchweb&fast_bread=true&p={random_number}&sig={signature}&token={token}"));
    }

    url.to_string()
}

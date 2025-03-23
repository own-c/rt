use std::collections::HashMap;

use anyhow::{anyhow, Result};
use log::error;

use super::{
    emote::{self, Emote, TWITCH_EMOTES_CDN},
    main,
    queries::{GraphQLQuery, GraphQLResponse},
};

pub struct TwitchUser {
    pub id: String,
    pub username: String,
    pub avatar: String,
    pub emotes: HashMap<String, Emote>,
}

pub async fn fetch_user(username: &str) -> Result<TwitchUser> {
    let gql = GraphQLQuery::full_user(username);

    let response: GraphQLResponse = match main::send_query(gql).await {
        Ok(response) => response,
        Err(err) => {
            return Err(anyhow!("Failed to fetch user '{username}': {err}"));
        }
    };

    let Some(user) = response.data.user else {
        return Err(anyhow!("User '{username}' not found"));
    };

    let mut user_emotes: HashMap<String, Emote> = HashMap::new();
    for product in user.subscription_products.unwrap() {
        for emote in product.emotes {
            let name = emote.token;
            let url = format!("{TWITCH_EMOTES_CDN}/{}/default/dark/1.0", emote.id);

            let emote = Emote {
                name: name.clone(),
                url,
                width: 28,
                height: 28,
            };

            user_emotes.insert(name, emote);
        }
    }

    let user_id = user.id.unwrap();

    let seventv_emotes = match emote::fetch_7tv_emotes(&user_id).await {
        Ok(emotes) => emotes,
        Err(err) => {
            error!("Failed to fetch 7tv emotes: {err}");
            HashMap::new()
        }
    };

    let bettertv_emotes = match emote::fetch_bettertv_emotes(&user_id).await {
        Ok(emotes) => emotes,
        Err(err) => {
            error!("Failed to fetch bettertv emotes: {err}");
            HashMap::new()
        }
    };

    user_emotes.extend(seventv_emotes);
    user_emotes.extend(bettertv_emotes);

    let user = TwitchUser {
        id: user_id,
        username: username.to_string(),
        avatar: user.profile_image_url.unwrap_or_default(),
        emotes: user_emotes,
    };

    Ok(user)
}

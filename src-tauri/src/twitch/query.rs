use serde::{Deserialize, Serialize};

const USE_LIVE_QUERY_HASH: &str =
    "639d5f11bfb8bf3053b424d9ef650d04c4ebb7d94711d644afb08fe9a0fad5d9";

/// Main query struct used throughout the app.
///
/// I've thought a lot about how to make requests to the API, I believe that the amount of operations using persisted queries
/// would outweigh the benefits of the using them (less overhead when parsing on the server side).
///
/// For now, until a better method is found, I will be doing it this way.
#[derive(Serialize)]
pub struct GraphQLQuery {
    query: String,
}

impl GraphQLQuery {
    pub fn full_user(username: &str) -> Self {
        let gql = format!(
            r#"{{
                user(login: "{username}") {{
                    id
                    profileImageURL(width: 50)
                    stream {{
                        title
                        viewersCount
                        createdAt
                        game {{
                            id
                            name
                        }}
                    }}
                    subscriptionProducts {{
                        emotes {{
                            id
                            token
                        }}
                    }}
                }}
            }}"#
        );

        Self {
            query: gql.trim().to_string(),
        }
    }

    /// Used to retrieve playback access token for a stream.
    pub fn playback_query(username: &str, backup_stream: bool) -> Self {
        let platform = if backup_stream { "ios" } else { "web" };
        let player_type = if backup_stream { "autoplay" } else { "site" };

        let gql = format!(
            r#"{{
                streamPlaybackAccessToken(
                    channelName: "{username}",
                    params: {{
                        platform: "{platform}",
                        playerBackend: "mediaplayer",
                        playerType: "{player_type}",
                    }}
                )
                {{
                    value
                    signature
                }}
            }}"#
        );

        Self {
            query: gql.trim().to_string(),
        }
    }
}

// Most fields here are optional because this struct is used in different queries,
// not having them optional would cause issues when deserializing the response.

#[derive(Deserialize)]
pub struct GraphQLResponse {
    pub data: GraphQLResponseData,
}

#[derive(Deserialize)]
pub struct GraphQLResponseData {
    pub user: Option<GraphQLResponseUser>,
    #[serde(
        rename = "streamPlaybackAccessToken",
        skip_serializing_if = "Option::is_none"
    )]
    pub stream_playback_access_token: Option<StreamPlaybackAccessToken>,
}

#[derive(Deserialize)]
pub struct GraphQLResponseUser {
    pub id: Option<String>,
    #[serde(rename = "profileImageURL")]
    pub profile_image_url: Option<String>,
    #[serde(rename = "subscriptionProducts")]
    pub subscription_products: Option<Vec<SubscriptionProduct>>,
}

#[derive(Deserialize)]
pub struct SubscriptionProduct {
    pub emotes: Vec<GraphQLResponseEmote>,
}

#[derive(Deserialize)]
pub struct StreamPlaybackAccessToken {
    pub value: String,
    pub signature: String,
}

#[derive(Deserialize)]
pub struct GraphQLResponseEmote {
    pub id: String,
    pub token: String,
}

// Persistent queries and their responses.

// I don't plan on querying the stream info when refreshing users, so this query is really good for this.

#[derive(Serialize)]
pub struct UseLiveQuery {
    #[serde(rename = "operationName")]
    operation_name: String,
    variables: ChannelLoginVariable,
    extensions: QueryExtensions,
}

#[derive(Serialize, Deserialize)]
pub struct ChannelLoginVariable {
    #[serde(rename = "channelLogin")]
    channel_login: String,
}

impl ChannelLoginVariable {
    pub fn new(channel_login: &str) -> Self {
        Self {
            channel_login: channel_login.to_string(),
        }
    }
}

impl UseLiveQuery {
    pub fn new(channel_login: &str) -> Self {
        Self {
            operation_name: String::from("UseLive"),
            variables: ChannelLoginVariable::new(channel_login),
            extensions: QueryExtensions::new(USE_LIVE_QUERY_HASH),
        }
    }
}

#[derive(Deserialize)]
pub struct UseLiveResponse {
    pub data: UseLiveResponseData,
}

#[derive(Deserialize)]
pub struct UseLiveResponseData {
    pub user: UseLiveResponseUser,
}

#[derive(Deserialize)]
pub struct UseLiveResponseUser {
    pub login: String,
    pub stream: Option<UseLiveResponseStream>,
}

#[derive(Deserialize)]
pub struct UseLiveResponseStream {
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

// Every persistent query has these fields

#[derive(Serialize, Deserialize)]
pub struct QueryExtensions {
    #[serde(rename = "persistedQuery")]
    persisted_query: PersistedQuery,
}

impl QueryExtensions {
    pub fn new(hash: &str) -> Self {
        Self {
            persisted_query: PersistedQuery {
                version: 1,
                sha256_hash: hash.to_string(),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PersistedQuery {
    version: u64,
    #[serde(rename = "sha256Hash")]
    sha256_hash: String,
}

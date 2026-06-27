use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{self, ActivityType::Listening},
};
use rand::distr::{Alphanumeric, SampleString};
use reqwest::Client;
use std::{fs, time::Duration};

#[derive(serde::Deserialize, Default)]
struct Config {
    #[serde(rename = "applicationID")]
    application_id: i64,
    #[serde(rename = "http_address")]
    httpaddr: String,
    username: String,
    password: String,
    #[serde(rename = "useimages")]
    imagebool: bool,
    pollingrate: i32,
}
fn parseconfig(config: &mut Config) -> serde_json::Result<()> {
    let configfile: String = fs::read_to_string("config.json").unwrap();
    let parsed: Config = serde_json::from_str(&configfile)?;

    config.application_id = parsed.application_id;
    config.httpaddr = parsed.httpaddr;
    config.username = parsed.username;
    config.password = parsed.password;
    config.imagebool = parsed.imagebool;
    config.pollingrate = parsed.pollingrate;

    Ok(())
}

#[derive(Debug)]
struct TokenData {
    token: md5::Digest,
    salt: String,
}
fn gentoken(password: &str) -> TokenData {
    let salt: String = Alphanumeric.sample_string(&mut rand::rng(), 6);
    let generatedtoken: md5::Digest = md5::compute(format!("{}{}", password, salt));

    TokenData {
        token: generatedtoken,
        salt,
    }
}

async fn apirequest(
    configstruct: &Config,
    tokendata: &TokenData,
    parsed_api_data: &mut ParsedData,
    body: &Client,
) -> Result<String, reqwest::Error> {
    let url: String = format!("{}/rest/getNowPlaying.view", configstruct.httpaddr);
    let token_stripped: &String = &format!("{:x}", tokendata.token);

    let respbody = body
        .get(url)
        .query(&[
            ("u", configstruct.username.as_str()),
            ("t", token_stripped),
            ("s", tokendata.salt.as_str()),
            ("v", "1.16.1"),
            ("c", "navidromeRPC"),
            ("f", "json"),
        ])
        .send()
        .await?;

    let mut imageurl = respbody.url().clone();
    imageurl.set_path("/rest/getCoverArt");
    imageurl
        .query_pairs_mut()
        .append_pair("size", "512")
        .append_pair("id", &parsed_api_data.cover_art);
    parsed_api_data.constructedlargeimageurl = imageurl.to_string();
    let response: String = respbody.text().await?;
    Ok(response)
}

#[derive(serde::Deserialize, Default)]
struct ParsedData {
    username: String,

    title: String,
    artist: String,
    album: String,
    #[serde(rename = "playCount")]
    play_count: i32,

    #[serde(rename = "coverArt")]
    cover_art: String,
    constructedlargeimageurl: String,
    // constructedsmallimageurl: (),
}
fn parseapirequest(parsed_api_data: &mut ParsedData, apidata: &str) {
    let parsed: serde_json::Value = serde_json::from_str(apidata).unwrap_or_default();
    let entry: &serde_json::Value = &parsed["subsonic-response"]["nowPlaying"]["entry"][0];

    parsed_api_data.username = entry["username"].as_str().unwrap_or_default().to_string();
    parsed_api_data.title = entry["title"].as_str().unwrap_or_default().to_string();
    parsed_api_data.artist = entry["artist"].as_str().unwrap_or_default().to_string();
    parsed_api_data.album = entry["album"].as_str().unwrap_or_default().to_string();
    parsed_api_data.play_count = entry["playCount"].as_i64().unwrap_or_default() as i32;
    parsed_api_data.cover_art = entry["coverArt"].as_str().unwrap_or_default().to_string();
}

fn init_ipc(
    parsed_api_data: &ParsedData,
    client: &mut DiscordIpcClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let largeimageurl: String = format!("{}", parsed_api_data.constructedlargeimageurl);

    client.set_activity(
        activity::Activity::new()
            .activity_type(Listening)
            .name(format!("{}", &parsed_api_data.artist))
            .details(format!("{}", &parsed_api_data.title))
            .state(format!(
                "in: {} :: {} plays",
                &parsed_api_data.album, &parsed_api_data.play_count
            ))
            .assets(activity::Assets::new().large_image(largeimageurl.to_string())),
    )?;

    Ok(())
}

fn reclient(configstruct: &Config) -> DiscordIpcClient {
    let mut client: DiscordIpcClient = DiscordIpcClient::new(format!("{}", configstruct.application_id));
    if let Err(initerror) = client.connect() {
        eprintln!("reclient RPC connect fail :: {}", initerror);
        std::process::exit(1)
    };
    return client;
}

#[tokio::main]
async fn main() {
    let mut configstruct: Config = Config::default();

    parseconfig(&mut configstruct).unwrap();

    let token: TokenData = gentoken(&configstruct.password);
    let mut parsed_api_data: ParsedData = ParsedData::default();
    let mut apidata: String = String::new();
    let mut client: DiscordIpcClient = reclient(&configstruct);
    let body: Client = reqwest::Client::new();
    let mut mediastate: bool = false;

    tokio::select! {
        _ = async {
                loop {
                    apidata = apirequest(&configstruct, &token, &mut parsed_api_data, &body).await.unwrap();
                    parseapirequest(&mut parsed_api_data, &apidata);

                    if parsed_api_data.title.is_empty() {
                        let _ = client.close();

                        mediastate = false;
                        println!("closing ipc, ms {} (f)", mediastate);
                    } else if !parsed_api_data.title.is_empty() && !mediastate {
                        client = reclient(&configstruct);
                        apidata = apirequest(&configstruct, &token, &mut parsed_api_data, &body).await.unwrap();
                        if let Err(initerror) = init_ipc(&parsed_api_data, &mut client) {
                            eprintln!("RPC Init fail :: {}", initerror);
                            std::process::exit(1)
                        };

                        mediastate = true;
                        println!("opening ipc, ms {} (t)", mediastate);
                    };

                    // debug // println!("{} {} {} {} {} {} {}", parsed_api_data.username, parsed_api_data.title, parsed_api_data.artist, parsed_api_data.album, parsed_api_data.play_count, parsed_api_data.cover_art, parsed_api_data.constructedlargeimageurl);

                    tokio::time::sleep(Duration::from_secs(configstruct.pollingrate as u64)).await;
                }
            } => {}

            _ = tokio::signal::ctrl_c() => {
                let _ = client.close();
            }


    }

}

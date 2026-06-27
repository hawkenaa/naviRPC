# naviRPC
A Lightweight Linux Discord Rich Presence Scraper for Navidrome Music Servers in Rust

<img width="281" height="115" alt="example_1" src="https://github.com/user-attachments/assets/4f38059f-5a69-455c-b888-ad017ab51ef5" />

___

naviRPC supports HTTPS and HTTP (http untested) over public and local domain.
No Oauth2 required.

naviRPC is __relatively__ simple though incredibly lightweight. (KISS)

Designed for Linux, on Linux. (Artix)

___
# Config and Running

``` {
    "applicationID": 0000000000000000000,
    "http_address": "https://navidrome.yoursite.tld",
    "username": "username",
    "password": "password",
    "useimages": true/false,
    "pollingrate": 5 
}
```

A valid Navidrome Account is required to GET from the subsonic API.

Server address must be in a vanilla format __with__ headers attached, IE:
"http://192.168.1.213:8096"
"https://navidrome.yoursite.tld"

HTTPS and HTTP is handled __manually__ via config.

An Application ID (Discord Application) is used to give Rich Presence a name and valid handshake. These can be named anything, but must be an application.

Displaying music artwork is only possible if your library is publically exposed (reachable over internet), as Discord cannot access Jellyfin over LAN or VPN (Tailscale). This does not apply to Rich Presence.

Polling rate is used to manage polling for your Navidrome Server and IPC socket (RPC). Any integer can be used, but 3-15 seconds are recomended, with lower integers resulting in faster updates, and vice-versa. Your IPC socket will only update if your media changes (Different Track), you pause your media, or if you skip forward or backward in your track.

the IPC connection will automatically close when no media is playing whatsoever, ~~but remains open if you have items queued or pause your media~~ navidrome automatically sets your media to non-active after a set amount of time, which WILL close your connection. IPC will automatically reopen when media is found active (playing).

This program will only pull the first session found, paused or not, and does ~~not~~ (yet) have user-filter support.

Due to the limitations of the OpenSubsonicAPI and SubsonicAPI, timestamps do not automatically save using the navidrome web-player, though custom clients may post and save to the server, allowing timestamps to show.
Timestamps are not-yet supported but I will look into it.

___

To run, download the code and config, with each in the same directory. use `cargo run` with an appropriate config and rich presence will launch. Make sure Discord is not sandboxed in any way (Flatpak) to allow IPC to connect. (there are workarounds to this)

Init processes are delegated to the user, as this does not run by itself on boot.
___

As I have moved to Navidrome, I do plan on adding to this project.
(IE: custom status arrangements, timestamp support, daemon and docker support)

This project was started for learning _Rust_.
_(meaning all gruelly handwritten!!!!!)_

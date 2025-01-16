# Comet

[![Build nightly](https://github.com/imLinguin/comet/actions/workflows/build.yml/badge.svg)](https://github.com/imLinguin/comet/actions/workflows/build.yml)
[![Version](https://img.shields.io/github/v/release/imLinguin/comet?label=version)](https://github.com/imLinguin/comet/releases/latest)
[![Static Badge](https://img.shields.io/badge/Steam%20Deck%20Usage%20Guide-darkslategrey?logo=steamdeck)](docs/steamdeck/USAGE.md)

Open Source implementation of GOG Galaxy's Communication Service

This project aims to implement calls made by game through SDK.  
Note: that means it can't and won't replace Communication Service in official client

This will provide minimal and platform-agnostic SDK. For use in game launchers like Heroic or Lutris

Project is continuation of Yepoleb's work https://gitlab.com/Yepoleb/comet/ but in
~~Python~~ [now in Rust](https://github.com/imLinguin/comet/issues/15)

## Supported Requests

### Game

- [x] LIBRARY_INFO_REQUEST
- [x] AUTH_INFO_REQUEST
- [x] GET_USER_STATS_REQUEST
- [x] SUBSCRIBE_TOPIC_REQUEST
- [x] UPDATE_USER_STAT_REQUEST
- [x] DELETE_USER_STATS_REQUEST
- [x] GET_USER_ACHIEVEMENTS_REQUEST
- [x] UNLOCK_USER_ACHIEVEMENT_REQUEST
- [x] CLEAR_USER_ACHIEVEMENT_REQUEST
- [x] DELETE_USER_ACHIEVEMENTS_REQUEST
- [x] GET_LEADERBOARDS_REQUEST
- [x] GET_LEADERBOARDS_BY_KEY_REQUEST
- [x] GET_LEADERBOARD_ENTRIES_GLOBAL_REQUEST
- [x] GET_LEADERBOARD_ENTRIES_AROUND_USER_REQUEST
- [x] GET_LEADERBOARD_ENTRIES_FOR_USERS_REQUEST
- [x] SET_LEADERBOARD_SCORE_REQUEST
- [x] CREATE_LEADERBOARD_REQUEST
- [ ] GET_GLOBAL_STATS_REQUEST

### Overlay

This includes calls made to be forwarded to game process

- [x] START_GAME_SESSION_REQUEST
- [x] OVERLAY_FRONTEND_INIT_DATA_REQUEST
- [x] OVERLAY_STATE_CHANGE_NOTIFICATION
- [x] ACCESS_TOKEN_REQUEST
- [x] OVERLAY_INITIALIZATION_NOTIFICATION
- [x] NOTIFY_ACHIEVEMENT_UNLOCKED
- [x] SHOW_WEB_PAGE
- [x] VISIBILITY_CHANGE_NOTIFICATION
- [x] SHOW_INVITATION_DIALOG
- [ ] GAME_JOIN_REQUEST_NOTIFICATION
- [ ] GAME_INVITE_SENT_NOTIFICATION

## How to use

Comet integration in game launchers

- Heroic - ✅ an experimental feature enabled by default (as of 2.15.0)
- Lutris - ❓ planned, no ETA
- Minigalaxy - ❓ open for Pull Requests

For manual instructions see [running](#running)

Some client SDK versions require Windows service to be registered, refer to [dummy service](./dummy-service/README.md)

### Authentication

You need to obtain `access_token`, `refresh_token` and `user_id` either manually, or by importing them:

#### Via [Heroic Games Launcher](https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher)

Log in to GOG within the launcher.  
Use `--from-heroic` for automatic import.

#### Via [Lutris](https://github.com/lutris/lutris)

Log in to Lutris's GOG source.  
Use `--from-lutris` for automatic import.

#### Via [wyvern](https://github.com/nicohman/wyvern) (CLI)

Log in to GOG in wyvern  
Use `--from-wyvern` for automatic import.

#### Via [gogdl](https://github.com/Heroic-Games-Launcher/heroic-gogdl) (CLI)

If GOG authentication has never been performed in Heroic on the current user, create the expected directory:

```
mkdir -p $HOME/.config/heroic/gog_store
```

Then, run the command:

```
./bin/gogdl --auth-config-path $HOME/.config/heroic/gog_store/auth.json auth --code <code>
```

Obtain the code by logging in using this URL, then copying the code value from the resulting URL:

https://login.gog.com/auth?client_id=46899977096215655&layout=galaxy&redirect_uri=https%3A%2F%2Fembed.gog.com%2Fon_login_success%3Forigin%3Dclient&response_type=code

### Running

```
comet --token "<access_token>" --refresh_token "<refresh_token>" --user-id <user_id> --username <USERNAME>
```

Or if you are using Heroic/gogdl

```
comet --from-heroic --username <USERNAME>
```

Or Lutris

```
comet --from-lutris --username <USERNAME>
```

Or wyvern

```
comet --from-wyvern --username <USERNAME>
```

Or use the shortcut script provided for non-Steam shortcuts. See the [Steam Deck Usage Guide](docs/steamdeck/USAGE.md).

## Configuration

You can adjust basic overlay settings with comet configuration file.  
File locations:

- Windows - `%APPDATA%/comet/config.toml`
- Mac - `~/Library/Application Support/comet/config.toml`
- Linux - `$XDG_CONFIG_HOME/comet/config.toml`

Default configuration file is as follows

```toml
[overlay]
notification_volume = 50  # value from 0 to 100
position = "bottom_right" # position where notifications are shown: top_left top_right bottom_left bottom_right

# Controls chat message notifications
[overlay.notifications.chat]
enabled = true
sound= true

# Controls notifications when friend becomes online
[overlay.notifications.friend_online]
enabled = true
sound= true

# Controls notifications when someone sends you a friend invititation
[overlay.notifications.friend_invite]
enabled = true
sound= true

# Controls notifications when friend starts playing a game
[overlay.notifications.friend_game_start]
enabled = true
sound= true

# Controls notifications when someone sends you a game invite
[overlay.notifications.game_invite]
enabled = true
sound= true
```

## Contributing

Join [Heroic Discord](https://discord.gg/rHJ2uqdquK) and reach out to us on
special [thread](https://discord.com/channels/812703221789097985/1074048840958742648)

[Here](https://imlinguin.vercel.app/blog/galaxy-comm-serv-re-setup) you can find a blog post about setting up
environment for tracing the Communication Service calls (involving Proxifier and custom mitmproxy)

Reverse engineered protobuf definitions are available here: https://github.com/Yepoleb/gog_protocols

## Debugging SDK Client

In order to dump logging from SDK client (the game) download [GalaxyPeer.ini](https://items.gog.com/GalaxyPeer.zip),
when placed next to game .exe it will write GalaxyPeer.log when the game is running.

> [!WARNING]  
> Proceed with caution, the log may contain sensitive information,
> make sure to remove such data before sharing the file with others.

## Sponsoring

If you want to contribute financially you can do so via my [Ko-Fi](https://ko-fi.com/imlinguin).  
You can also use any of the options to [support Heroic](https://heroicgameslauncher.com/donate)

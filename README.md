# Comet

Open Source implementation of GOG Galaxy's Communication Service

This project aims to implement calls made by game through SDK.  
Note: that means it can't and won't replace Communication Service in official client

This will provide minimal and platform-agnostic SDK. For use in game launchers like Heroic or Lutris

Project is continuation of Yepoleb's work [https://gitlab.com/Yepoleb/comet/](https://gitlab.com/Yepoleb/comet/) but in
~~Python~~ [now in Rust](https://github.com/imLinguin/comet/issues/15)

## Supported Requests

Excluding Overlay, and Cloud Sync related.

- [x] AUTH_INFO_REQUEST
- [x] GET_USER_STATS_REQUEST
- [x] SUBSCRIBE_TOPIC_REQUEST
- [x] UPDATE_USER_STAT_REQUEST
- [ ] DELETE_USER_STATS_REQUEST
- [ ] GET_GLOBAL_STATS_REQUEST
- [x] GET_USER_ACHIEVEMENTS_REQUEST
- [x] UNLOCK_USER_ACHIEVEMENT_REQUEST
- [x] CLEAR_USER_ACHIEVEMENT_REQUEST
- [ ] DELETE_USER_ACHIEVEMENTS_REQUEST
- [x] GET_LEADERBOARDS_REQUEST
- [x] GET_LEADERBOARD_ENTRIES_GLOBAL_REQUEST
- [x] GET_LEADERBOARD_ENTRIES_AROUND_USER_REQUEST
- [x] GET_LEADERBOARD_ENTRIES_FOR_USERS_REQUEST
- [ ] SET_LEADERBOARD_SCORE_REQUEST
- [ ] AUTH_STATE_CHANGE_NOTIFICATION
- [x] GET_LEADERBOARDS_BY_KEY_REQUEST
- [ ] CREATE_LEADERBOARD_REQUEST
- [ ] OVERLAY_STATE_CHANGE_NOTIFICATION

## How to use

Currently service supports small amount of calls, but these are enough to play Gwent for example.

Check [running](#running)

Some client SDK versions require Windows service to be registered, refer to [dummy service](./dummy-service/README.md)

### Authentication

You need to obtain `access_token`, `refresh_token` and `user_id` either manually, or by importing them:

#### Via [Heroic Games Launcher](https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher)

Log in to GOG within the launcher.

#### Via [Lutris](https://github.com/lutris/lutris)

Log in to Lutris's GOG source.

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

## Contributing

Join [Heroic Discord](https://discord.gg/rHJ2uqdquK) and reach out to us on
special [thread](https://discord.com/channels/812703221789097985/1074048840958742648)

[Here](https://imlinguin.vercel.app/blog/galaxy-comm-serv-re-setup) you can find a blog post about setting up
environment for tracing the Communication Service calls (involving Proxifier and custom mitmproxy)

Reverse engineered protobuf definitions are available here: https://github.com/Yepoleb/gog_protocols

## Debugging SDK Client

In order to dump logging from SDK client download [GalaxyPeer.ini](https://items.gog.com/GalaxyPeer.zip),
when placed next to game .exe it will write GalaxyPeer.log when the game is running.

> [!WARNING]  
> Proceed with caution, the log may contain sensitive information,
> make sure to remove such data before sharing the file with others.

## Sponsoring

If you want to contribute financially you can do so via my [Ko-Fi](https://ko-fi.com/imlinguin).  
You can also use any of the options to [support Heroic](https://heroicgameslauncher.com/donate)


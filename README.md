# Comet
Open Source implementation of GOG Galaxy's Communication Service

This project aims to implement calls made by game through SDK.

Project is continuation of Yepoleb's work [https://gitlab.com/Yepoleb/comet/](https://gitlab.com/Yepoleb/comet/) but in Python


## Supported Requests
Excluding Overlay, and Cloud Sync related.

- [x] AUTH_INFO_REQUEST
- [x] GET_USER_STATS_REQUEST
- [ ] SUBSCRIBE_TOPIC_REQUEST
- [ ] UPDATE_USER_STAT_REQUEST
- [ ] DELETE_USER_STATS_REQUEST
- [ ] GET_GLOBAL_STATS_REQUEST
- [x] GET_USER_ACHIEVEMENTS_REQUEST
- [x] UNLOCK_USER_ACHIEVEMENT_REQUEST
- [ ] CLEAR_USER_ACHIEVEMENT_REQUEST
- [ ] DELETE_USER_ACHIEVEMENTS_REQUEST
- [ ] GET_LEADERBOARDS_REQUEST
- [ ] GET_LEADERBOARD_ENTRIES_GLOBAL_REQUEST
- [ ] GET_LEADERBOARD_ENTRIES_AROUND_USER_REQUEST
- [ ] GET_LEADERBOARD_ENTRIES_FOR_USERS_REQUEST
- [ ] SET_LEADERBOARD_SCORE_REQUEST
- [ ] AUTH_STATE_CHANGE_NOTIFICATION
- [ ] GET_LEADERBOARDS_BY_KEY_REQUEST
- [ ] CREATE_LEADERBOARD_REQUEST
- [ ] GET_USER_TIME_PLAYED_REQUEST
- [ ] SHARE_FILE_REQUEST
- [ ] START_GAME_SESSION_REQUEST
- [ ] OVERLAY_STATE_CHANGE_NOTIFICATION
- [ ] CONFIGURE_ENVIRONMENT_REQUEST


## How to use

Currently service supports small amount of calls, but these are enough to play Gwent for example.

First, you neeed to obtain data about account `access_token`, `refresh_token` and `user_id` 

(for Heroic these can be found in `.config/heroic/gog_store/config.json`)

### Dependencies 
Currently the only dependency is python protocolbuffers

```sh
pip install protobuf
```
Alternatively you can install it using your Linux distro's package manager

### Running
```
./bin/comet --token "<access_token>" --refresh_token "<refresh_token>" --user-id <user_id>
```


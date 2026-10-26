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

### Custom events

You can also specify custom events that comet will POST to specified url.  
You can do it with top-level `custom_event` array


```toml
[[custom_event]]
enabled = true
url = "http://127.0.0.1:9970/my_cool_event?my_cool_url_param=my_cool_value"
```

The payload is a JSON structure in the following format

```json
{
    "name": "event_type",
    "client_id": "APP_CLIENT_ID",
    "data": {
        /* EVENT SPECIFIC DATA */
    }
}
```

At the moment comet supports following events.

| Name | Data structure |
| ---- | ------- |
| achievement_unlocked | [Achievement](https://github.com/imLinguin/comet/tree/78243e8c69a40acd16607f3b8c41047a84cd843b/src/api/gog/achievements.rs#L9-L22) |

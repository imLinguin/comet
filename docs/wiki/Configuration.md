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
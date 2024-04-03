# Using with the Steam Deck

Using Comet with Valve's Steam Deck (running SteamOS) is possible in both Desktop and Game Mode. Comet will only function with games that do support [GOG Galaxy's achievement system](https://www.gog.com/en/games?features=achievements). **If your game does not work out of the box, check Known Issues below.**

Using `comet_shortcut.sh` will simplify the process of launching Heroic Games Launcher generated non-Steam game shortcuts with Comet in the background. That script works in both Modes.

## How to Use
1. Install Comet and its shortcut script. (See the installation steps below.)
2. Change the Heroic shortcuts on Steam to include a reference to the shortcut script.
3. Launch the game through either Desktop or Game Mode!
4. **When stopping your game**: close the game by using the Steam button to get to the overlay, selecting the window of your running game and close it with the X button. **Directly exiting the game through Steam will not sync your GOG playtime via Heroic!** After the game's closed through the mentioned "close window" method, you can simply close the remaining Comet window through the same manner or by selecting the "Exit game" option through Steam itself.

## Known Issues

- **Not all GOG achievement games are supported** - some games (e.g. [Cuphead](https://www.gog.com/game/cuphead)) do not support the way Comet currently works on its own, due to an outdated SDK used for GOG Galaxy features. 
  
  **To solve it**: you will need to install the `GalaxyCommunications` dummy application. (For Steam Deck users, the necessary files (the `.bat` script and the dummy `.exe`) have been included in the Linux artifact.)

  1. Keep the `comet` Linux artifact items in a directory Heroic has access to, such as `~/Documents` or `~/Desktop`.
  2. Go to Heroic Games Launcher, to the malfunctioning game's settings screen..
  3. Scroll down the WINE tab of the game's settings screen until you see `RUN EXE ON PREFIX`.
  4. Drag and drop the `install-dummy-service.bat` onto `RUN EXE ON PREFIX` to install the dummy service for the game to detect.
  5. Play the game as you would expect. It should now function with Comet's features!

## Installation steps

1. Make sure you are logged into GOG on Heroic Games Launcher.
2. Download the latest release of Comet from [the latest GitHub Actions run](https://github.com/imLinguin/comet/actions) labelled `comet-x86_64-unknown-linux-gnu`.
3. Extract the downloaded `.zip` archive to a desired place.
   > It is recommended to have the `comet` binary put into the `~/Documents` directory. Otherwise: choose any directory where Heroic has access to.
6. Mark both `comet_shortcut.sh` and `comet` as `is Executable`. (Right click on the file > Properties > Permissions tab > click on the `Is executable` checkbox.)
7. Use Heroic Games Launcher to create non-Steam game shortcuts of any of your GOG games to use with Comet.
8. Restart Steam to have the newly created shortcuts show up.
9. For each Heroic Games Launcher shortcut:
    - Move the current launch options (`run com.heroicgameslauncher.hgl ...`) into the target field, next to `flatpak`.
    - Put the following in the launch options field:
    `"<file path of comet_shortcut.sh>" %command%`
        > Replace what's between `< >` with the full file path (e.g. `~/Documents/comet/comet_shortcut.sh`)
10. Open the `comet_shortcut.sh` file with Kate (right click on the file > Open with Kate), and edit the following values:
    - `gog_username`
        > Change the `username` value after `=` to your GOG username
    - `path_to_comet`
        > Change the `path_to_comet` value after `=` (while keeping the `'` characters in tact) to the full file path of the `comet` binary.
11. Start any game shortcut that has the shortcut script included (see step 9.) in its launch options to play the game with achievement support!

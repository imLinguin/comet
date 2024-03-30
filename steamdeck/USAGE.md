# Using with the Steam Deck

Using Comet with Valve's Steam Deck (running SteamOS) is possible in both Desktop and Game Mode. Comet will only function with games that do support [GOG Galaxy's achievement system](https://www.gog.com/en/games?features=achievements). **If your game does not work out of the box, check Known Issues below.**

Using `comet_shortcut.sh` will simplify the process of launching Heroic Games Launcher generated non-Steam game shortcuts with Comet in the background. That script works in both Modes.

## How to Use
1. Install Comet and its shortcut script. (See the installation steps below.)
2. Change the Heroic shortcuts on Steam to include a reference to the shortcut script.
3. Launch the game through either Desktop or Game Mode!
4. **When stopping your game**: close the game by using the Steam button to get to the overlay, selecting the window of your running game and close it with the X button. **Directly exiting the game through Steam will not sync your GOG playtime via Heroic!** After the game's closed through the mentioned "close window" method, you can simply close the remaining Comet window through the same manner or by selecting the "Exit game" option through Steam itself.

## Known Issues

- **Not all GOG achievement games are supported** - some games (e.g. [Cuphead](https://www.gog.com/en/game/cuphead)) do not support the way Comet currently works on its own, due to an outdated SDK used for GOG Galaxy features. 
  
  **To solve it**: you will need to install the `GalaxyCommunications` dummy application. (For Steam Deck users, the necessary files (the `.bat` script and the `pfx` folder) have been included in the Linux artifact.)

  1. Grab the GalaxyCommunication dummy artifact from the latest GitHub Actions run.
  2. Extract the files to any place, for easy copy-pasting.
  3. Go to Heroic, to the malfunctioning game's settings screen.
  4. Using the file explorer (Dolphin), go to the game's WinePrefix folder, mentioned in the WINE tab on the Heroic game's settings page.
  5. Copy the `pfx` folder of the Galaxy Communications artifact/`.zip` file to it. Accept "writing into the directory". (This will place the GalaxyCommunications dummy program in the correct place for you.)
  6. Scroll down the WINE tab of the game's settings screen until you see `RUN EXE ON PREFIX`.
  7. Drag and drop the `install-dummy-service.bat` onto `RUN EXE ON PREFIX` to install the dummy service for the game to detect.
  8. Play the game as you would expect. It should now function with Comet's features!

## Installation steps

1. Make sure you are logged into GOG on Heroic Games Launcher.
2. Download the latest release of Comet from [the latest GitHub Actions run](https://github.com/imLinguin/comet/actions) labelled `comet-x86_64-unknown-linux-gnu`.
3. Extract the downloaded `.zip` archive to a desired place.
   > It is recommended to have the `comet` binary put into the `/home/deck/` directory, as that is the default file path in the script. You are free to change it to a different install location, however. This does include on external storage such as a microSD card inserted in the Steam Deck.
4. Download `comet_shortcut.sh` from the GitHub repository. (Click on the file, then on the icon with the arrow pointing downwards into the bracket.)
5. Put the `comet_shortcut.sh` file in any location.
   > It is recommended to put it in the same directory as the `comet` binary.
6. Mark both `comet_shortcut.sh` and `comet` as `is Executable`. (Right click on the file > Properties > Permissions tab > click on the `Is executable` checkbox.)
7. Use Heroic Games Launcher to create non-Steam game shortcuts of any of your GOG games to use with Comet.
8. Restart Steam to have the newly created shortcuts show up.
9. For each Heroic Games Launcher shortcut:
    - Move the current launch options (`run com.heroicgameslauncher.hgl ...`) into the target field, next to `flatpak`.
    - Put the following in the launch options field:
    `"/home/deck/comet_shortcut.sh" %ccommand%`
        > If you put the `comet_shortcut.sh` file in a different location: replace `/home/deck/comet_shortcut.sh` with the full file path of `comet_shortcut.sh`.
10. Open the `comet_shortcut.sh` file with Kate (right click on the file > Open with Kate), and edit the following values:
    - `gog_username`
        > Change the `username` value after `=` to your GOG username
    - `path_to_comet`
        > Change the `path_to_comet` value after `=` (while keeping the `'` characters in tact) to the full file path of the `comet` binary.
11. Start any game shortcut that has the shortcut script included (see step 9.) in its launch options to play the game with achievement support!

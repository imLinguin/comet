# Game Compatibility

As this is a work-in-progress replacement, not all GOG Galaxy feature enabled games may function as expected. You can use `CTRL+F` to search for the game you would like to learn more about its compatibility.

## Contribute

To contribute, follow the Contribution Guidelines mentioned in the [Home Page of the Wiki](Home) and follow the following steps:

<details>
<summary>Steps</summary>

- Copy the last column of the table
- Edit the following information for your tested game compatibility:
  - Game Title (with the GOG storefront linked to it as a Hyperlink, in the following format: `[Game's name](GOG url here)`)
  - Game Version
    - To check the official Game Version name, either find it in-game or follow the following steps on Heroic:
      1. Go to the game's page on Heroic Games Launcher
      2. Hover over the three-point menu button and click on `Modify Installation`
      3. Click on the checkbox next to `Keep the game at specific version`
      4. The selected version should be the one you have currently installed. Note the version and date of said version as Game Version.
  - Comet Version
    - Go by the version name in the Releases tab, of the version you downloaded.
    - Did you use Comet when it had no Releases available, or are you using a build that's not part of the releases? Mention (and possibly link) to the commit of the version you used.
  - GalaxyCommunication.exe Service Required
    - Test if the game does function with achievements and leaderboards/Comet. If it does not, try to install the service as you can [read in the dummy service documentation]](https://github.com/imLinguin/comet/tree/main/dummy-service/README.md).
    - If the service is required, fill in with `🟩 Yes`. If the service is not required, fill in with `🔲 No`.
  - GOG Galaxy Features
    - Fill in the `🟩` (working), `🔲` (not present in-game) or `❌` (not working) for the following features:
      - Achievements
      - Leaderboard
  - Notes
    - Any additional notes you would find important to mention with regards to the game compatibility. For example: possible issues, workarounds, specifics like switchable leaderboards between GOG and a different service.
</details>

## Game Compatibility Table

### Legend


| Icon | Meaning |
|---|---------------------------------------------------------------------------------------|
| 🟩 | Required (GalaxyCommunication.exe requirement), Working (GOG Galaxy Features)         |
| 🔲 | Not Required (GalaxyCommunication.exe requirement), Not Present (GOG Galxay Features) |
| ❌ | Not Working (GOG Galaxy Features)                                                     |

### Table

| Game Title  | Game Version | Comet Version  | GalaxyCommunication.exe Service Required  | GOG Galaxy Features | Notes |
| ------------- | ------------- | ------------- | ------------- | ------------- | ------------- |
| [Absolute Drift](https://www.gog.com/en/game/absolute_drift)  | Version 5f6049d (6/26/2023)  | [commit `55e4025`](https://github.com/imLinguin/comet/commit/55e402538df3bff354bf2e1e9a54fa4e5e091122)  | 🔲 No | 🟩 Achievements ❌ Leaderboard | Achievements do work. Leaderboard support incomplete; errors out on retrieving and updating stats from GOG Galaxy servers. Leaderboards should work fine when **not** using the GOG Galaxy leaderboards. **You can change the leaderboards in Settings > Your Data Rights.**  |
| [Cuphead](https://www.gog.com/game/cuphead)  | Version 1.3.4 (8/19/2022)  | [commit `55e4025`](https://github.com/imLinguin/comet/commit/55e402538df3bff354bf2e1e9a54fa4e5e091122) | 🟩 Yes | 🟩 Achievements 🔲 Leaderboard | GalaxyCommunication.exe service required for game to start communicating with GOG. Otherwise, Achievements won't work. No Leaderboards present in-game. |

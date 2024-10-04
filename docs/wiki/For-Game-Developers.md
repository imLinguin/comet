> [!IMPORTANT]
> This is **not** an official page affiliated with GOG. These are my findings while I was looking into SDK client libraries that GOG provides.  
> While the libraries disclosed below have proven to work in production there **no one guarantees** that.  
> Nevertheless I encourage you to at least try linking your game against and see the results

## You made it here!

If you are seeing this page then you are probably considering shipping a Linux native build of your game on GOG.  
Excelent choice! If you've researched the topic of GOG Galaxy SDK then you probably know that GOG claims such thing is not supported on Linux.

**But, don't leave yet** while this is what GOG claims, it's not necessairly true (I guess they don't want to commit to supporting Linux fully).  
It was proven to work by games like

- Stardew Valley
- Indivisible
- Streets of Rage 4
- Crypt of the NecroDancer

## How to get native library

In the developer portal in `Galaxy Components` > `SDK` you'll see the section with downloads

![image](https://github.com/user-attachments/assets/3bdc9728-ad09-4bd4-83c4-09c745433a8a)

The `Steam-runtime` is what you are looking for. Pick an appropriate bitness and link your app against that.
This build is up-to date like the rest of SDK packages.

> [!NOTE]
> There is no GalaxyPeer library for Linux, it seems to be statically linked within the provided libGalaxy.so library.


## Why bother?

Going out of your way to use the library that GOG is too shy (⁄ ⁄>⁄ ▽ ⁄<⁄ ⁄) to talk about may feel weird but there are advantages.

1. DLC Discovery https://docs.gog.com/sdk-dlc-discovery
   - GOG encourages you to build your own solution for DLC Discovery on Linux, because normally this is something that SDK client handles

2. Multiplayer
   - Having SDK client library is required to be able to provide multiplayer functionality and user identity features. (this still has to be optional - remember about the [DRM free nature](https://docs.gog.com/sdk/#drm-and-sdk) of GOG)

3. Less work (potentially)
   - It can be tedious to support multiple store fronts especially when one of them requires you to do platform specific workarounds because they fail to stay consitent with provided feature set.


## What this comet repo has to do with all this?

Comet is a drop-in replacement that allows players to leverage "GOG Galaxy-only" functionality (multiplayer, achievements, stats and leaderboards) on every platform.

The main goal is to bring this feature set to Linux users and let them use the platform to the fullest.

The project already ships to thousands of users through [Heroic Games Launcher](https://heroicgameslauncher.com/) and soon more Linux clients will follow.


## Not convinced?

Depending on features your game provides not using this lib may be more troublesome than expected, you should evaluate this per your project scope.

However, even if you still don't want to do it, your decision is respected. After all Linux users who will want to access those online features can still use Proton to run Windows build of your game, so not everything is lost. 


*Have a nice day!*  
*from Comet developers*
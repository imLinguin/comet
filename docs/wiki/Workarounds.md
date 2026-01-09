> [!NOTE]
> This functionality is not shipped as a release yet. It's available in nightly build from main branch CI.

## What are workarounds?

Workarounds aim to provide fixes for games with bad GOG implementation.
At the moment comet supports the following types of workarounds:
- Cumulative achievements

## Cumulative achievements

This is a feature that Steam provides but GOG doesn't. 
Where it's possible to link a `Progress Stat` to the Achievement, allowing for said achievement to unlock automatically upon reaching certain threshold on the stat.

In order to specify the workaround for given game you need to create the file in proper locations.

Workarounds directory locations:

- Windows - `%LOCALAPPDATA%/comet/workarounds`
- Mac - `~/Library/Application Support/comet/workarounds`
- Linux - `$XDG_DATA_HOME/comet/workarounds`

You will neeed to create a text file named `<CLIENT_ID>.progress`
You can obtain the CLIENT_ID from build information on gogdb.org

The contents should follow this schema

```
<ACHIEVEMENT_KEY> <STAT_KEY> <THRESHOLD>
```

e.g for Dark Sky - `57131431910270832.progress`

```
An_Electric_Touch Static_Damage_Dealt 250
Bonebreaker Frail_Count 20
Combustion_Expert Ignite_Reaction_Count 20
Contagion Sick_Count 20
Defibrillation_Expert Short_Circuit_Count 20
Unlimited_Power Boosted_Count 10
Spa_Month Relaxed_Count 10
Sitting_Ducks Exposed_Count 20
Playing_With_Fire Burn_Damage_Dealt 250
```

This schema is based on GOG's Steam SDK Wrapper achievement file - which is an undocumented feature, but is often used in pair with games.
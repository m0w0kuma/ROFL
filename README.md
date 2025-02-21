# ROFL 
ROFL is a tool for parsing and extracting information from League of Legends replay file(.rofl).
## Why ROFL?
Riot Games offers minimal game data through their APIs, often lacking critical granular details. For example:
- Player positions are only available on a 1-minute interval basis, making it impossible to accurately track player movement patterns throughout the game.
- No information about wards on the map is provided.

Analysis like [that](https://www.reddit.com/r/leagueoflegends/comments/1i0ky00/soloq_ward_heatmap/), for example, is not possible using riot API.

I have some personal objectives with this project: 
- Attempt to statistically determine the optimal jungle path for a given champion.
- Analyze movement patterns and their correlation with game outcomes. For instance, explore whether winning teams maintain higher jungle-support proximity.

## Features
Right now, we can: 
  - Extract champions position(x, y) in intervals of one sec.
  - Extract placed wards information:
    - Duration
    - Position
    - Type
    - Owner role
    - Team
  - In the future(soon), extract jungle camps information for pathing inference. 
## Quickstart
Download the .zip file in release section.

## Usage
To parse a single file:
```console
./ROFL.exe file -r /path/to/replay_file.rofl -o /path/to/output_file.json
```  
Example:
  ![cli](https://github.com/user-attachments/assets/068a1880-4145-4000-977f-e612f0670b35)


## Output File
This is the truncated version of the .json output of a random game:
```javascript
{
    "metadata": {
        "game_len": 1386200,
        "players": [
            {
                "name": "",
                "position": "Top",
                "skin": "Poppy",
                "team": "Blue"
            },
            {
                "name": "",
                "position": "Jungle",
                "skin": "MasterYi",
                "team": "Blue"
            },
            {
                "name": "",
                "position": "Mid",
                "skin": "Azir",
                "team": "Blue"
            },
            {
                "name": "",
                "position": "Adc",
                "skin": "Ezreal",
                "team": "Blue"
            },
            {
                "name": "",
                "position": "Support",
                "skin": "Maokai",
                "team": "Blue"
            },
            {
                "name": "",
                "position": "Top",
                "skin": "Shen",
                "team": "Red"
            },
            {
                "name": "",
                "position": "Jungle",
                "skin": "Sejuani",
                "team": "Red"
            },
            {
                "name": "",
                "position": "Mid",
                "skin": "Katarina",
                "team": "Red"
            },
            {
                "name": "",
                "position": "Adc",
                "skin": "MissFortune",
                "team": "Red"
            },
            {
                "name": "",
                "position": "Support",
                "skin": "Nautilus",
                "team": "Red"
            }
        ],
        "version": "5.4.",
        "winning_team": "Red"
    },
    "players_state": [
        {
            #                  ...
            #                  ...
            #                  ...
            "players": [
                {
                    "champ": "Poppy",
                    "name": "",
                    "pos": [
                        1002.0,
                        4088.0
                    ],
                    "role": "Top",
                    "team": "Blue"
                },
                {
                    "champ": "MasterYi",
                    "name": "",
                    "pos": [
                        2372.0,
                        3086.0
                    ],
                    "role": "Jungle",
                    "team": "Blue"
                },
                {
                    "champ": "Azir",
                    "name": "",
                    "pos": [
                        2425.5,
                        2734.7
                    ],
                    "role": "Mid",
                    "team": "Blue"
                },
                {
                    "champ": "Ezreal",
                    "name": "",
                    "pos": [
                        3350.0,
                        1438.0
                    ],
                    "role": "Adc",
                    "team": "Blue"
                },
                {
                    "champ": "Maokai",
                    "name": "",
                    "pos": [
                        3106.0,
                        2092.0
                    ],
                    "role": "Support",
                    "team": "Blue"
                },
                {
                    "champ": "Shen",
                    "name": "",
                    "pos": [
                        12975.7,
                        11532.8
                    ],
                    "role": "Top",
                    "team": "Red"
                },
                {
                    "champ": "Sejuani",
                    "name": "",
                    "pos": [
                        13950.0,
                        14130.0
                    ],
                    "role": "Jungle",
                    "team": "Red"
                },
                {
                    "champ": "Katarina",
                    "name": "",
                    "pos": [
                        12150.1,
                        11574.2
                    ],
                    "role": "Mid",
                    "team": "Red"
                },
                {
                    "champ": "MissFortune",
                    "name": "",
                    "pos": [
                        12574.3,
                        10876.9
                    ],
                    "role": "Adc",
                    "team": "Red"
                },
                {
                    "champ": "Nautilus",
                    "name": "",
                    "pos": [
                        12006.0,
                        11434.0
                    ],
                    "role": "Support",
                    "team": "Red"
                }
            ],
            "timestamp": 18.97
        }
        #                  ...
        #                  ...
        #                  ...
    ],
    "wards": [
        #                  ...
        #                  ...
        #                  ...
        {
            "duration": 90.23948669433594,
            "name": "YellowTrinket",
            "owner": {
                "name": "",
                "role": "Jungle",
                "team": "Blue"
            },
            "pos": [
                7506,
                9834
            ],
            "team": "Blue",
            "timestamp": 46.60076904296875
        },
        {
            "duration": 90.12826538085938,
            "name": "YellowTrinket",
            "owner": {
                "name": "",
                "role": "Jungle",
                "team": "Red"
            },
            "pos": [
                6290,
                10064
            ],
            "team": "Red",
            "timestamp": 68.35939025878906
        }
        #                  ...
        #                  ...
        #                  ...
    ]
}
```

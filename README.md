# ROFL 
ROFL is a tool for parsing and extracting information from League of Legends replay file(.rofl).
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

OBS: For ROFL to work we need a .patch file that contains patch informations, i will try my best to stay up to date and provide these for every upcoming patch. 
You can find the .patch files in the zip file in release section.
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
"metadata": {
    "game_len": 2192410,
    "players": [
        {
            "name": "ta d borest",
            "position": "Top",
            "skin": "Irelia",
            "team": "Blue"
        },
        {
            "name": "Vinicete",
            "position": "Jungle",
            "skin": "Karthus",
            "team": "Blue"
        },
        {
            "name": "",
            "position": "Mid",
            "skin": "Ahri",
            "team": "Blue"
        },
        {
            "name": "Trigo 11",
            "position": "Adc",
            "skin": "Ashe",
            "team": "Blue"
        },
        {
            "name": "Telaszz",
            "position": "Support",
            "skin": "Rell",
            "team": "Blue"
        },
        {
            "name": "goyangyi",
            "position": "Top",
            "skin": "Kennen",
            "team": "Red"
        },
        {
            "name": "ai calica24",
            "position": "Jungle",
            "skin": "Kindred",
            "team": "Red"
        },
        {
            "name": "Bionic",
            "position": "Mid",
            "skin": "Swain",
            "team": "Red"
        },
        {
            "name": "tinowns01",
            "position": "Adc",
            "skin": "Corki",
            "team": "Red"
        },
        {
            "name": "Moon NE WC3",
            "position": "Support",
            "skin": "Zac",
            "team": "Red"
        }
    ],
    "version": "5.1.",
    "winning_team": "Blue"
},
"players": {
         "Blue": {
            "Top": [
                #                  ...
                #                  ...
                #                  ...
                { "pos": [100.0, 200.0], "timestamp": 5.0 },
                #                  ...
                #                  ...
                #                  ...
            ],
            "Jungle": [
                #                  ...
                #                  ...
                #                  ...
                { "pos": [700.0, 800.0], "timestamp": 18.0 }
                #                  ...
                #                  ...
                #                  ...
            ],
            "Mid": [
                #                  ...
                #                  ...
                #                  ...
                { "pos": [1100.0, 1200.0], "timestamp": 22.0 }
                #                  ...
                #                  ...
                #                  ...
            ],
            "Adc": [
                #                  ...
                #                  ...
                #                  ...
                { "pos": [5492.9, 5648.7], "timestamp": 25.613 }
                #                  ...
                #                  ...
                #                  ...
            ],
            "Support": [
                #                  ...
                #                  ...
                #                  ...
                { "pos": [1500.0, 1600.0], "timestamp": 21.0 }
                #                  ...
                #                  ...
                #                  ...
            ]
        },
        "Red": {
            "Top": [
                #                  ...
                #                  ...
                #                  ...
                { "pos": [400.0, 500.0], "timestamp": 16.0 }
                #                  ...
                #                  ...
                #                  ...
            ],
            "Jungle": [
                #                  ...
                #                  ...
                #                  ...
                { "pos": [800.0, 900.0], "timestamp": 19.0 }
                #                  ...
                #                  ...
                #                  ...
            ],
            "Mid": [
                #                  ...
                #                  ...
                #                  ...
                { "pos": [1200.0, 1300.0], "timestamp": 23.0 }
                #                  ...
                #                  ...
                #                  ...
            ],
            "Adc": [
                #                  ...
                #                  ...
                #                  ...
                { "pos": [5492.9, 5648.7], "timestamp": 26.0 }
                #                  ...
                #                  ...
                #                  ...
            ],
            "Support": [
                #                  ...
                #                  ...
                #                  ...
                { "pos": [1600.0, 1700.0], "timestamp": 22.0 }
                #                  ...
                #                  ...
                #                  ...
            ]
        }
    },
"wards": [
        #                  ...
        #                  ...
        #                  ...
        {
            "duration": 123.147705078125,
            "name": "SightWard",
            "owner_role": "Support",
            "pos": [
                6314,
                10104
            ],
            "team": "Blue",
            "timestamp": 1893.936767578125
        },
        {
            "duration": 1.001953125,
            "name": "SightWard",
            "owner_role": "Mid",
            "pos": [
                6328,
                8372
            ],
            "team": "Red",
            "timestamp": 2018.554443359375
        }
        #                  ...
        #                  ...
        #                  ...
],
}
```

# ROFL 
ROFL is a tool for parsing and extracting information from League of Legends replay(.rofl).
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
## Information
For ROFL to work we need a .patch file that contains patch informations, i will try my best to stay up to date and provide these every patch. 
The files can be found in .zip in the release section.
## Usage
To parse a single file:
-  ./ROFL.exe -p /path/to/patch_file.patch file -r /path/to/replay_file.rofl -o /path/to/output_file.json
  
To parse various files in a folder:
-  ./ROFL.exe -p /path/to/patch_file.patch folder -r /path/to/replay_folder -o /path/to/output_folder 

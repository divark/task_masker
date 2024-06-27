# Purpose
Task Masker is an application built to engage with a live audience on a platform (Only Twitch so far) through the medium of a game. The interactions of the community are mapped to behaviors that happen in the game automatically, such as an NPC spawning to represent some chatter and approaching the Streamer's character to speak.

# Features
## Current
- A Chatter and Subscriber is represented by NPCs on the map.
- The Streamer is represented as a playable character triggered to move to specific spots via manual input.
- Donations and Subscriptions serve as triggers to one or more entities listening for them.
- A Chat message is displayed as a pop-up dialogue when received, changing the portrait depending on who is speaking.
- The Playable Character and all NPCs are capable of traveling to any point on a map containing Ground and Air tiles.
- Tiled is the only supported map type.
- Background music plays in a loop, randomly choosing the next track randomly.

## Future
- Portrait Preferences are loaded and saved for some Subscriber, influencing the NPC's Sprite both in-game, and when chatting.
- Twitch events influence what gets triggered, moved, etc.
- The Streamer's character moves automatically to specific spots based on what processes are running in the background.

# Caveats
- The source code alone is open source through the GPLv3 license, but not the assets. In order to honor the different licenses per asset, I've opted to not include assets in the repository at all.
    - However, all assets have been referenced as to where to download them yourself in each folder's README.md.
- This is not intended for a general audience, only serving my needs to get started with streaming on Twitch for now.
    - In the far future, I would be happy to make a tool similar to Mario Maker to setup your own Task Masker environment, but that will not happen for a while as I figure out what works at all.

# Building
With or without the assets, Task Masker is designed to be testable without them, only focusing on the non-visual logic. To check the current state of the project,
1. Download Rust if you have not already.
2. Clone this repository.
3. In a terminal, run `cargo test` to check which features are implemented and working as intended.

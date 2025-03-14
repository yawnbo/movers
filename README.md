# Webscraper to watch movies on (WIP)
Very basic and functioning on sticks project that is planned to be worked on, nothing is final or formal and the code is a mess that should be cleaned eventually.

# Installation
Currently only linux and macos are tested and supported as I have no clue how to get it to work on windows but am open for implementing it.

## NOT RECOMMENDED 

Install with cargo (not stable and will crash if a series shows up in list)
```
cargo install movers
```
or from source (**RECOMMENDED**),  
1. Clone repo
```
git clone https://github.com/yawnbo/movers.git
```
2. Build with cargo
```
cd movers && cargo build --release
```
3. Run with 
```
./target/release/movers -S <MOVIE>
```

# Features
---
- Watching movies
- downloading (planned)
- Subtitles
- Series/episodes (planned)
- other players (iina, planned)
- config and arg parsing (planned)
## Inspo
Thanks to [Film-central](https://github.com/JDALab/film-central) and [MisaelVM](https://github.com/MisaelVM) for the decryption algo

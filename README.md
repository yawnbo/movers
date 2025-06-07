# NOTICE
Currently broken due to changes upstream, don't know if i will end up fixing
# Webscraper to watch movies on (WIP)
Very basic and functioning on sticks project that is planned to be worked on, nothing is final or formal and the code is a mess that should be cleaned eventually.

# Installation
Currently only linux and macos are tested and supported as I have no clue how to get it to work on windows but am open for implementing it.

## NOT RECOMMENDED 

Install with cargo (less stable most times)
```bash
cargo install movers
```
or from source (**RECOMMENDED**),  
1. Clone repo
```bash
git clone https://github.com/yawnbo/movers.git
```
2. Build with cargo
```bash
cd movers && cargo build --release
```
3. Run with 
```bash
./target/release/movers -S <MOVIE>
```

# Features
---
- Watching movies
- downloading (planned)
- Subtitles
- Series/episodes
- other players (iina, planned)
- config and arg parsing (planned)

## Dev notes on the todo (don't read this it's not important)
I don't know what happened, but it works now for some reason. IF the codec gets messed up again I think the solution will be to trim the last byte of the ts packet retrieved from the 206 response, but I also don't know how to automate this with a proxy, so it'll probably only work with downloading when implemented. 
#### Todo:
vidlink/cloudburst/stormproxy scraping (very hard)
## Inspo
Thanks to [Film-central](https://github.com/JDALab/film-central) and [MisaelVM](https://github.com/MisaelVM) for the decryption algo

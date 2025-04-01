# NOTICE 
Breaks with ffmpeg 7.1+ currently because of commit 91d96dc8ddaebe0b6cb393f672085e6bfaf15a31 to avformat/hls. This creates security checks for extensions to be of the same type and thus breaks streaming. I'll try to get a workaround working but for now streaming is somewhat working.

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

#### Todo:
vidlink/cloudburst/stormproxy scraping (very hard)
## Inspo
Thanks to [Film-central](https://github.com/JDALab/film-central) and [MisaelVM](https://github.com/MisaelVM) for the decryption algo

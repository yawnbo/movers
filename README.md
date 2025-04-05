# NOTICE 
Currently very bad quality because of (i assume) some changes on the scraped end, (yall lame for corrupting packets) but I'll try to have it fixed. In the meantime, downloading vids will be done sooner as the output is more stable. 



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
Packets need to be manually curl'd and truncated to exclude or include the last byte because I have no clue what the fuck ffmpeg is reading when it sometimes decides to work and other times doesn't even though the hashes are the FUCKING SAME!!!!!! 

Idea list
- libavformat respects content-length header and doesn't read the last byte even though it's returned
- libavformat DOES NOT respect content-length and reads the last header even though it's not needed
- curl individual 206 response packets ( I still think they only break on the partial contents as far as I've looked ) and see if they play with mpv or ffpplay
    - as a subnote to the above, a playlist can be made with these packets using the original list but with the downloaded files, where playing them can reduce uncertainty in breaking packets and if the packets are missing a byte or needing a byte (unlikely as the header)
- look at the etags and other headers between the 206 packets and SHA SUM THEM AGAIN BECAUSE I DON'T BELIEVE THEY ARE THE FUCKING SAME
    - EVERYSINGLE PACKET SHOULD BE RE HASHED BECAUSE WHY IS THIS HAPPENING?????? 

#### Todo:
vidlink/cloudburst/stormproxy scraping (very hard)
## Inspo
Thanks to [Film-central](https://github.com/JDALab/film-central) and [MisaelVM](https://github.com/MisaelVM) for the decryption algo

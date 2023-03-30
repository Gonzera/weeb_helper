# weeb_helper

I simple tool to download anime when it releases based on your anilist watching list.

It is currently a WIP and lacks a lot of features.

This is my first software with rust, so it has a lot of stuff that is wrong xD.

### Usage

```bash
# Setup config files
./weeb_helper -i

# Runs it
./weeb_helper -d
```

### Requirements

- Anilist account
- QBittorrent with webUI enable (local or remote)

### TODO

- [ ] Add quality preference
- [ ] Add release group preference
- [x] Use anilist notifications instead of the current solution
- [ ] Clean up a little bit
- [ ] Loggin stuff (priority)

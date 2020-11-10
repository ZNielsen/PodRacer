# PodRacer

[![CodeFactor](https://www.codefactor.io/repository/github/znielsen/podracer/badge)](https://www.codefactor.io/repository/github/znielsen/podracer)
[![Build Status](https://travis-ci.com/znielsen/podracer.svg?branch=main)](https://travis-ci.com/znielsen/podracer)
![GitHub repo size](https://img.shields.io/github/repo-size/znielsen/podracer)
![GitHub contributors](https://img.shields.io/github/contributors/znielsen/podracer)
![GitHub stars](https://img.shields.io/github/stars/znielsen/podracer?style=social)
![GitHub forks](https://img.shields.io/github/forks/znielsen/podracer?style=social)


A podcast catch-up service running on your very own server!

## Features
- *Self hosted* - complete autonomy over your feeds
- *Time shift podcasts* - Have a favorite show that died out and want to relive it? PodRacer lets you relive the podcast by creating a feed where the first episode was published _today_.
- *Variable publishing rate* - Do you want to catch up on serial podcast? Set the rate to be > 1x and PodRacer will scale the shifted publish dates, letting you slowly (or quickly) catch up to real time.

# WORK IN PROGRESS - still very buggy. It's functional but not polished yet

Tests were done against [Overcast](https://apps.apple.com/us/app/overcast/id888422857) ([info](https://overcast.fm/podcasterinfo)). Other podcast players might have issues I didn't see, feel free to file a github issue and I'll do what I can to fix it up.

## Installation

I run this server on a Raspberry Pi 3.

Grab the binary from the github releases
```
wget https://github.com/znielsen/PodRacer/releases/newest
```

Alternatively, you can clone and build. You will need the rust nightly build for rocket.
```
rustup override set nightly && rustup update && cargo update && cargo build --release
```
It's going to take a very long time to build. That's Rust.

### Start the server
```
cargo run --release
```
Sometimes (not quite sure why only sometimes) my Pi3 struggles to build when multiple threads are enabled (the default). You can `cargo clean && cargo build -j 1` to force building with only one thread. This will take way longer, but you don't run out of memory.

If you want to access the server remotely, set up port forwarding in your router.

### Keeping the process alive
You can either run in a tmux instance or use `nohup` and send it to the background.

## Configuration

## Usage
You can communicate with the server via HTTP methods (mostly just `POST` and `GET`). The included `create_feed.sh` bash script makes a `curl` request to the server, you can customize the arguments via command line, or change up the following examples (assuming your server is at `0.0.0.0`, listening on port `1234`)

### Create a new PodRacer feed
- POST to 0.0.0.0:1234/create_feed
  - parameters:
    - url [string] - The actual RSS feed for the podcast.
    - rate [float] - Used to scale the time between episodes.
        For a weekly podcast, a rate of 2.0 will give episodes every 3.5 days. A rate of 1.2 will give episodes roughly every 6 days. A rate of 1.0 will just time shift the podcast as if the first episode was published today.
    - integrate_new [bool] - Should PodRacer check the actual RSS feed for updates, or should it just stick with the current backlog?
        Set to false if you plan on listening contemporaneously, but also want to work through the backlog.
        Set to true if you want to listen to all the episodes in order, eventually catching up to real time. Once you are caught up, you can either unsubscribe from the PodRacer feed and subscribe to the 'real' feed, or just leave it - as long as PodRacer is running, it will continue to update the feed.
    - start_ep [int] - The episode number to start on.
        This episode will appear to come out the day you create the feed. All previous episodes will appear published as well.
        Note this argument selects from the number of episodes published in the feed, and may not match the publisher's self reported numbering.
  - example call:
    ```bash
    curl -X POST -G \
        --data-urlencode "url=http://example.com" \
        --data-urlencode "rate=1.2" \
        --data-urlencode "integrate_new=false" \
        ${hostname}:${port}/${slug}
    ```

### Force update of the specified podcast
- POST to 0.0.0.0:1234/update/<url>
  - parameters:
    - url [string] -

### Force update of all podcasts on this server
- POST to 0.0.0.0:1234/update
  - parameters: none

### Delete an existing PodRacer feed
- POST to 0.0.0.0:1234/delete_feed
  - parameters:
    - url [string] - The PodRacer RSS feed to delete from the server.
        This will permanently delete the PodRacer feed, so use with caution.
    - example call:
        TODO - add example call

### List all the PodRacer feeds on this server
- GET to 0.0.0.0:1234/list_feeds
  - parameters: none
  - example call:

## Contributing

### License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

### Donating
If you like it then you shoulda thrown a dollar at it.

If you found PodRacer useful, a donation would be great! Recommended $1/podcast (as in per feed run through the service, not per episode). No pressure, but supporting software you like is cool.
![Paypal Donate](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=Y8HPAAJZTVT8E&item_name=Developer+Tip&currency_code=USD)

### Maintainers
[Zach Nielsen](https://github.com/ZNielsen) - @ZNielsen

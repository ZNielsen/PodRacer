# PodRacer

[![CodeFactor](https://www.codefactor.io/repository/github/znielsen/podracer/badge)](https://www.codefactor.io/repository/github/znielsen/podracer)
[![Build Status](https://travis-ci.com/znielsen/podracer.svg?branch=main)](https://travis-ci.com/znielsen/podracer)
![GitHub repo size](https://img.shields.io/github/repo-size/znielsen/podracer)
![GitHub contributors](https://img.shields.io/github/contributors/znielsen/podracer)
![GitHub stars](https://img.shields.io/github/stars/znielsen/podracer?style=social)
![GitHub forks](https://img.shields.io/github/forks/znielsen/podracer?style=social)


A podcast catch-up service running on your very own server!

I'm running one at [podracer.zachn.me](http://podracer.zachn.me), feel free to take the service for a spin. I'm planning on keeping this up and available for free, but I'm not making any promises until this project becomes a bit more stable.

## Features
- **Time shift podcasts** - Have a favorite show that died out and want to relive it? PodRacer lets you experience it anew by creating a feed where the first episode was published _today_.
- **Variable publishing rate** - Do you want to catch up on serial podcast? Set the rate to be > 1x and PodRacer will scale the shifted publish dates, letting you slowly (or quickly) catch up to real time. Podcasts coming at you too fast? Set the rate to be < 1x to make a bi-weekly show weekly, or a weekly show bi-weekly.
- **Integrates new episodes** - Once you catch up, PodRacer integrates the new episodes as they are published, seamlessly transferring you over to the "normal" listening experience.
- **Self hosted** - complete autonomy over your feeds.

PodRacer is most useful if you prefer to listen to your podcasts as they are published (i.e. they are dropped into a feed, and you just lisen in order). It lets legacy content seamlessly fit into the stream of contemporary content.


Tests were done against [Overcast](https://apps.apple.com/us/app/overcast/id888422857) ([info](https://overcast.fm/podcasterinfo)). Other podcast players might have quirks I didn't see. Feel free to file a github issue and I'll do what I can to fix it up.

## Installation

I run this server on a Raspberry Pi 3.

You will need the rust nightly build for rocket.
```
rustup override set nightly && rustup update && cargo update && cargo build --release
```
It's going to take a very long time to build. That's Rust.

Sometimes (not quite sure why only sometimes) my Pi3 struggles to build when multiple threads are enabled (the default). You can `cargo clean && cargo build --release -j 1` to force building with only one thread. This will take way longer, but you don't run out of memory.

### Start the server
```
cargo run --release
```

Podcast clients will need to access the server remotely, so you will have to set up [port forwarding](https://www.howtogeek.com/66214/how-to-forward-ports-on-your-router/) in your router.

### Keeping the process alive
You can either run in a tmux instance or use `nohup` and send it to the background.

## Usage
Beyond the web UI, you can communicate with the server via HTTP methods (mostly just `POST` and `GET`). The included `create_feed.sh` bash script makes a `curl` request to the server, you can customize the arguments via command line, or change up the following examples (assuming your server is at `0.0.0.0`, listening on port `1234`)

### Create a new PodRacer feed
- POST to 0.0.0.0:1234/create_feed
  - parameters:
    - url [string] - The actual RSS feed for the podcast.
    - rate [float] - Used to scale the time between episodes.
        For a weekly podcast, a rate of 2.0 will give episodes every 3.5 days. A rate of 1.2 will give episodes roughly every 6 days. A rate of 1.0 will just time shift the podcast as if the first episode was published today.
    - start_ep [int] - The episode number to start on.
        This episode will appear to come out the day you create the feed. All previous episodes will appear published as well.
        Note this argument selects from the number of episodes published in the feed, and may not match the publisher's self reported numbering.
        I have also noticed some podcast players take a bit to organize the "back catalog" correctly, so episodes may appear out of order for a bit. In my experience it resolves in an hour or so, which I assume correlates with the player's next "update feed" call.
  - example call:
    ```bash
    curl -X POST -G \
        --data-urlencode "url=http://example.com" \
        --data-urlencode "rate=1.2" \
        --data-urlencode "start_ep=1" \
        ${hostname}:${port}/${slug}
    ```

### Force update of the specified podcast
- POST to 0.0.0.0:1234/update/<url>
  - parameters:
    - url [string] - Either the folder name of the podcast to update, or the racer subscribe url of the podcast to update.

### Force update of all podcasts on this server
- POST to 0.0.0.0:1234/update
  - parameters: none

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

[![paypal](https://www.paypalobjects.com/en_US/i/btn/btn_donate_SM.gif)](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=Y8HPAAJZTVT8E&currency_code=USD)



### Maintainers
[Zach Nielsen](https://github.com/ZNielsen) - @ZNielsen

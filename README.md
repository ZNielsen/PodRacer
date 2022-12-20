# PodRacer

[![CodeFactor](https://www.codefactor.io/repository/github/znielsen/podracer/badge)](https://www.codefactor.io/repository/github/znielsen/podracer)
[![Better Uptime Badge](https://betteruptime.com/status-badges/v1/monitor/6cu8.svg)](https://betteruptime.com/?utm_source=status_badge)
![GitHub repo size](https://img.shields.io/github/repo-size/znielsen/podracer)
![GitHub contributors](https://img.shields.io/github/contributors/znielsen/podracer)
![GitHub stars](https://img.shields.io/github/stars/znielsen/podracer?style=social)
![GitHub forks](https://img.shields.io/github/forks/znielsen/podracer?style=social)


A podcast catch-up service running on your very own server!

I'm running one at [podracer.zachn.me](http://podracer.zachn.me), feel free to take the service for a spin. I'm planning on keeping this up and available for free, but I'm not making any promises -- if it starts to get swamped I'll have to figure out something more scalable.

## Features
- **Time shift podcasts** - Have a favorite show that died out and want to relive it? PodRacer lets you experience it anew by creating a feed where the first episode was published _today_.
- **Two rate-control modes**
  - **Variable publishing rate** - Do you want to catch up on serial podcast? Set the rate to be > 1x and PodRacer will scale the shifted publish dates, letting you slowly (or quickly) catch up to real time. Podcasts coming at you too fast? Set the rate to be < 1x to make a bi-weekly show weekly, or a weekly show bi-weekly.
  - **Publish every X days** - For archived podcasts that have all episodes "published" on the same day, PodRacer can set a fixed amount of time between each episode, restoring the periodic publishing feel.
- **Integrates new episodes** - Once you catch up, PodRacer integrates the new episodes as they are published, seamlessly transferring you over to the "normal" listening experience.
- **Self hosted** - complete autonomy over your feeds.

PodRacer is most useful if you prefer to listen to your podcasts as they are published (i.e. they are dropped into a feed, and you just listen in order). It lets legacy content seamlessly fit into the stream of contemporary content.

Tests were done against [Overcast](https://apps.apple.com/us/app/overcast/id888422857) ([info](https://overcast.fm/podcasterinfo)). Other podcast players might have quirks I didn't see. Feel free to file a github issue and I'll do what I can to fix it up.

## Podcast Archiver
There is also a small utility tool `podarch`, used to archive all the episodes of a given feed. See `podarch -h` for usage.

## Installation

I run this service on a Raspberry Pi 3.

```
rustup update && cargo update && cargo build --release
```
It's going to take a very long time to build. That's Rust.

Sometimes (not quite sure why only sometimes) my Pi3 struggles to build when multiple threads are enabled (the default). You can `cargo clean && cargo build --release -j 1` to force building with only one thread. This will take way longer, but you don't run out of memory.

Builds go much faster on beefier hardware. Cross-compile by installing `cross` (via `cargo install cross`), then building for the Pi:
```
cross build --release --target=armv7-unknown-linux-gnueabihf --features vendored-openssl
```
The binary will end up in `target/armv7-unknown-linux-gnueabihf/release`.

### Start the server
```
cargo run --release --bin podracer
```

Podcast clients will need to access the server remotely, so you will have to set up [port forwarding](https://www.howtogeek.com/66214/how-to-forward-ports-on-your-router/) in your router.

### Keeping the process alive
You can either run in a tmux instance or use `nohup` and send it to the background. I've set it up as a `systemd` service, but that is beyond the scope of this README.

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

# PodRacer

[![CodeFactor](https://www.codefactor.io/repository/github/znielsen/podracer/badge)](https://www.codefactor.io/repository/github/znielsen/podracer)
[![Build Status](https://travis-ci.com/znielsen/podracer.svg?branch=main)](https://travis-ci.com/znielsen/podracer)
![GitHub repo size](https://img.shields.io/github/repo-size/znielsen/podracer)
![GitHub contributors](https://img.shields.io/github/contributors/znielsen/podracer)
![GitHub stars](https://img.shields.io/github/stars/znielsen/podracer?style=social)
![GitHub forks](https://img.shields.io/github/forks/znielsen/podracer?style=social)


A podcast catch-up service

# WORK IN PROGRESS, NOT FUNCTIONAL YET

## Installation

I run this server on a Raspberry Pi 3.

Grab the binary from the github releases
```
wget https://github.com/znielsen/PodRacer/releases/newest
```

Alternatively, you can clone and build. You will need the rust nightly build for rocket.
```
rustup override set nightly && cargo build --release
```
It's going to take a very long time to build. That's Rust.

### Start the server
```
cargo run --release
```

If you want to access the server remotely, set up port forwarding in your router.

## Configuration

## Usage
You can make use of the included `create_feed.sh` bash script. It gets the options from you and makes a `curl` request to the proper address. You can communicate with the server via HTTP methods (mostly just `POST` and `GET`) at the following addresses (assuming your server is at `0.0.0.0`, listening on port `1234`)

### Create a new PodRacer feed
- POST to 0.0.0.0:1234/create_feed
  - parameters:
    - url [string] -The actual RSS feed for the podcast.
    - rate [float] - Used to scale the time between episodes.
        For a weekly podcast, a rate of 2.0 will give episodes every 3.5 days. A rate of 1.2 will give episodes roughly every 6 days. A rate of 1.0 will just time shift the podcast as if the first episode was published today.
    - integrate_new [bool] - Should PodRacer check the actual RSS feed for updates, or should it just stick with the current backlog?
        Set to false if you plan on listening contemporaneously, but also want to work through the backlog.
        Set to true if you want to listen to all the episodes in order, eventually catching up to real time. Once you are caught up, you can either unsubscribe from the PodRacer feed and subscribe to the 'real' feed, or just leave it - as long as PodRacer is running, it will continue to update the feed.
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

If you found PodRacer useful, a donation would be great! Recommended $1/podcast.

### Maintainers
[Zach Nielsen](https://github.com/ZNielsen) - @ZNielsen

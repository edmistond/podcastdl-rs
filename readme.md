# podcastdl-rs

This is a simple rust-based podcast downloader tool. The use case I had in mind when I built it was
parsing a feed and being able to select an episode (one at a time, currently) and download it for
transcription since I am deaf.

I know very little rust and built this entirely in cursor in about two hours. There is a lot of room
for improvement, but as an experiment and a useful tool it's a good start for me. I am working to learn
rust and hope to improve it over time.

## Installation

Clone the repo and run `cargo build` to install dependencies and build the binary.

## Usage

Run the binary and pass in the feed URL or local file path.

``` bash
podcastdl-rs https://example.com/feed.rss
```

You can also do `cargo run <file or feed url>` if that's more convenient. Files are downloaded to the current directory.

Once the tool starts, arrow up/down to select an entry, 'd' to download, 'q' to quit.

## Known Issues

- There is no way to cancel a download once it starts
- Ctrl-C doesn't work.

p2r (pinboard to raindrop)
---

[![Build Status](https://github.com/seth-brown/p2r/actions/workflows/build-status.yml/badge.svg)](https://github.com/seth-brown/p2r/actions/workflows/build-status.yml)

This is a simple CLI for transforming a Pinboard user's bookmarks into
[Raindrop](http://raindrop.io)'s bookmark format.

### Installation

1. Obtain a Pinboard API token from [here](https://pinboard.in/settings/password).
2. Compile the binary:

``` bash
λ cargo build --release
```

### Usage
* Help:

``` bash
λ ./target/release/p2r --help
```

* Example usage:

``` bash
λ ./target/release/p2r \
  --output=bookmarks.csv     `# raindrop formatted bookmarks` \
  --pinboard-token={TOKEN}   `#api token from above (1)` \
  --raindrop-folder=Imported `# raindrop location for bookmarks` \
  --user-tags=@pinboard      `# add tags to all imported bookmarks` \
  --clean-description        `# remove linebreaks from description field` \
```

* Upload output CSV to Raindrop: https://app.raindrop.io/settings/import

### Testing

``` bash
λ cargo test
```

p2r (pinboard to raindrop)
---

![Build Status][1]

This is a simple CLI for transforming a Pinboard user's bookmarks into
[Raindrop][2]'s bookmark format.

### Installation

1. Obtain your Pinboard account API token from [here][3].
2. Compile the binary:

``` bash
λ cargo build --release
```

### Usage
* Help:

``` bash
./target/release/p2r --help
```

* Example usage:

``` bash
./target/release/p2r \
  --output=bookmarks.csv     `# raindrop formatted bookmarks` \
  --pinboard-token={TOKEN}   `#api token from above (1)` \
  --raindrop-folder=Imported `# raindrop location for bookmarks` \
  --user-tags=@pinboard      `# add tags to all imported bookmarks` \
  --clean-description        `# remove linebreaks from description field` \
```

* uploaded output files to Raindrop: https://app.raindrop.io/settings/import

### Testing

``` bash
λ cargo test
```

[1]: https://github.com/seth-brown/p2r/workflows/Arch%20Linux%20Build%20Status/badge.svg)
[2]: http://raindrop.io
[3]: https://pinboard.in/settings/password

# Reporter-CLI

A generator of news short video

## prerequisites

- ffmpeg
- rust tool chains

## Build

```rust
cargo build --release
```

```sh
# the assets fold is need!
cp ./assets <output-dir> -r
```

## Usage

```sh
# help
reporter-cli -h
```

### get hot news list

```sh
reporter-cli list <source>
# suppert news source: pengpai, ithome
```

output:

```sh
[{"source":"ithome","title":"OPPO 陈希：有厂商因不能及时上 12306 服务被用户差评去闹，不让我们提首发","url":"https://www.ithome.com/0/877/158.htm","pics":[],"videos":[]}]
```

### generate short video

```sh
reporter-cli --rm -v <host dir>:/app/<output path> shot \
  --source pengpai \
  --url <url from list> \
  --title <title from list> \
  --pics <pics from list> \
  --videos <videos from list> \
  --author <video author> \
  --tts-url <your tts server url: https://tts.example.com> \
  --openai-url <openai url> \
  --openai-key <openai key> \
  --model <openai model> \
  --output <output path, prefer absolute>
```
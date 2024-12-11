# .PAIN Image Format 🎨

## What is this? 😱

PAIN is a deliberately inefficient image format that encrypts each pixel using SHA3-512 hashing. It's a joke format designed to make your CPU suffer.

## Features ✨

-   🐌 Extremely slow processing
-   💾 Massive file bloat
-   🔥 CPU-melting decryption
-   🧵 Multi-threaded (but still painful)
-   🎨 Supports any input image format
-   😈 Perfect for torturing hardware

## File Format Spec 📑

```
width:height;[SHA3-512(pixel1)];[SHA3-512(pixel2)];...
```

Each pixel is encoded as: 0xRRGGBBAA → SHA3-512 hash

## Usage 🚀

### Encrypt (relatively quick)

```
cargo run
> 1
> input.png
```

### Decrypt (prepare to wait...)

```
cargo run
> 2
> encrypted.pain
```

### Run test (10x10 image with blue color)

```
cargo run
> 3
```

## Performance 📊

| Operation  |   Speed    | Pain Level |
| :--------: | :--------: | :--------: |
| Encryption |  Okay ⚡   |     😐     |
| Decryption | Glacial 🐌 |     😱     |
| File Size  |  Huge 💾   |     🤯     |

## Why? 🤔

Because sometimes we need to remind ourselves that efficiency isn't everything.

## License 📜

Public Domain - This crime against computing belongs to everyone.

---

_Made with ❤️ and significant CPU abuse_

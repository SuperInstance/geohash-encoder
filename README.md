# Geohash Encoder

**A Rust library for encoding and decoding geographic coordinates using the Geohash system** — a hierarchical spatial index that represents lat/lon pairs as compact base32 strings.

## Why It Matters

Geohash is the spatial indexing standard used by Redis, Elasticsearch, MongoDB, and countless geolocation services. By interleaving the bits of latitude and longitude, geohashes produce strings where **shared prefixes indicate geographic proximity** — the hash `u4pruyd` is inside `u4pr`, which is inside `u4`. This prefix property enables efficient bounding-box queries: `WHERE geohash LIKE 'u4pr%'` finds all points in a ~5 km × 5 km region using a simple B-tree index. The 8-character hash achieves ±19 m precision, sufficient for most consumer mapping applications.

## How It Works

**Encoding** alternates between longitude and latitude bits, performing a binary search on each range. For each of the 5 bits in a base32 character: if the coordinate is in the upper half of the current range, emit 1 and narrow to the upper half; otherwise emit 0 and narrow to the lower half. Longitude is encoded first (even bit positions), then latitude (odd positions). Every 5 bits produce one base32 character from the alphabet `0123456789bcdefghjkmnpqrstuvwxyz` (note: `a`, `i`, `l`, `o` are excluded to avoid confusion with digits).

**Decoding** reverses this: each base32 character yields 5 bits, which are used to narrow the lat/lon ranges. The result is the center point of the decoded cell, with precision determined by the hash length.

**Neighbor computation** decodes the center point, offsets by one cell width in each of 8 directions (N, NE, E, SE, S, SW, W, NW), and re-encodes. The cell dimensions shrink as `180°/4^(5p/2)` for latitude and `360°/4^((5p+1)/2)` for longitude, where `p` is the precision (hash length).

## Quick Start

```rust
use geohash_encoder::{encode, decode, neighbors};

fn main() {
    // Encode: latitude first, then longitude, with precision
    let hash = encode(57.64911, 10.40744, 12);
    println!("Geohash: {}", hash); // u4pruydqqvj8

    // Decode back to coordinates
    let (lat, lon) = decode(&hash).unwrap();
    println!("Decoded: ({:.5}, {:.5})", lat, lon);

    // Get 8 neighbors (for ring queries)
    let nbs = neighbors(&hash).unwrap();
    for (i, n) in nbs.iter().enumerate() {
        println!("Neighbor {}: {}", i, n);
    }
}
```

## API

| Function | Complexity | Description |
|---|---|---|
| `encode(lat, lon, precision)` | **O(p)** | Encode coordinates to a geohash string |
| `decode(hash)` | **O(p)** | Decode a geohash to `(lat, lon)` center point |
| `neighbors(hash)` | **O(p)** | Compute 8 adjacent geohashes (ring-1) |

## Architecture Notes

Part of the SuperInstance geospatial toolkit. Used alongside `h3-index` (hexagonal indexing) for fleet vehicle tracking and proximity queries. See the [Architecture Guide](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## License

MIT

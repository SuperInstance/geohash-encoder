# geohash-encoder: Geohash Encoding and Decoding

A zero-dependency implementation of the **Geohash** geocoding system (Gustavo Niemeyer, 2008). Encodes (latitude, longitude) pairs into base-32 strings and decodes them back, with neighbor computation for spatial adjacency queries.

## Why It Matters

Geohashing is the standard technique for encoding continuous geographic coordinates into a discrete, prefix-comparable string. Because geohashes form a **Z-order curve** (Morton code) over the Earth's surface, they enable:

- **Database indexing**: Nearby points share string prefixes, making B-tree range queries efficient
- **Spatial proximity**: "Find all points within geohash `u4pr`" is a simple string prefix scan
- ** Adjustable precision**: A 1-character hash covers ~5000 km; a 12-character hash covers ~3.7 cm
- **Distributed systems**: Geohash ranges partition cleanly across shards

Used by Redis, MongoDB, Elasticsearch, Google's S2 geometry, and every ride-sharing/delivery app.

## How It Works

### Encoding Algorithm

The encoder interleaves bits for longitude and latitude, performing a **binary search** on the bounding box:

```
For each bit position (up to precision × 5):
    if even bit: bisect longitude range
    if odd bit:  bisect latitude range
    if coordinate ≥ midpoint: append 1, narrow lower bound
    else:                    append 0, narrow upper bound
After 5 bits: emit next base-32 character
```

The bit interleaving (longitude first) ensures that geohashes with common prefixes are geographically close — this is the key property of the Z-order curve.

### Base-32 Alphabet

Geohash uses a custom base-32 alphabet that excludes `a`, `i`, `l`, and `o` to avoid confusion:

```
0123456789bcdefghjkmnpqrstuvwxyz
```

### Decoding

Reverse process: each base-32 character yields 5 bits, de-interleaved into lon/lat bit sequences that narrow the bounding box. The decoded coordinate is the **center** of the final bounding box.

### Precision vs. Cell Size

| Precision | Cell Width | Cell Height | Example Coverage |
|-----------|-----------|-------------|-----------------|
| 1 | 5000 km | 5000 km | Continent |
| 4 | 39 km | 20 km | City |
| 7 | 153 m | 76 m | City block |
| 9 | 4.8 m | 4.8 m | Indoor |
| 12 | 3.7 cm | 1.9 cm | Point |

### Neighbor Computation

Given a geohash, computes its 8 surrounding cells by decoding to (lat, lon), offsetting by the cell dimensions, and re-encoding. This enables ring queries: get all neighbors at the same precision level.

### Complexity

| Operation | Time | Space |
|-----------|------|-------|
| `encode(lat, lon, p)` | O(p) | O(p) |
| `decode(hash)` | O(p) | O(1) |
| `neighbors(hash)` | O(p) | O(1) |

Where p = precision (number of characters).

## Quick Start

```rust
use geohash_encoder::{encode, decode, neighbors};

// Encode Copenhagen: 57.64911°N, 10.40744°E
let hash = encode(57.64911, 10.40744, 12);
assert_eq!(&hash[..4], "u4pr");

// Decode back (center of cell)
let (lat, lon) = decode(&hash).unwrap();
assert!((lat - 57.64911).abs() < 0.001);

// Get 8 neighbors
let n = neighbors(&hash).unwrap();
assert_eq!(n.len(), 8);
```

## API

| Function | Signature | Description |
|----------|-----------|-------------|
| `encode` | `(lat: f64, lon: f64, precision: usize) -> String` | Encode coordinates to geohash |
| `decode` | `(hash: &str) -> Option<(f64, f64)>` | Decode to (lat, lon) center |
| `neighbors` | `(hash: &str) -> Option<Vec<String>>` | 8 adjacent cells at same precision |

## Architecture Notes

This is a **γ (gamma)** module — pure functions, no state, no I/O. In the γ + η = C framework, it provides the spatial encoding primitive. An **η** layer would build spatial indexes (geohash tree, prefix trie), proximity search, and clustering on top of these primitives. The Z-order curve is not a perfect space-filling curve (it has discontinuities at power-of-2 boundaries), but it is the most widely deployed one.

## References

- Niemeyer, G. (2008). *Geohash: A Degree of Precision*. geohash.org.
- Morton, G. M. (1966). *A Computer Oriented Geodetic Data Base*. IBM.
- Sahr, K., White, D., & Kimerling, A. J. (2003). *Geodesic Discrete Global Grid Systems*. Cartography and Geographic Information Science 30(2).

## License

MIT

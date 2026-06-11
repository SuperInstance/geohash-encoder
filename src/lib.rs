//! Geohash encoding and decoding for geographic coordinates.

const BASE32_CHARS: &[u8; 32] = b"0123456789bcdefghjkmnpqrstuvwxyz";

fn char_to_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'b'..=b'h' => Some(c - b'a' + 10),
        b'j' | b'k' => Some(c - b'a' + 9),
        b'm'..=b'n' => Some(c - b'a' + 8),
        b'p'..=b'z' => Some(c - b'a' + 7),
        _ => None,
    }
}

pub fn encode(lat: f64, lon: f64, precision: usize) -> String {
    let mut lat_range = (-90.0, 90.0);
    let mut lon_range = (-180.0, 180.0);
    let mut hash = String::with_capacity(precision);
    let mut bit = 0u8;
    let mut val = 0u8;

    while hash.len() < precision {
        if bit % 2 == 0 {
            let mid = (lon_range.0 + lon_range.1) / 2.0;
            if lon >= mid {
                val = val * 2 + 1;
                lon_range.0 = mid;
            } else {
                val *= 2;
                lon_range.1 = mid;
            }
        } else {
            let mid = (lat_range.0 + lat_range.1) / 2.0;
            if lat >= mid {
                val = val * 2 + 1;
                lat_range.0 = mid;
            } else {
                val *= 2;
                lat_range.1 = mid;
            }
        }
        bit += 1;
        if bit == 5 {
            hash.push(BASE32_CHARS[val as usize] as char);
            bit = 0;
            val = 0;
        }
    }
    hash
}

pub fn decode(hash: &str) -> Option<(f64, f64)> {
    let mut lat_range = (-90.0, 90.0);
    let mut lon_range = (-180.0, 180.0);
    let mut is_lon = true;

    for c in hash.bytes() {
        let mut val = char_to_val(c)?;
        for i in (0..5).rev() {
            let mask = 1u8 << i;
            let bit = val & mask != 0;
            if is_lon {
                let mid = (lon_range.0 + lon_range.1) / 2.0;
                if bit { lon_range.0 = mid; } else { lon_range.1 = mid; }
            } else {
                let mid = (lat_range.0 + lat_range.1) / 2.0;
                if bit { lat_range.0 = mid; } else { lat_range.1 = mid; }
            }
            is_lon = !is_lon;
        }
    }
    Some(((lat_range.0 + lat_range.1) / 2.0, (lon_range.0 + lon_range.1) / 2.0))
}

pub fn neighbors(hash: &str) -> Option<Vec<String>> {
    let (lat, lon) = decode(hash)?;
    let precision = hash.len();
    let dlat = 180.0 / 4f64.powi(precision as i32 * 5 / 2);
    let dlon = 360.0 / 4f64.powi((precision as i32 * 5 + 1) / 2);
    let offsets = [
        (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (-1.0, 1.0),
        (-1.0, 0.0), (-1.0, -1.0), (0.0, -1.0), (1.0, -1.0),
    ];
    Some(offsets.iter().map(|(dlat_off, dlon_off)| {
        encode((lat + dlat * dlat_off).clamp(-90.0, 90.0), lon + dlon * dlon_off, precision)
    }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let hash = encode(57.64911, 10.40744, 12);
        assert_eq!(&hash[..4], "u4pr");
        let (lat, lon) = decode(&hash).unwrap();
        assert!((lat - 57.64911).abs() < 0.001);
        assert!((lon - 10.40744).abs() < 0.001);
    }
}

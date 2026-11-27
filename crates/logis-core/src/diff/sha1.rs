//! SHA-1 (FIPS 180-4), for git object ids only: diffs print git blob hashes, as chezmoi does
//! through go-git's `plumbing.ComputeHash`. Not for anything that needs a secure hash.

/// The git object id of a blob holding `data`: SHA-1 of `blob <len>\0<data>`, in hex.
#[must_use]
pub fn git_blob_hash(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(format!("blob {}\0", data.len()).as_bytes());
    hasher.update(data);
    to_hex(&hasher.finish())
}

fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
}

struct Sha1 {
    state: [u32; 5],
    block: Vec<u8>,
    length: u64,
}

impl Sha1 {
    fn new() -> Self {
        Self {
            state: [
                0x6745_2301,
                0xEFCD_AB89,
                0x98BA_DCFE,
                0x1032_5476,
                0xC3D2_E1F0,
            ],
            block: Vec::with_capacity(64),
            length: 0,
        }
    }

    fn update(&mut self, mut data: &[u8]) {
        self.length += data.len() as u64;
        while !data.is_empty() {
            let take = (64 - self.block.len()).min(data.len());
            self.block.extend_from_slice(&data[..take]);
            data = &data[take..];
            if let Ok(block) = <[u8; 64]>::try_from(self.block.as_slice()) {
                self.compress(&block);
                self.block.clear();
            }
        }
    }

    fn finish(mut self) -> [u8; 20] {
        let bits = self.length.wrapping_mul(8);
        let mut padding = vec![0x80];
        let used = (self.block.len() + 1) % 64;
        padding.resize(1 + (64 + 56 - used) % 64, 0);
        padding.extend_from_slice(&bits.to_be_bytes());
        let length = self.length;
        self.update(&padding);
        self.length = length;
        let mut out = [0; 20];
        for (chunk, word) in out.chunks_mut(4).zip(self.state) {
            chunk.copy_from_slice(&word.to_be_bytes());
        }
        out
    }

    #[allow(
        clippy::many_single_char_names,
        reason = "the names in FIPS 180-4, so the code can be checked against it"
    )]
    fn compress(&mut self, block: &[u8]) {
        let mut w = [0u32; 80];
        for (i, word) in block.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = self.state;
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..20 => ((b & c) | (!b & d), 0x5A82_7999),
                20..40 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..60 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        for (state, value) in self.state.iter_mut().zip([a, b, c, d, e]) {
            *state = state.wrapping_add(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sha1;

    fn hex(data: &[u8]) -> String {
        let mut hasher = Sha1::new();
        hasher.update(data);
        super::to_hex(&hasher.finish())
    }

    /// FIPS 180 test vectors, including the two-block message and one million `a`s.
    #[test]
    fn known_digests() {
        assert_eq!(hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(
            hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
        );
        assert_eq!(
            hex(&vec![b'a'; 1_000_000]),
            "34aa973cd4c4daa4f61eeb2bdbad27316534016f"
        );
    }
}

//! Whether data is binary, as chezmoi's `isBinary` decides it: Go's `http.DetectContentType`
//! (the WHATWG MIME sniffing algorithm, on the first 512 bytes) not returning a `text/` type.
//!
//! Every signature Go matches that is not a text type makes the data binary. Signatures that
//! need a binary byte to match are covered by the binary-byte check, which Go applies last;
//! the ones below are those an all-text prefix can match. Verify against Go's `sniff.go` when
//! changing this.

/// Go sniffs at most this many bytes.
const SNIFF_LEN: usize = 512;

/// Prefixes that Go maps to a type that is not `text/`, and that contain no binary byte.
const TEXT_LOOKING_BINARY: &[&[u8]] = &[
    b"%PDF-",
    b"%!PS-Adobe-",
    b"GIF87a",
    b"GIF89a",
    b"BM",
    b"ID3",
    b"OTTO",
    b"ttcf",
    b"wOFF",
    b"wOF2",
];

/// Data that Go would not call text. Empty data is text (`isBinary` checks the length first).
#[must_use]
pub fn is_binary(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }
    let data = &data[..data.len().min(SNIFF_LEN)];
    if has_bom(data) {
        return false;
    }
    if TEXT_LOOKING_BINARY.iter().any(|sig| data.starts_with(sig)) || is_riff_media(data) {
        return true;
    }
    data.iter().any(|&b| is_binary_byte(b))
}

/// UTF-16 and UTF-8 byte order marks make Go call the data text.
fn has_bom(data: &[u8]) -> bool {
    data.starts_with(&[0xFE, 0xFF])
        || data.starts_with(&[0xFF, 0xFE])
        || data.starts_with(&[0xEF, 0xBB, 0xBF])
}

/// `RIFF....WAVE`, `RIFF....AVI ` and `FORM....AIFF`, whose length bytes may be text.
fn is_riff_media(data: &[u8]) -> bool {
    let tagged = |head: &[u8], tail: &[u8]| {
        data.len() >= 12 && data.starts_with(head) && &data[8..12] == tail
    };
    tagged(b"RIFF", b"WAVE") || tagged(b"RIFF", b"AVI ") || tagged(b"FORM", b"AIFF")
}

/// The WHATWG "binary data byte": control characters other than tab, newline, form feed,
/// carriage return and escape.
fn is_binary_byte(b: u8) -> bool {
    matches!(b, 0x00..=0x08 | 0x0B | 0x0E..=0x1A | 0x1C..=0x1F)
}

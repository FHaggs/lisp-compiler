pub type Word = i64;

// The mask for all immediate types (char, bool, nil)
pub const K_IMMEDIATE_TAG_MASK: Word = 0x3f; // 0b00111111

// --- Character Encoding ---
pub const K_CHAR_TAG: Word = 0x0f; // 0b00001111
const K_CHAR_MASK: Word = 0xff; // Mask to extract the char value
const K_CHAR_SHIFT: u32 = 8;

// --- Boolean Encoding ---
pub const K_BOOL_TAG: Word = 0x1f; // 0b00011111
const K_BOOL_MASK: Word = 0x80; // Mask to extract the bool value (the 7th bit)
const K_BOOL_SHIFT: u32 = 7;

// --- Canonical Object Values ---

/// The single, unique value representing 'nil'.
pub fn object_nil() -> Word {
    0x2f // 0b00101111
}

/// The single, unique value representing 'true'.
pub fn object_true() -> Word {
    encode_bool(true)
}

/// The single, unique value representing 'false'.
pub fn object_false() -> Word {
    encode_bool(false)
}

// --- Integer Encoding ---
// We use 62 bits for encoding ints, 1 is the sign bit.
const K_INTEGER_MAX: Word = (1_i64 << (62 - 1)) - 1;
const K_INTEGER_MIN: Word = -(1_i64 << (62 - 1));
const K_INTEGER_SHIFT: u32 = 2;

pub fn encode_integer(value: Word) -> Word {
    assert!(
        value >= K_INTEGER_MIN && value <= K_INTEGER_MAX,
        "Integer out of range"
    );
    value << K_INTEGER_SHIFT
}

pub fn decode_integer(value: Word) -> Word {
    // Note: The right shift on a signed integer (i64) is an
    // arithmetic shift, which correctly preserves the sign.
    value >> K_INTEGER_SHIFT
}

pub fn encode_char(value: char) -> Word {
    ((value as Word) << K_CHAR_SHIFT) | K_CHAR_TAG
}

pub fn decode_char(value: Word) -> char {
    let decoded = (value >> K_CHAR_SHIFT) & K_CHAR_MASK;
    // This can panic if the value is not a valid char.
    // In a real compiler, you might want safer handling.
    std::char::from_u32(decoded as u32).unwrap()
}

pub fn encode_bool(value: bool) -> Word {
    ((value as Word) << K_BOOL_SHIFT) | K_BOOL_TAG
}

pub fn decode_bool(value: Word) -> bool {
    (value & K_BOOL_MASK) != 0
}

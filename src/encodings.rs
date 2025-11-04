pub type Word = i64;

// --- Private Constants (Implementation Details) ---
// (These are the same as before, but not all need to be pub)
const K_CHAR_TAG: Word = 0x0f;
const K_CHAR_MASK: Word = 0xff;
const K_CHAR_SHIFT: u32 = 8;

const K_BOOL_TAG: Word = 0x1f;
const K_BOOL_MASK: Word = 0x80;
const K_BOOL_SHIFT: u32 = 7;

const K_NIL_VALUE: Word = 0x2f;

const K_INTEGER_MAX: Word = (1_i64 << (62 - 1)) - 1;
const K_INTEGER_MIN: Word = -(1_i64 << (62 - 1));
const K_INTEGER_SHIFT: u32 = 2;
const K_INTEGER_MASK: Word = 0x03; // Mask to check the integer tag (0b11)
const K_INTEGER_TAG: Word = 0x00; // Tag for integers (0b00)

/// A type-safe wrapper for a 64-bit tagged Lisp value.
/// It has the same size and performance as a raw i64.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // Guarantees it's just a Word
pub struct LispValue(Word);

impl LispValue {
    pub fn from_raw_word(word: Word) -> Self {
        LispValue(word)
    }
    // --- 1. CONSTRUCTORS (Encoding) ---

    /// Creates a new LispValue from a native integer.
    pub fn from_integer(value: Word) -> Self {
        assert!(
            value >= K_INTEGER_MIN && value <= K_INTEGER_MAX,
            "Integer out of range"
        );
        // The tag (0b00) is implicit in the shift.
        LispValue(value << K_INTEGER_SHIFT)
    }

    /// Creates a new LispValue from a native char.
    pub fn from_char(value: char) -> Self {
        LispValue(((value as Word) << K_CHAR_SHIFT) | K_CHAR_TAG)
    }

    /// Creates a new LispValue from a native bool.
    pub fn from_bool(value: bool) -> Self {
        LispValue(((value as Word) << K_BOOL_SHIFT) | K_BOOL_TAG)
    }

    /// Returns the canonical 'nil' LispValue.
    pub fn nil() -> Self {
        LispValue(K_NIL_VALUE)
    }

    /// Returns the canonical 'true' LispValue.
    pub fn true_val() -> Self {
        Self::from_bool(true)
    }

    /// Returns the canonical 'false' LispValue.
    pub fn false_val() -> Self {
        Self::from_bool(false)
    }

    // --- 2. TYPE CHECKERS (Tag Checking) ---

    pub fn is_integer(&self) -> bool {
        (self.0 & K_INTEGER_MASK) == K_INTEGER_TAG
    }

    pub fn is_char(&self) -> bool {
        (self.0 & K_CHAR_TAG) == K_CHAR_TAG // Simplified check for this scheme
    }

    pub fn is_bool(&self) -> bool {
        (self.0 & K_BOOL_TAG) == K_BOOL_TAG // Simplified check
    }

    pub fn is_nil(&self) -> bool {
        self.0 == K_NIL_VALUE
    }

    // --- 3. ACCESSORS (Decoding) ---

    /// If this value is an integer, returns the decoded i64.
    pub fn as_integer(&self) -> Option<Word> {
        if self.is_integer() {
            Some(self.0 >> K_INTEGER_SHIFT)
        } else {
            None
        }
    }

    /// If this value is a char, returns the decoded char.
    pub fn as_char(&self) -> Option<char> {
        if self.is_char() {
            let decoded = (self.0 >> K_CHAR_SHIFT) & K_CHAR_MASK;
            std::char::from_u32(decoded as u32) // Returns Option<char>
        } else {
            None
        }
    }

    /// If this value is a bool, returns the decoded bool.
    pub fn as_bool(&self) -> Option<bool> {
        if self.is_bool() {
            Some((self.0 & K_BOOL_MASK) != 0)
        } else {
            None
        }
    }

    // --- 4. RAW ACCESS ---

    /// Returns the raw encoded 64-bit Word.
    /// Your compiler will use this to pass the value to the assembler.
    pub fn as_raw_word(&self) -> Word {
        self.0
    }
}

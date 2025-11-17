pub type Word = i64;
pub type UWord = u64;

// POINTER TAGGING SCHEMA
// High                                                         Low
// XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX00  Integer
// 0000000000000000000000000000000000000000000000000XXXXXXX00001111  Character
// 00000000000000000000000000000000000000000000000000000000X0011111  Boolean
// 0000000000000000000000000000000000000000000000000000000000101111  Nil
// XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX001  Pair
// XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX010  Vector
// XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX011  String
// XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX101  Symbol
// XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX110  Closure

// struct TagsDict {
//     tags: Vec<(Word, Word)>,
// }
// impl TagsDict {
//     fn new() -> Self {
//         TagsDict { tags: Vec::new() }
//     }
//     fn add_tag(&mut self, tag: Word, mask: Word) {
//         self.tags.push((tag, mask));
//     }
//     pub fn check_for_overlapping(&self) {}
// }

pub const K_CHAR_TAG: Word = 0x0f;
const K_CHAR_MASK: Word = 0xff;
pub const K_CHAR_SHIFT: u32 = 8;

pub const K_BOOL_TAG: Word = 0x1f;
pub const K_BOOL_MASK: Word = 0x80;
pub const K_BOOL_SHIFT: u32 = 7;

const K_NIL_VALUE: Word = 0x2f;

const K_INTEGER_MAX: Word = (1_i64 << (62 - 1)) - 1;
const K_INTEGER_MIN: Word = -(1_i64 << (62 - 1));
pub const K_INTEGER_SHIFT: u32 = 2;
pub const K_INTEGER_MASK: Word = 0x03;
pub const K_INTEGER_TAG: Word = 0x00;

// Pairs
const K_PAIR_TAG: Word = 0x1;
const K_HEAP_TAG_MASK: Word = 0x7; // 0b111
const K_HEAP_PTR_MASK: Word = !K_HEAP_TAG_MASK;
// Symbols
const K_SYMBOL_TAG: Word = 0x5; // 0b101

/// TODO: Alloc this in our custom heap, using a bump allocator
/// This is the memory layout for a 'cons' cell on the heap.
#[derive(Debug, Clone, Copy)]
// We align it to 8 bytes, which is standard for 64-bit.
// This guarantees the pointer to it will end in 0b000.
#[repr(C, align(8))]
pub struct Pair {
    pub car: LispValue,
    pub cdr: LispValue,
}

// Should I own it?
#[derive(Debug, Clone)]
// We align it to 8 bytes, which is standard for 64-bit.
// This guarantees the pointer to it will end in 0b000.
#[repr(C, align(8))]
pub struct Symbol {
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // Guarantees it's just a Word
pub struct LispValue(Word);

impl LispValue {
    pub fn from_pair_pointer(ptr: *mut Pair) -> Self {
        let addr = ptr as Word;
        assert!(
            (addr & K_HEAP_TAG_MASK) == 0,
            "Pointer is not 8-byte aligned!"
        );
        // Create the tagged pointer by OR-ing the tag.
        LispValue(addr | K_PAIR_TAG)
    }
    pub fn is_pair(&self) -> bool {
        (self.0 & K_HEAP_TAG_MASK) == K_PAIR_TAG
    }
    pub fn from_symbol_pointer(ptr: *mut Symbol) -> Self {
        let addr = ptr as Word;
        assert!(
            (addr & K_HEAP_TAG_MASK) == 0,
            "Pointer is not 8-byte aligned!"
        );
        LispValue(addr | K_SYMBOL_TAG)
    }

    /// Checks if this LispValue is a tagged pointer to a Symbol.
    pub fn is_symbol(&self) -> bool {
        (self.0 & K_HEAP_TAG_MASK) == K_SYMBOL_TAG
    }

    /// If this value is a Symbol, returns the raw, untagged pointer to it.
    pub fn as_symbol_pointer(&self) -> Option<*mut Symbol> {
        if self.is_symbol() {
            let addr = self.0 & K_HEAP_PTR_MASK;
            Some(addr as *mut Symbol)
        } else {
            None
        }
    }
    pub fn from_raw_word(word: Word) -> Self {
        LispValue(word)
    }

    /// Creates a new LispValue from a native integer.
    pub fn from_integer(value: Word) -> Self {
        assert!(
            value >= K_INTEGER_MIN && value <= K_INTEGER_MAX,
            "Integer out of range"
        );
        // The tag (0b00) is implicit in the shift.
        LispValue(value << K_INTEGER_SHIFT)
    }

    pub fn from_char(value: char) -> Self {
        LispValue(((value as Word) << K_CHAR_SHIFT) | K_CHAR_TAG)
    }

    pub fn from_bool(value: bool) -> Self {
        LispValue(((value as Word) << K_BOOL_SHIFT) | K_BOOL_TAG)
    }

    pub fn nil() -> Self {
        LispValue(K_NIL_VALUE)
    }

    pub fn true_val() -> Self {
        Self::from_bool(true)
    }

    pub fn false_val() -> Self {
        Self::from_bool(false)
    }

    pub fn is_integer(&self) -> bool {
        (self.0 & K_INTEGER_MASK) == K_INTEGER_TAG
    }

    pub fn is_char(&self) -> bool {
        (self.0 & K_CHAR_TAG) == K_CHAR_TAG
    }

    pub fn is_bool(&self) -> bool {
        (self.0 & K_BOOL_TAG) == K_BOOL_TAG
    }

    pub fn is_nil(&self) -> bool {
        self.0 == K_NIL_VALUE
    }

    pub fn as_integer(&self) -> Option<Word> {
        if self.is_integer() {
            Some(self.0 >> K_INTEGER_SHIFT)
        } else {
            None
        }
    }

    pub fn as_char(&self) -> Option<char> {
        if self.is_char() {
            let decoded = (self.0 >> K_CHAR_SHIFT) & K_CHAR_MASK;
            std::char::from_u32(decoded as u32)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if self.is_bool() {
            Some((self.0 & K_BOOL_MASK) != 0)
        } else {
            None
        }
    }

    pub fn as_raw_word(&self) -> Word {
        self.0
    }
    pub fn as_pair_pointer(&self) -> Option<*mut Pair> {
        if self.is_pair() {
            let addr = self.0 & K_HEAP_PTR_MASK;
            Some(addr as *mut Pair)
        } else {
            None
        }
    }

    // Debug and print the rust value by checking all tags
    pub fn print(&self) {
        if self.is_bool() {
            println!("Bool: {}", self.as_bool().unwrap());
        } else if self.is_integer() {
            println!("Integer: {}", self.as_integer().unwrap());
        } else if self.is_char() {
            println!("Char: {}", self.as_char().unwrap());
        } else if self.is_nil() {
            println!("Nil");
        } else if self.is_symbol() {
            println!("Symbol: TODO!");
        } else if self.is_pair() {
            println!("Pair: {:?}", self.as_pair_pointer().unwrap());
        } else {
            println!("Unknown");
        }
    }
}

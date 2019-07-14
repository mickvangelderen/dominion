#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum Tag {
    Join = 1
}

impl Tag {
    #[inline]
    pub fn into_bytes(self) -> [u8; 4] {
        (self as u32).to_le_bytes()
    }

    #[inline]
    pub fn try_from_bytes(bytes: [u8; 4]) -> Result<Self, u32> {
        match u32::from_le_bytes(bytes) {
            x if x == Tag::Join as u32 => Ok(Tag::Join),
            invalid => Err(invalid),
        }
    }
}

#[repr(C)]
pub struct Join {
}

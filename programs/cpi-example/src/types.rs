use anchor_lang::prelude::*;

pub enum Side {
    Ask,
    Bid,
}

impl Side {
    pub fn from_u8(value: u8) -> Result<Self> {
        assert!(value == 0 || value == 1);
        match value {
            0 => Ok(Side::Ask),
            1 => Ok(Side::Bid),
            _ => unreachable!(),
        }
    }
}

use std::{error::Error, fmt::Display, num::ParseIntError, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddrMode {
    Absolute,
    AbsoluteIndexedIndirect,
    AbsoluteIndexedWithX,
    AbsoluteIndexedWithY,
    AbsoluteIndirect,
    Accumulator,
    Immediate,
    Implied,
    Relative,
    ZeroPage,
    ZeroPageIndexedIndirect,
    ZeroPageIndexedWithX,
    ZeroPageIndexedWithY,
    ZeroPageIndirect,
    ZeroPageIndirectIndexedWithY,
    ZeroPageRelative,
}

impl super::IsOriginal for AddrMode {
    fn is_original(&self) -> bool {
        !matches!(
            self,
            AddrMode::AbsoluteIndexedIndirect
                | AddrMode::ZeroPageIndirect
                | AddrMode::ZeroPageRelative
        )
    }
}

impl AddrMode {
    pub fn get_cycles(&self) -> u8 {
        match self {
            AddrMode::Absolute => 4,
            AddrMode::AbsoluteIndexedIndirect => 6,
            AddrMode::AbsoluteIndexedWithX => 4,
            AddrMode::AbsoluteIndexedWithY => 4,
            AddrMode::AbsoluteIndirect => 6,
            AddrMode::Accumulator => 2,
            AddrMode::Immediate => 2,
            AddrMode::Implied => 2,
            AddrMode::Relative => 2,
            AddrMode::ZeroPage => 3,
            AddrMode::ZeroPageIndexedIndirect => 6,
            AddrMode::ZeroPageIndexedWithX => 4,
            AddrMode::ZeroPageIndexedWithY => 4,
            AddrMode::ZeroPageIndirect => 5,
            AddrMode::ZeroPageIndirectIndexedWithY => 5,
            AddrMode::ZeroPageRelative => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseAddrModeError{ s: String }

impl Display for ParseAddrModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("ParseAddrModeError: Unable to parse {}", &self.s).as_str())
    }
}

impl Error for ParseAddrModeError {}

impl FromStr for AddrMode {
    type Err = ParseAddrModeError;

    /**
     * Convert a string of the form #$08 or $(ABCD),Y etc to the corresponding AddrMode
     */
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rparen = s.find(')');
        let comma = s.find(',');
        let selector = (s.bytes().nth(0), s.bytes().nth(1), rparen, comma);
        let ret = match selector {
            (Some(b'$'), Some(c),None,None) if c != b'(' && s.len() == 5 =>
              Ok(AddrMode::Absolute),
            (Some(b'('), Some(b'$'),Some(rp),Some(co)) if co < rp && s.len() == 7 =>
              Ok(AddrMode::AbsoluteIndexedIndirect),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'X') && s.len() == 9 =>
              Ok(AddrMode::AbsoluteIndexedWithX),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'Y') && s.len() == 9 =>
              Ok(AddrMode::AbsoluteIndexedWithY),
            (Some(b'('), Some(b'$'), Some(_),None) if s.len() == 7 =>
              Ok(AddrMode::AbsoluteIndirect),
            (Some(b'#'), Some(b'$'), None,None) =>
              Ok(AddrMode::Immediate),
            _ if s.len() == 3 =>
              Ok(AddrMode::Implied),
            (Some(b'$'), Some(c),None,None) if c != b'(' && s.len() == 3 =>
              Ok(AddrMode::ZeroPage),
            (Some(b'('), Some(b'$'),Some(rp),Some(co)) if co < rp && s.len() == 5 =>
              Ok(AddrMode::ZeroPageIndexedIndirect),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'X') && s.len() == 7 =>
              Ok(AddrMode::ZeroPageIndexedWithX),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'Y') && s.len() == 7 =>
              Ok(AddrMode::ZeroPageIndexedWithY),
            (Some(b'('), Some(b'$'), Some(_),None) if s.len() == 5 =>
              Ok(AddrMode::ZeroPageIndirect),
            (Some(b'('), Some(b'$'),Some(rp),Some(co)) if co > rp && s.len() == 5 =>
              Ok(AddrMode::ZeroPageIndirectIndexedWithY),
            _ => Err(ParseAddrModeError { s: s.to_owned() }),
        };
        ret
    }
}

impl From<ParseIntError> for ParseAddrModeError {
    fn from(value: ParseIntError) -> Self {
        ParseAddrModeError { s: format!("ParseIntError occurred: {}", value).to_string() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AddrModeWithAddr {
    pub mode: AddrMode,
    pub arg: u16,
    pub arg_size: u8,
}

impl FromStr for AddrModeWithAddr {

    type Err = ParseAddrModeError;

    /**
     * Convert a string of the form #$08 or $(ABCD),Y etc to the corresponding (AddrMode, u16)
     * (the argument)
     */
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rparen = s.find(')');
        let comma = s.find(',');
        let selector = (s.bytes().nth(0), s.bytes().nth(1), rparen, comma);
        let ret = match selector {
            (Some(b'$'), Some(c),None,None) if c != b'(' && s.len() == 5 =>
              (AddrMode::Absolute, u16::from_str_radix(&s.to_owned()[1..5], 16)?, 2),
            (Some(b'('), Some(b'$'),Some(rp),Some(co)) if co < rp && s.len() == 9 =>
              (AddrMode::AbsoluteIndexedIndirect, u16::from_str_radix(&s.to_owned()[2..6], 16)?, 2),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'X') && s.len() == 7 =>
              (AddrMode::AbsoluteIndexedWithX, u16::from_str_radix(&s.to_owned()[1..5], 16)?, 2),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'Y') && s.len() == 7 =>
              (AddrMode::AbsoluteIndexedWithY, u16::from_str_radix(&s.to_owned()[1..5], 16)?, 2),
            (Some(b'('), Some(b'$'), Some(_),None) if s.len() == 7 =>
              (AddrMode::AbsoluteIndirect, u16::from_str_radix(&s.to_owned()[2..6], 16)?, 2),
            (Some(b'#'), Some(b'$'), None,None) =>
              (AddrMode::Immediate, u16::from_str_radix(&s.to_owned()[2..4], 16)?, 1),
            (Some(b'$'), Some(c),None,None) if c != b'(' && s.len() == 3 =>
              (AddrMode::ZeroPage, u16::from_str_radix(&s.to_owned()[1..3], 16)?, 1),
            (Some(b'('), Some(b'$'),Some(rp),Some(co)) if co < rp && s.len() == 7 =>
              (AddrMode::ZeroPageIndexedIndirect, u16::from_str_radix(&s.to_owned()[2..4], 16)?, 1),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'X') && s.len() == 5 =>
              (AddrMode::ZeroPageIndexedWithX, u16::from_str_radix(&s.to_owned()[1..3], 16)?, 1),
            (Some(b'$'), Some(_), None,Some(co)) if s.bytes().nth(co+1) == Some(b'Y') && s.len() == 5 =>
              (AddrMode::ZeroPageIndexedWithY, u16::from_str_radix(&s.to_owned()[1..3], 16)?, 1),
            (Some(b'('), Some(b'$'), Some(_),None) if s.len() == 5 =>
              (AddrMode::ZeroPageIndirect, u16::from_str_radix(&s.to_owned()[2..4], 16)?, 1),
            (Some(b'('), Some(b'$'),Some(rp),Some(co)) if co > rp && s.len() == 7 =>
              (AddrMode::ZeroPageIndirectIndexedWithY, u16::from_str_radix(&s.to_owned()[2..4], 16)?, 1),
            _ => Err(ParseAddrModeError { s: s.to_owned() })?,
        };
        Ok(AddrModeWithAddr{ mode: ret.0, arg: ret.1, arg_size: ret.2 })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_addr_mode_with_addr_from_str() {
        let res = AddrModeWithAddr::from_str("$AAAA");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::Absolute, arg: 0xaaaa, arg_size: 2 }), res);
        let res = AddrModeWithAddr::from_str("($AAAA,X)");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::AbsoluteIndexedIndirect, arg: 0xaaaa, arg_size: 2 }), res);
        let res = AddrModeWithAddr::from_str("$AAAA,X");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::AbsoluteIndexedWithX, arg: 0xaaaa, arg_size: 2 }), res);
        let res = AddrModeWithAddr::from_str("$AAAA,Y");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::AbsoluteIndexedWithY, arg: 0xaaaa, arg_size: 2 }), res);
        let res = AddrModeWithAddr::from_str("($AAAA)");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::AbsoluteIndirect, arg: 0xaaaa, arg_size: 2 }), res);
        let res = AddrModeWithAddr::from_str("#$AA");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::Immediate, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("#$AA");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::Immediate, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("$AA");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::ZeroPage, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("($AA,X)");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::ZeroPageIndexedIndirect, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("$AA,X");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::ZeroPageIndexedWithX, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("$AA,Y");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::ZeroPageIndexedWithY, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("($AA)");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::ZeroPageIndirect, arg: 0xaa, arg_size: 1 }), res);
        let res = AddrModeWithAddr::from_str("($AA),Y");
        assert_eq!(Ok(AddrModeWithAddr { mode: AddrMode::ZeroPageIndirectIndexedWithY, arg: 0xaa, arg_size: 1 }), res);
    }
}
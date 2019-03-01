pub struct TwelveBits<I> {
    bytes: I,
    part: u16,
    bits: u8,
}

impl<I: Iterator<Item=u8>> TwelveBits<I> {
    pub fn new(bytes: I) -> Self {
        Self {
            bytes,
            part: 0,
            bits: 0,
        }
    }
}

impl<I: Iterator<Item=u8>> Iterator for TwelveBits<I> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let byte = self.bytes.next()?;
            self.part <<= 8;
            self.part |= u16::from(byte);
            self.bits += 8;
            match self.bits {
                16 => {
                    let result = (self.part & 0xFFF0) >> 4;
                    self.part &= 0x000F;
                    self.bits -= 12;
                    return Some(result);
                }
                12 => {
                    let result = self.part;
                    self.part = 0;
                    self.bits -= 12;
                    return Some(result);
                }
                8 | 4 => {
                    continue;
                }
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn test() {
    let bytes = [0x11, 0x12, 0x22, 0x33, 0x34];
    let res = TwelveBits::new(bytes.iter().cloned()).collect::<Vec<u16>>();
    assert_eq!(&res, &[0x111, 0x222, 0x333]);
}

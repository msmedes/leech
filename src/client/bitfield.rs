pub struct Bitfield {
    bitfield: Vec<u8>,
}

impl Bitfield {
    pub fn has_piece(&self, index: usize) -> bool {
        // figure out which byte we need to look inside
        let byte_index = index / 8;
        // figure out the bit offset inside of that bit
        let offset = index % 8;
        self.bitfield[byte_index] >> (7 - offset) & 1 != 0
    }

    pub fn set_piece(&mut self, index: usize) {
        let byte_index = index / 8;
        let offset = index % 8;
        self.bitfield[byte_index] |= 1 << (7 - offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_piece() {
        let bitfield = Bitfield {
            bitfield: vec![84, 84],
        }; // 01010100, 01010100
        let expected = [
            false, true, false, true, false, true, false, false, false, true, false, true, false,
            true, false, false,
        ];
        for i in 0..expected.len() {
            assert_eq!(bitfield.has_piece(i), expected[i]);
        }
    }

    #[test]
    fn set_piece() {
        let mut bitfield = Bitfield {
            bitfield: vec![84, 84],
        };

        // nothing should happen
        bitfield.set_piece(3);
        assert_eq!(bitfield.bitfield, vec![84, 84]);

        bitfield.set_piece(4);
        assert!(bitfield.has_piece(4));
    }

    #[test]
    #[should_panic]
    fn set_piece_index_out_of_bounds() {
        let mut bitfield = Bitfield {
            bitfield: vec![84, 84],
        };

        bitfield.set_piece(24);
    }
}

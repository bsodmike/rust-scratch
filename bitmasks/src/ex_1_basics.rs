#[cfg(test)]
mod bitmask {
    use bitmask_enum::bitmask;

    #[bitmask(u8)]
    enum Bitmask {
        Flag1, // defaults to 0b00000001

        CustomFlag3 = 0b00000100,

        Flag2, // defaults to 0b00000010
        Flag3, // defaults to 0b00000100

        // 4th bit
        Flag4 = 1 << 3,

        Flag13_1 = 0b00000001 | 0b00000100,
        Flag13_2 = Self::Flag1.or(Self::Flag3).bits,
        Flag13_3 = Self::Flag1.bits | Self::CustomFlag3.bits,

        Flag123 = {
            let flag13 = Self::Flag13_1.bits;
            flag13 | Self::Flag2.bits
        },
    }

    #[test]
    fn howto_define_bitmask() {
        // Using the bitwise `or` operator, we can set single bits in the bitmask.
        // Notice we have set the 1st and 3rd bits.
        let bm = Bitmask::Flag1 | Bitmask::Flag3;
        assert_eq!(bm, 0b00000101);

        // Equality comparison
        assert!(bm == Bitmask::Flag13_1);

        // We can set further bits like this
        assert_eq!(bm | Bitmask::Flag2, 0b00000111);

        // The crate has convenience methods too, that let you do this
        let bm123 = bm.or(Bitmask::Flag2).bits;
        assert_eq!(bm123, 0b00000111);
    }

    #[test]
    fn check_bit_is_set() {
        let bm = Bitmask::Flag2 | Bitmask::Flag4;
        let value = 3_u8; // 0b00000011

        // The 2nd bit is set
        assert!((value & bm.bits) > 0);

        // Same as
        assert!((value & bm.bits) != 0);
    }
}

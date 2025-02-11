const fn mask<const BITS: usize>() -> u8 {
    match BITS {
        1 => 0b1,
        2 => 0b11,
        3 => 0b111,
        4 => 0b1111,
        8 => u8::MAX,
        _ => unreachable!()
    }
}

const fn invert<const BITS: usize>() -> usize {
    match BITS {
        1 => 8,
        2 => 4,
        4 => 2,
        8 => 1,
        _ => unreachable!()
    }
}

const fn invert_mask<const BITS: usize>() -> u8 {
    match BITS {
        1 => mask::<8>(),
        2 => mask::<4>(),
        4 => mask::<2>(),
        8 => mask::<1>(),
        _ => unreachable!()
    }
}

const fn get_shift<const BITS: usize>(i: usize) -> u8 {
    match BITS {
        1 => i as u8 & mask::<3>(),
        2 => (i as u8 & mask::<2>()) << 1,
        4 => (i as u8 & mask::<1>()) << 2,
        8 => 0,
        _ => unreachable!()
    }
}

/// Will tell you how much u8 are required to store n values that are BITS bits long
const fn required_u8<const BITS: usize>(n: usize) -> usize {
    n/invert::<BITS>() + if n & invert_mask::<BITS>() as usize != 0 { 1 } else { 0 }
}

#[inline(always)]
fn vec_for<const BITS: usize>(n: usize) -> Vec<u8> {
    vec![0; required_u8::<BITS>(n)]
}

#[derive(Debug, Clone)]
pub enum PackedEnum {
    U1(Vec<u8>),
    U2(Vec<u8>),
    U4(Vec<u8>),
    U8(Vec<u8>),
}

impl PackedEnum {
    #[inline(always)]
    fn get(&self, i: usize) -> u8 {
        match self {
            Self::U1(data) => (data[i >> 3] >> get_shift::<1>(i)) & mask::<1>(),
            Self::U2(data) => (data[i >> 2] >> get_shift::<2>(i)) & mask::<2>(),
            Self::U4(data) => (data[i >> 1] >> get_shift::<4>(i)) & mask::<4>(),
            Self::U8(data) => data[i],
        }
    }

    #[inline(always)]
    fn set(&mut self, i: usize, value: u8) {
        if let Self::U8(data) = self {
            data[i] = value;
            return;
        }
        match self {
            Self::U1(data) => {
                let shift = get_shift::<1>(i);
                let i = i >> 3;
                data[i] &= !(mask::<1>() << shift);
                data[i] |= value << shift;
            },
            Self::U2(data) => {
                let shift = get_shift::<2>(i);
                let i = i >> 2;
                data[i] &= !(mask::<2>() << shift);
                data[i] |= value << shift;
            },
            Self::U4(data) => {
                let shift = get_shift::<4>(i);
                let i = i >> 1;
                data[i] &= !(mask::<4>() << shift);
                data[i] |= value << shift;
            }
            Self::U8(_) => unreachable!()
        }
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = u8> + 'a> {
        match self {
            Self::U1(data) => Box::new(data.iter().flat_map(|a| { [
                (a & mask::<1>()), ((a >> 1) & mask::<1>()), 
                ((a >> 2) & mask::<1>()), ((a >> 3) & mask::<1>()), 
                ((a >> 4) & mask::<1>()), ((a >> 5) & mask::<1>()), 
                ((a >> 6) & mask::<1>()), (a >> 7),
            ]})),
            Self::U2(data) => Box::new(data.iter().flat_map(|a| {
                [(a & mask::<2>()), ((a >> 2) & mask::<2>()), ((a >> 4) & mask::<2>()), (a >> 6)]
            })),
            Self::U4(data) => Box::new(data.iter().flat_map(|a| {
                [(a & mask::<4>()), (a >> 4)]
            })),
            Self::U8(data) => Box::new(data.iter().map(|a| *a)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackedU8s {
    pub data: PackedEnum,
    pub mask: u8,
    pub length: usize,
}

impl PackedU8s {
    pub fn new(length: usize) -> Self {
        let packed= PackedEnum::U1(vec![0; required_u8::<1>(length)]);
        Self { mask: mask::<1>(), data: packed, length }
    }

    pub fn bits_for(value: u8) -> u8 {
        value.max(1).ilog2() as u8 + 1
    }

    pub fn from(values: &[u8]) -> Self {
        let max = *values.iter().max().unwrap_or(&0);
        PackedU8s::from_with_bits(values, PackedU8s::bits_for(max))
    }

    pub fn from_with_bits(values: &[u8], bits: u8) -> Self {
        let length = values.len();
        if bits <= 1 {
            let mut res = vec_for::<1>(length);
            for (i, chunk) in values.chunks(8).enumerate() {
                for (j, value) in chunk.iter().enumerate() {
                    res[i] |= value << j;
                }
            }
            PackedU8s {
                data: PackedEnum::U1(res),
                length,
                mask: mask::<1>()
            }
        } else if bits <= 2 {
            let mut res = vec_for::<2>(length);
            for (i, chunk) in values.chunks(4).enumerate() {
                for (j, value) in chunk.iter().enumerate() {
                    res[i] |= value << (j << 1);
                }
            }
            PackedU8s {
                data: PackedEnum::U2(res),
                length,
                mask: mask::<2>()
            }
        } else if bits <= 4 {
            let mut res = vec_for::<4>(length);
            for (i, chunk) in values.chunks(2).enumerate() {
                for (j, value) in chunk.iter().enumerate() {
                    res[i] |= value << (j << 2);
                }
            }
            PackedU8s {
                data: PackedEnum::U4(res),
                length,
                mask: mask::<4>()
            }
        } else {
            PackedU8s {
                data: PackedEnum::U8(values.iter().map(|a| *a as u8).collect()),
                length,
                mask: mask::<8>()
            }
        }
    }

    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = u8> + 'a> {
        self.data.iter()
    }

    #[inline]
    pub fn get(&self, i: usize) -> u8 {
        self.data.get(i)
    }

    #[inline]
    fn upscale_if_needed(&mut self, value: u8) {
        if (value & self.mask) == value {
            return;
        }
        let values = self.data.iter().take(self.length).collect::<Vec<_>>();
        *self = PackedU8s::from_with_bits(&values, PackedU8s::bits_for(value));
    }

    #[inline]
    pub fn set(&mut self, i: usize, value: u8) {
        self.upscale_if_needed(value);
        self.data.set(i, value)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use crate::packed_u8::mask;
    use super::PackedU8s;

    fn test_equal(uints: &PackedU8s, values: &[u8]) {
        for (i, value) in values.iter().enumerate() {
            assert_eq!(*value, uints.get(i));
        }
    }

    fn roundtrip(uints: &mut PackedU8s, values: &[u8]) {
        for (i, value) in values.iter().enumerate() {
            uints.set(i, *value);
        }
        test_equal(uints, values);
    }

    #[test]
    pub fn test_from_iter() {
        let mut rng = rand::thread_rng();
        let values: [u8; 100] = [(); 100].map(|_| rng.gen_range(0..16));
        let uints = PackedU8s::from(&values);
        test_equal(&uints, &values);
    }

    fn test_ubits<const BITS: usize>() {
        let mut rng = rand::thread_rng();
        let mut uints = PackedU8s::new(100);
        let values: [u8; 100] = [(); 100].map(|_| rng.gen_range(0..2u32.pow(BITS as u32)) as u8);
        roundtrip(&mut uints, &values);
        assert!(uints.mask == mask::<BITS>());
    }

    #[test]
    pub fn test_u1() {
        test_ubits::<1>();
    }

    #[test]
    pub fn test_u2() {
        test_ubits::<2>();
    }

    #[test]
    pub fn test_u4() {
        test_ubits::<4>();
    }

    #[test]
    pub fn test_u8() {
        test_ubits::<8>();
    }
}
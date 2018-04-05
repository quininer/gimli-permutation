#![no_std]

extern crate byteorder;
extern crate gimli_permutation;

use core::{ cmp, mem };
use byteorder::{ ByteOrder, LittleEndian };
use gimli_permutation::{ S, gimli };


pub const RATE: usize = 16;
type State = [u8; S * 4];


#[derive(Clone)]
pub struct GimliHash {
    state: State,
    pos: usize
}

impl Default for GimliHash {
    fn default() -> Self {
        GimliHash { state: [0; S * 4], pos: 0 }
    }
}

impl GimliHash {
    #[inline]
    pub fn update(&mut self, buf: &[u8]) {
        self.absorb(buf);
    }

    #[inline]
    pub fn finalize(self, buf: &mut [u8]) {
        self.xof().squeeze(buf);
    }

    #[inline]
    pub fn xof(mut self) -> XofReader {
        self.pad();
        XofReader { state: self.state, pos: 0 }
    }

    #[inline]
    fn permutation(state: &mut State) {
        #[inline]
        fn array_as_block(arr: &mut [u8; S * 4]) -> &mut [u32; S] {
            unsafe { mem::transmute(arr) }
        }

        let state = array_as_block(state);
        LittleEndian::from_slice_u32(state);
        gimli(state);
        LittleEndian::from_slice_u32(state);
    }

    fn absorb(&mut self, buf: &[u8]) {
        let mut start = 0;
        let mut len = buf.len();

        while len > 0 {
            let take = cmp::min(RATE - self.pos, len);
            for (dst, &src) in self.state[self.pos..][..take].iter_mut()
                .zip(&buf[start..][..take])
            {
                *dst ^= src;
            }
            self.pos += take;
            start += take;
            len -= take;

            if self.pos == RATE {
                Self::permutation(&mut self.state);
                self.pos = 0;
            }
        }
    }

    fn pad(&mut self) {
        self.state[self.pos] ^= 0x1f;
        self.state[RATE - 1] ^= 0x80;
        Self::permutation(&mut self.state);
    }
}


pub struct XofReader {
    state: State,
    pos: usize
}

impl XofReader {
    pub fn squeeze(&mut self, buf: &mut [u8]) {
        let take = cmp::min(RATE - self.pos, buf.len());
        let (prefix, buf) = buf.split_at_mut(take);

        if !prefix.is_empty() {
            prefix.copy_from_slice(&self.state[self.pos..][..take]);
            self.pos += take;

            if self.pos == RATE {
                GimliHash::permutation(&mut self.state);
                self.pos = 0;
            }
        }

        for chunk in buf.chunks_mut(RATE) {
            let take = chunk.len();
            chunk.copy_from_slice(&self.state[self.pos..][..take]);

            if self.pos == RATE {
                GimliHash::permutation(&mut self.state);
            } else {
                self.pos += take;
            }
        }
    }
}
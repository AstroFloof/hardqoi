#![no_std]
#![allow(clippy::unusual_byte_groupings)]

extern crate alloc;
extern crate bytemuck;

pub use arch_switch::implementation::{decode::decode, encode::encode};
use common::*;

#[path = "./arch_switch.rs"]
mod arch_switch;

pub(crate) trait Hashing {
    fn update(&mut self, pixel_feed: &[RGBA]);
    unsafe fn fetch(&self, hash: HASH) -> &RGBA;
    unsafe fn fetch_mut(&mut self, hash: HASH) -> &mut RGBA;
    fn swap(&mut self, pixel: &RGBA) -> (RGBA, HASH);
}

pub mod common {
    use alloc::vec::Vec;
    use core::convert::TryInto;

    #[cfg(feature = "image_compat")]
    use image::{DynamicImage, GenericImageView};

    pub const MAGIC_QOIF: [u8; 4] = *b"qoif";
    pub const QOI_OP_RGBA: u8 = 0xff_u8;
    pub const QOI_OP_RGB: u8 = 0xfe_u8;
    pub const QOI_OP_INDEX: u8 = 0b00_000000_u8;
    pub const QOI_OP_DIFF: u8 = 0b01_000000_u8;
    pub const QOI_OP_LUMA: u8 = 0b10_000000_u8;
    pub const QOI_OP_RUN: u8 = 0b11_000000_u8;
    pub const END_8: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
    pub type RGBA = u32; // four bytes pixel
    pub type HASH = u8; // 6 bit pixel hash
    pub type SBPX = u8; // byte subpixel
    pub type HashIndexedArray = [RGBA; 64];

    #[derive(Clone, Copy)]
    pub struct QOIHeader {
        pub width: u32,
        pub height: u32,
        pub has_alpha: bool,
        pub linear_rgb: bool,
    }

    #[cfg(feature = "image_compat")]
    impl From<&DynamicImage> for QOIHeader {
        fn from(img: &DynamicImage) -> Self {
            let (width, height) = img.dimensions();
            Self {
                width,
                height,
                has_alpha: img.color().has_alpha(),
                linear_rgb: false, // TODO detect or parameterize this somehow
            }
        }
    }

    impl From<&[u8]> for QOIHeader {
        fn from(bytes: &[u8]) -> Self {
            assert_eq!(
                bytes[0..4],
                MAGIC_QOIF,
                "Data is not a QOI image (magic bytes \"qoif\" not found)"
            );
            Self {
                width: u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
                height: u32::from_be_bytes(bytes[8..12].try_into().unwrap()),
                has_alpha: bytes[12] - 3 == 1,
                linear_rgb: bytes[13] > 0,
            }
        }
    }

    impl QOIHeader {
        pub fn to_bytes(&self) -> Vec<u8> {
            let mut bytes = Vec::with_capacity(14);
            bytes.extend(MAGIC_QOIF);
            bytes.extend(self.width.to_be_bytes());
            bytes.extend(self.height.to_be_bytes());
            bytes.push(self.has_alpha as u8 + 3u8);
            bytes.push(self.linear_rgb as u8);
            bytes
        }

        pub fn image_size(&self) -> usize {
            self.width as usize * self.height as usize
        }
    }
}

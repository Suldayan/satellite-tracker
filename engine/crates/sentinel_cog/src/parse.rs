use log::debug;
use crate::error::{CogError, CogResult};
use crate::fetch::fetch_range;

pub const TAG_IMAGE_WIDTH: u16 = 256;
pub const TAG_IMAGE_LENGTH: u16 = 257;
pub const TAG_COMPRESSION: u16 = 259;
pub const TAG_TILE_WIDTH: u16 = 322;
pub const TAG_TILE_LENGTH: u16 = 323;
pub const TAG_TILE_OFFSETS: u16 = 324;
pub const TAG_TILE_BYTE_COUNTS: u16 = 325;
pub const TAG_SUB_IFDS: u16 = 330;
pub const TAG_PIXEL_SCALE: u16 = 33550;
pub const TAG_MODEL_TIEPOINT: u16 = 33922;

const TYPE_SHORT: u16 = 3;
const TYPE_LONG: u16 = 4;
const TYPE_LONG8: u16 = 16;
const TYPE_DOUBLE: u16 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TiffTag {
    ImageWidth,
    ImageLength,
    Compression,
    TileWidth,
    TileLength,
    TileOffsets,
    TileByteCounts,
    SubIFDs,
    PixelScale,
    ModelTiepoint,
    Unknown(u16),
}

impl TiffTag {
    pub fn from_raw(tag: u16) -> Self {
        match tag {
            TAG_IMAGE_WIDTH => Self::ImageWidth,
            TAG_IMAGE_LENGTH => Self::ImageLength,
            TAG_COMPRESSION => Self::Compression,
            TAG_TILE_WIDTH => Self::TileWidth,
            TAG_TILE_LENGTH => Self::TileLength,
            TAG_TILE_OFFSETS => Self::TileOffsets,
            TAG_TILE_BYTE_COUNTS => Self::TileByteCounts,
            TAG_SUB_IFDS => Self::SubIFDs,
            TAG_PIXEL_SCALE => Self::PixelScale,
            TAG_MODEL_TIEPOINT => Self::ModelTiepoint,
            other => Self::Unknown(other),
        }
    }

    pub fn as_raw(self) -> u16 {
        match self {
            Self::ImageWidth => TAG_IMAGE_WIDTH,
            Self::ImageLength => TAG_IMAGE_LENGTH,
            Self::Compression => TAG_COMPRESSION,
            Self::TileWidth => TAG_TILE_WIDTH,
            Self::TileLength => TAG_TILE_LENGTH,
            Self::TileOffsets => TAG_TILE_OFFSETS,
            Self::TileByteCounts => TAG_TILE_BYTE_COUNTS,
            Self::SubIFDs => TAG_SUB_IFDS,
            Self::PixelScale => TAG_PIXEL_SCALE,
            Self::ModelTiepoint => TAG_MODEL_TIEPOINT,
            Self::Unknown(v) => v,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GeoTransform {
    pub origin_x: f64,
    pub origin_y: f64,
    pub pixel_x: f64,
    pub pixel_y: f64,
}

pub struct IfdInfo {
    pub tile_offsets: Vec<(u64, u64)>,
    pub img_w: u32,
    pub img_h: u32,
    pub tile_w: u32,
    pub tile_h: u32,
    pub tiles_across: u32,
    pub geo: Option<GeoTransform>,
}

pub fn is_little_endian(data: &[u8]) -> CogResult<bool> {
    match data.get(0..2) {
        Some(b"II") => Ok(true),
        Some(b"MM") => Ok(false),
        _ => Err(CogError::InvalidHeader("Expected 'II' or 'MM' at offset 0".into())),
    }
}

fn read_u16(data: &[u8], off: usize, le: bool) -> CogResult<u16> {
    let s = data.get(off..off + 2).ok_or(CogError::OutOfBounds { offset: off, size: 2 })?;
    Ok(if le { u16::from_le_bytes([s[0], s[1]]) } else { u16::from_be_bytes([s[0], s[1]]) })
}

fn read_u32(data: &[u8], off: usize, le: bool) -> CogResult<u32> {
    let s = data.get(off..off + 4).ok_or(CogError::OutOfBounds { offset: off, size: 4 })?;
    Ok(if le {
        u32::from_le_bytes([s[0], s[1], s[2], s[3]])
    } else {
        u32::from_be_bytes([s[0], s[1], s[2], s[3]])
    })
}

fn read_u64(data: &[u8], off: usize, le: bool) -> CogResult<u64> {
    let s = data.get(off..off + 8).ok_or(CogError::OutOfBounds { offset: off, size: 8 })?;
    Ok(if le {
        u64::from_le_bytes(s.try_into().unwrap())
    } else {
        u64::from_be_bytes(s.try_into().unwrap())
    })
}

fn read_f64(data: &[u8], off: usize, le: bool) -> CogResult<f64> {
    let s = data.get(off..off + 8).ok_or(CogError::OutOfBounds { offset: off, size: 8 })?;
    Ok(if le {
        f64::from_le_bytes(s.try_into().unwrap())
    } else {
        f64::from_be_bytes(s.try_into().unwrap())
    })
}

fn read_typed(data: &[u8], off: usize, type_id: u16, tag: u16, le: bool) -> CogResult<u64> {
    match type_id {
        TYPE_SHORT => Ok(read_u16(data, off, le)? as u64),
        TYPE_LONG => Ok(read_u32(data, off, le)? as u64),
        TYPE_LONG8 => read_u64(data, off, le),
        other => Err(CogError::UnsupportedTagType { tag, type_id: other }),
    }
}

fn type_size(type_id: u16, tag: u16) -> CogResult<usize> {
    match type_id {
        TYPE_SHORT => Ok(2),
        TYPE_LONG => Ok(4),
        TYPE_LONG8 => Ok(8),
        TYPE_DOUBLE => Ok(8),
        other => Err(CogError::UnsupportedTagType { tag, type_id: other }),
    }
}

fn read_inline_values(ifd: &[u8], entry: usize, type_id: u16, count: usize, le: bool) -> CogResult<Vec<u64>> {
    let sz = type_size(type_id, 0)?;
    (0..count).map(|j| read_typed(ifd, entry + 8 + j * sz, type_id, 0, le)).collect()
}

fn read_ext_values(
    client: &reqwest::blocking::Client,
    url: &str,
    ifd: &[u8],
    entry: usize,
    type_id: u16,
    tag: u16,
    count: usize,
    le: bool,
) -> CogResult<Vec<u64>> {
    let sz = type_size(type_id, tag)?;
    if count * sz <= 4 {
        return read_inline_values(ifd, entry, type_id, count, le);
    }
    let ext_offset = read_u32(ifd, entry + 8, le)? as u64;
    let ext_end = ext_offset + (count * sz) as u64 - 1;
    debug!("Tag {tag}: fetching {count} ext values at bytes={ext_offset}-{ext_end}");
    let ext = fetch_range(client, url, ext_offset, ext_end)?;
    (0..count).map(|j| read_typed(&ext, j * sz, type_id, tag, le)).collect()
}

fn read_f64_values(
    client: &reqwest::blocking::Client,
    url: &str,
    ifd: &[u8],
    entry: usize,
    count: usize,
    le: bool,
) -> CogResult<Vec<f64>> {
    let ext_offset = read_u32(ifd, entry + 8, le)? as u64;
    let ext_end = ext_offset + (count * 8) as u64 - 1;
    let ext = fetch_range(client, url, ext_offset, ext_end)?;
    (0..count).map(|j| read_f64(&ext, j * 8, le)).collect()
}

pub fn parse_subifds(header: &[u8]) -> CogResult<Vec<u32>> {
    let le = is_little_endian(header)?;
    let ifd_offset = read_u32(header, 4, le)? as usize;
    let entry_count = read_u16(header, ifd_offset, le)? as usize;
    debug!("Main IFD at offset {ifd_offset}, {entry_count} entries");

    for i in 0..entry_count {
        let entry = ifd_offset + 2 + i * 12;
        let tag = read_u16(header, entry, le)?;
        let type_id = read_u16(header, entry + 2, le)?;
        let count = read_u32(header, entry + 4, le)? as usize;

        if tag != TAG_SUB_IFDS { continue; }

        let sz = type_size(type_id, tag)?;
        let offsets = read_inline_values(header, entry, type_id, count, le)?
            .into_iter()
            .map(|v| v as u32)
            .collect();

        debug!("Found {count} SubIFD(s) at tag 330 (type={type_id}, sz={sz})");
        return Ok(offsets);
    }

    debug!("No SubIFD tag found, falling back to main IFD at {ifd_offset}");
    Ok(vec![ifd_offset as u32])
}

pub fn parse_ifd_bytes(
    client: &reqwest::blocking::Client,
    url: &str,
    ifd: &[u8],
    le: bool,
) -> CogResult<IfdInfo> {
    let entry_count = read_u16(ifd, 0, le)? as usize;
    debug!("Parsing IFD: {entry_count} entries");

    let mut img_w: Option<u32> = None;
    let mut img_h: Option<u32> = None;
    let mut tile_w: Option<u32> = None;
    let mut tile_h: Option<u32> = None;
    let mut offsets: Option<Vec<u64>> = None;
    let mut byte_counts: Option<Vec<u64>> = None;
    let mut pixel_scale: Option<Vec<f64>> = None;
    let mut tiepoint: Option<Vec<f64>> = None;

    for i in 0..entry_count {
        let entry = 2 + i * 12;
        if entry + 12 > ifd.len() { break; }

        let tag = read_u16(ifd, entry, le)?;
        let type_id = read_u16(ifd, entry + 2, le)?;
        let count = read_u32(ifd, entry + 4, le)? as usize;

        if type_size(type_id, tag).is_err() { continue; }

        match tag {
            TAG_IMAGE_WIDTH | TAG_IMAGE_LENGTH | TAG_TILE_WIDTH | TAG_TILE_LENGTH => {
                let val = read_inline_values(ifd, entry, type_id, 1, le)?
                    .into_iter().next()
                    .ok_or(CogError::MissingTag { tag, name: tag_name(tag) })? as u32;
                match tag {
                    TAG_IMAGE_WIDTH => img_w = Some(val),
                    TAG_IMAGE_LENGTH => img_h = Some(val),
                    TAG_TILE_WIDTH => tile_w = Some(val),
                    TAG_TILE_LENGTH => tile_h = Some(val),
                    _ => unreachable!(),
                }
            }
            TAG_COMPRESSION => {
                let val = read_inline_values(ifd, entry, type_id, 1, le)?.into_iter().next().unwrap_or(1);
                debug!("Compression: {val}");
            }
            TAG_TILE_OFFSETS => {
                offsets = Some(read_ext_values(client, url, ifd, entry, type_id, tag, count, le)?);
            }
            TAG_TILE_BYTE_COUNTS => {
                byte_counts = Some(read_ext_values(client, url, ifd, entry, type_id, tag, count, le)?);
            }
            TAG_PIXEL_SCALE => {
                pixel_scale = Some(read_f64_values(client, url, ifd, entry, count, le)?);
            }
            TAG_MODEL_TIEPOINT => {
                tiepoint = Some(read_f64_values(client, url, ifd, entry, count, le)?);
            }
            _ => {}
        }
    }

    let offsets = offsets.ok_or(CogError::MissingTag { tag: TAG_TILE_OFFSETS, name: "TileOffsets" })?;
    let byte_counts = byte_counts.ok_or(CogError::MissingTag { tag: TAG_TILE_BYTE_COUNTS, name: "TileByteCounts" })?;
    let img_w = img_w.ok_or(CogError::MissingTag { tag: TAG_IMAGE_WIDTH, name: "ImageWidth" })?;
    let img_h = img_h.ok_or(CogError::MissingTag { tag: TAG_IMAGE_LENGTH, name: "ImageLength" })?;
    let tile_w = tile_w.ok_or(CogError::MissingTag { tag: TAG_TILE_WIDTH, name: "TileWidth" })?;
    let tile_h = tile_h.ok_or(CogError::MissingTag { tag: TAG_TILE_LENGTH, name: "TileLength" })?;

    if offsets.len() != byte_counts.len() {
        return Err(CogError::TileLengthMismatch {
            expected: offsets.len(),
            actual: byte_counts.len(),
        });
    }

    let geo = match (pixel_scale, tiepoint) {
        (Some(scale), Some(tp)) if scale.len() >= 2 && tp.len() >= 6 => {
            debug!("GeoTransform: origin=({}, {}), pixel=({}, {})", tp[3], tp[4], scale[0], scale[1]);
            Some(GeoTransform {
                origin_x: tp[3],
                origin_y: tp[4],
                pixel_x: scale[0],
                pixel_y: -scale[1],
            })
        }
        _ => {
            debug!("GeoTransform tags absent — bbox filtering unavailable");
            None
        }
    };

    let tiles_across = (img_w + tile_w - 1) / tile_w;

    Ok(IfdInfo {
        tile_offsets: offsets.into_iter().zip(byte_counts).collect(),
        img_w, img_h, tile_w, tile_h, tiles_across, geo,
    })
}

fn tag_name(tag: u16) -> &'static str {
    match tag {
        TAG_IMAGE_WIDTH => "ImageWidth",
        TAG_IMAGE_LENGTH => "ImageLength",
        TAG_COMPRESSION => "Compression",
        TAG_TILE_WIDTH => "TileWidth",
        TAG_TILE_LENGTH => "TileLength",
        TAG_TILE_OFFSETS => "TileOffsets",
        TAG_TILE_BYTE_COUNTS => "TileByteCounts",
        TAG_SUB_IFDS => "SubIFDs",
        TAG_PIXEL_SCALE => "PixelScale",
        TAG_MODEL_TIEPOINT => "ModelTiepoint",
        _ => "Unknown",
    }
}
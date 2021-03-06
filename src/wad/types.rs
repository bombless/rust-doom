pub use super::name::{WadName, WadNameCast};

pub type LightLevel = i16;
pub type LinedefFlags = u16;
pub type LinedefType = u16;
pub type SectorId = u16;
pub type SectorTag = u16;
pub type SectorType = u16;
pub type SidedefId = i16;
pub type SpecialType = u16;
pub type ThingFlags = u16;
pub type ThingType = u16;
pub type VertexId = u16;
pub type WadCoord = i16;
pub type SegId = u16;
pub type LinedefId = u16;
pub type ChildId = u16;


#[repr(packed)]
#[repr(C)]
#[derive(Copy)]
pub struct WadInfo {
    pub identifier       : [u8; 4],
    pub num_lumps        : i32,
    pub info_table_offset: i32,
}


#[repr(packed)]
#[repr(C)]
#[derive(Copy)]
pub struct WadLump {
    pub file_pos: i32,
    pub size    : i32,
    pub name    : WadName,
}


#[repr(packed)]
#[repr(C)]
#[derive(Copy)]
pub struct WadThing {
    pub x: WadCoord,
    pub y: WadCoord,
    pub angle: WadCoord,
    pub thing_type: ThingType,
    pub flags: ThingFlags,
}


#[repr(packed)]
#[repr(C)]
#[derive(Copy)]
pub struct WadVertex {
    pub x: WadCoord,
    pub y: WadCoord,
}


#[repr(packed)]
#[repr(C)]
#[derive(Copy)]
pub struct WadLinedef {
    pub start_vertex: VertexId,
    pub end_vertex: VertexId,
    pub flags: LinedefFlags,
    pub special_type: SpecialType,
    pub sector_tag: SectorTag,
    pub right_side: SidedefId,
    pub left_side: SidedefId,
}


#[repr(packed)]
#[repr(C)]
#[derive(Copy)]
pub struct WadSidedef {
    pub x_offset: WadCoord,
    pub y_offset: WadCoord,
    pub upper_texture: WadName,
    pub lower_texture: WadName,
    pub middle_texture: WadName,
    pub sector: SectorId,
}


#[repr(packed)]
#[repr(C)]
#[derive(Copy)]
pub struct WadSector {
    pub floor_height: WadCoord,
    pub ceiling_height: WadCoord,
    pub floor_texture: WadName,
    pub ceiling_texture: WadName,
    pub light: LightLevel,
    pub sector_type: SectorType,
    pub tag: SectorTag,
}


#[repr(packed)]
#[repr(C)]
#[derive(Copy)]
pub struct WadSubsector {
    pub num_segs: u16,
    pub first_seg: SegId,
}


#[repr(packed)]
#[repr(C)]
#[derive(Copy)]
pub struct WadSeg {
    pub start_vertex: VertexId,
    pub end_vertex: VertexId,
    pub angle: u16,
    pub linedef: LinedefId,
    pub direction: u16,
    pub offset: u16,
}


#[repr(packed)]
#[repr(C)]
#[derive(Copy)]
pub struct WadNode {
    pub line_x: WadCoord,
    pub line_y: WadCoord,
    pub step_x: WadCoord,
    pub step_y: WadCoord,
    pub right_y_max: WadCoord,
    pub right_y_min: WadCoord,
    pub right_x_max: WadCoord,
    pub right_x_min: WadCoord,
    pub left_y_max: WadCoord,
    pub left_y_min: WadCoord,
    pub left_x_max: WadCoord,
    pub left_x_min: WadCoord,
    pub right: ChildId,
    pub left: ChildId
}


#[repr(packed)]
#[repr(C)]
#[derive(Copy)]
pub struct WadTextureHeader {
    pub name: WadName,
    pub masked: u32,
    pub width: u16,
    pub height: u16,
    pub column_directory: u32,
    pub num_patches: u16
}


#[repr(packed)]
#[repr(C)]
#[derive(Copy)]
pub struct WadTexturePatchRef {
    pub origin_x: i16,
    pub origin_y: i16,
    pub patch: u16,
    pub stepdir: u16,
    pub colormap: u16,
}

impl WadLinedef {
    pub fn impassable(&self) -> bool { self.flags & 0x0001 != 0 }
    pub fn blocks_monsters(&self) -> bool { self.flags & 0x0002 != 0 }
    pub fn is_two_sided(&self) -> bool { self.flags & 0x0004 != 0 }
    pub fn upper_unpegged(&self) -> bool { self.flags & 0x0008 != 0 }
    pub fn lower_unpegged(&self) -> bool { self.flags & 0x0010 != 0 }
    pub fn secret(&self) -> bool { self.flags & 0x0020 != 0 }
    pub fn blocks_sound(&self) -> bool { self.flags & 0x0040 != 0 }
    pub fn always_shown_on_map(&self) -> bool { self.flags & 0x0080 != 0 }
    pub fn never_shown_on_map(&self) -> bool { self.flags & 0x0100 != 0 }
}

macro_rules! size_tests(
    ($($t:ident = $size:expr),+) => (
        #[cfg(test)]
        #[allow(non_snake_case)]
        mod size_tests {
            use std::mem::size_of;
            $(
                #[test]
                fn $t() {
                    use super::$t;
                    assert_eq!(size_of::<$t>(), $size);
                }
            )+
        }
    )
);

size_tests! {
    WadInfo = 12,
    WadLump = 16,
    WadThing = 10,
    WadVertex = 4,
    WadLinedef = 14,
    WadSidedef = 30,
    WadSector = 26,
    WadSubsector = 4,
    WadSeg = 12,
    WadNode = 28,
    WadTextureHeader = 22,
    WadTexturePatchRef = 10
}

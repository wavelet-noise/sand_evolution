pub type PointType = u16;
pub type IndexType = usize;

use cgmath::Point2;

pub const SECTOR_SIZE: Point2<PointType> = Point2::new(1024, 512);

pub fn point_to_index(pos: Point2<PointType>) -> IndexType {
    pos.y as IndexType * SECTOR_SIZE.x as IndexType + pos.x as IndexType
}

pub fn tuple_to_index(pos: (PointType, PointType)) -> IndexType {
    pos.1 as IndexType * SECTOR_SIZE.x as IndexType + pos.0 as IndexType
}

pub fn xy_to_index(i: PointType, j: PointType) -> IndexType {
    j as IndexType * SECTOR_SIZE.x as IndexType + i as IndexType
}

pub fn index_to_cell(index: IndexType) -> Point2<PointType> {
    Point2::new(
        (index % SECTOR_SIZE.y as IndexType) as PointType,
        (index / SECTOR_SIZE.y as IndexType) as PointType,
    )
}

// Система температуры по секциям
// Размер секции для температуры (32x32 пикселя)
pub const TEMP_SECTION_SIZE: PointType = 32;

pub fn get_temp_section_coords(i: PointType, j: PointType) -> (PointType, PointType) {
    (i / TEMP_SECTION_SIZE, j / TEMP_SECTION_SIZE)
}

pub fn get_temp_section_index(i: PointType, j: PointType) -> usize {
    let (sx, sy) = get_temp_section_coords(i, j);
    let sections_x = (SECTOR_SIZE.x + TEMP_SECTION_SIZE - 1) / TEMP_SECTION_SIZE;
    (sy as usize * sections_x as usize) + sx as usize
}

pub fn get_temp_sections_count() -> (usize, usize) {
    let sections_x = ((SECTOR_SIZE.x + TEMP_SECTION_SIZE - 1) / TEMP_SECTION_SIZE) as usize;
    let sections_y = ((SECTOR_SIZE.y + TEMP_SECTION_SIZE - 1) / TEMP_SECTION_SIZE) as usize;
    (sections_x, sections_y)
}

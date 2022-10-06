pub type PointType = u16;
pub type IndexType = usize;

use cgmath::Point2;

pub const SECTOR_SIZE: Point2<PointType> = Point2::new(1024, 512);

pub fn point_to_index(pos : Point2<PointType>) -> IndexType
{
    return pos.y as IndexType * SECTOR_SIZE.x as IndexType + pos.x as IndexType;
}

pub fn tuple_to_index(pos : (PointType, PointType)) -> IndexType
{
    return pos.1 as IndexType * SECTOR_SIZE.x as IndexType + pos.0 as IndexType;
}

pub fn xy_to_index(i : PointType, j : PointType) -> IndexType
{
    return j as IndexType * SECTOR_SIZE.x as IndexType + i as IndexType;
}

pub fn index_to_cell(index : IndexType) -> Point2<PointType>
{
    return Point2::new((index % SECTOR_SIZE.y as IndexType) as PointType, (index / SECTOR_SIZE.y as IndexType) as PointType);
}

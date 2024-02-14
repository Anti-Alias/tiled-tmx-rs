use std::collections::hash_map::Iter as HashMapIter;
use std::collections::HashMap;
use std::io::Read;
use roxmltree::{Document, Node};
use crate::{FillMode, Grid, Image, ObjectAlignment, Result, Tile, TileData, TileOffset, TileRenderSize};


#[derive(Clone, Default, Debug)]
pub struct Tileset {
    name: String,
    class: String,
    tile_width: u32,
    tile_height: u32,
    spacing: u32,
    margin: u32,
    tile_count: u32,
    columns: u32,
    object_alignment: ObjectAlignment,
    tile_render_size: TileRenderSize,
    fill_mode: FillMode,
    tile_offset: TileOffset,
    grid: Option<Grid>,
    image: Option<Image>,
    tiles: HashMap<u32, TileData>,
}

impl Tileset {
    pub fn name(&self) -> &str { &self.name }
    pub fn class(&self) -> &str { &self.class }
    pub fn tile_width(&self) -> u32 { self.tile_width }
    pub fn tile_height(&self) -> u32 { self.tile_height }
    pub fn spacing(&self) -> u32 { self.spacing }
    pub fn margin(&self) -> u32 { self.margin }
    pub fn tile_count(&self) -> u32 { self.tile_count }
    pub fn columns(&self) -> u32 { self.columns }
    pub fn object_alignment(&self) -> ObjectAlignment { self.object_alignment }
    pub fn tile_render_size(&self) -> TileRenderSize { self.tile_render_size }
    pub fn fill_mode(&self) -> FillMode { self.fill_mode }
    pub fn tile_offset(&self) -> TileOffset { self.tile_offset }
    pub fn grid(&self) -> Option<Grid> { self.grid }
    pub fn image(&self) -> Option<&Image> { self.image.as_ref() }
    pub fn tiles(&self) -> Tiles<'_> {
        Tiles {
            tileset: self,
            iter: self.tiles.iter(),
        }
    }

    /// Gets a tile using its local id.
    /// None if not found.
    pub fn tile(&self, id: u32) -> Option<Tile<'_>> {
        self.tiles.get(&id).map(|data| Tile::new(id, self, data))
    }

    /// Gets a tile using its x,y coordinates in the tileset.
    /// None if out of bounds.
    /// None if this is an image collection tileset.
    pub fn tile_at(&self, x: u32, y: u32) -> Option<Tile<'_>> {
        if self.image.is_none() { return None }
        if x > self.columns { return None }
        let id = y * self.columns + x;
        self.tile(id)
    }

    pub fn parse(mut read: impl Read) -> Result<Self> {
        let mut xml_str = String::new();
        read.read_to_string(&mut xml_str)?;
        Self::parse_str(&xml_str)
    }

    pub fn parse_str(xml_str: &str) -> Result<Self> {
        let mut result = Tileset::default();
        let xml_doc = Document::parse(&xml_str)?;
        let root = xml_doc.root();
        for node in root.children() {
            match node.tag_name().name() {
                "tileset" => result.parse_node(node)?,
                _ => {}
            }
        }
        Ok(result)
    }

    pub(crate) fn parse_node(&mut self, tileset_node: Node) -> Result<()> {

        // Attributes.
        for attr in tileset_node.attributes() {
            match attr.name() {
                "name" => self.name = String::from(attr.value()),
                "class" => self.class = String::from(attr.value()),
                "tilewidth" => self.tile_width = attr.value().parse()?,
                "tileheight" => self.tile_height = attr.value().parse()?,
                "spacing" => self.spacing = attr.value().parse()?,
                "margin" => self.margin = attr.value().parse()?,
                "tilecount" => self.tile_count = attr.value().parse()?,
                "columns" => self.columns = attr.value().parse()?,
                "objectalignment" => self.object_alignment = ObjectAlignment::parse(attr.value())?,
                "tilerendersize" => self.tile_render_size = TileRenderSize::parse(attr.value())?,
                "fillmode" => self.fill_mode = FillMode::parse(attr.value())?,
                _ => {}
            }
        }

        // If the tileset is based on a single image (which it usually is), ensure that every tile is populated.
        // Only image collection tilesets have id gaps.
        let image = parse_image(tileset_node)?;
        if let Some(image) = image {
            self.image = Some(image);
            for id in 0..self.tile_count {
                self.tiles.insert(id, TileData::default());
            }
        }

        // Process children
        for child in tileset_node.children() {
            match child.tag_name().name() {
                "tileoffset" => self.tile_offset = TileOffset::parse(child)?,
                "grid" => self.grid = Some(Grid::parse(child)?),
                "tile" => {
                    let (id, data) = TileData::parse(child)?;
                    self.tiles.insert(id, data);
                },
                _ => {}
            }
        }
        Ok(())
    }
}

fn parse_image(tileset_node: Node) -> Result<Option<Image>> {
    for child in tileset_node.children() {
        if child.tag_name().name() == "image" {
            let image = Image::parse(child)?;
            return Ok(Some(image))
        }
    }
    Ok(None)
}

/// Iterator over tiles in a tileset.
pub struct Tiles<'a> {
    tileset: &'a Tileset,
    iter: HashMapIter<'a, u32, TileData>,
}
impl<'a> Iterator for Tiles<'a> {
    type Item = Tile<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(id, data)| Tile::new(*id, self.tileset, data))
    }
}


#[cfg(test)]
mod test {
    use crate::Tileset;

    #[test]
    fn test_tileset() {
        let xml = include_str!("test_data/tilesets/vikings_of_midgard.tsx");
        let tileset = Tileset::parse_str(xml).unwrap();
        assert!(tileset.image.is_some());
        println!("{tileset:#?}");

        // ------- Tests fetching tiles by id -------
        let steve_tile = tileset.tile(0).unwrap();
        let is_steve = steve_tile.properties().get("is_steve").unwrap().as_bool().unwrap();
        assert_eq!(true, is_steve);

        let notsteve_tile = tileset.tile(1).unwrap();
        let is_steve = notsteve_tile.properties().get("is_steve").unwrap().as_bool().unwrap();
        assert_eq!(false, is_steve);

        let jerry_tile = tileset.tile(22).unwrap();
        let is_jerry = jerry_tile.properties().get("is_jerry").unwrap().as_bool().unwrap();
        assert_eq!(true, is_jerry);

        // ------- Tests fetching tiles by coordinates -------
        let steve_tile = tileset.tile_at(0, 0).unwrap();
        let is_steve = steve_tile.properties().get("is_steve").unwrap().as_bool().unwrap();
        assert_eq!(true, is_steve);

        let notsteve_tile = tileset.tile_at(1, 0).unwrap();
        let is_steve = notsteve_tile.properties().get("is_steve").unwrap().as_bool().unwrap();
        assert_eq!(false, is_steve);

        let jerry_tile = tileset.tile_at(6, 1).unwrap();
        let is_jerry = jerry_tile.properties().get("is_jerry").unwrap().as_bool().unwrap();
        assert_eq!(true, is_jerry);
    }

    #[test]
    fn test_collection_tileset() {
        let xml = include_str!("test_data/tilesets/collection.tsx");
        let tileset = Tileset::parse_str(xml).unwrap();
        assert!(tileset.image.is_none());

        // ------- Tests fetching tiles by id -------
        let steve_tile = tileset.tile(0).unwrap();
        let is_steve = steve_tile.properties().get("is_steve").unwrap().as_bool().unwrap();
        assert_eq!(true, is_steve);

        let notsteve_tile = tileset.tile(1).unwrap();
        let is_steve = notsteve_tile.properties().get("is_steve").unwrap().as_bool().unwrap();
        assert_eq!(false, is_steve);

        // ------- Tests fetching tiles by coordinates (always none since it's an image collection) -------
        assert_eq!(true, tileset.tile_at(0, 0).is_none());
        assert_eq!(true, tileset.tile_at(1, 0).is_none());
        assert_eq!(true, tileset.tile_at(2, 2).is_none());
    }

    #[test]
    fn test_tileset_animation() {
        let xml = include_str!("test_data/tilesets/vikings_of_midgard.tsx");
        let tileset = Tileset::parse_str(xml).unwrap();

        let tile = tileset.tile(144).unwrap();
        assert!(tile.animation().is_some());

        let tile = tileset.tile(145).unwrap();
        assert!(tile.animation().is_none());
    }

    #[test]
    fn test_tileset_objects() {
        let xml = include_str!("test_data/tilesets/shape.tsx");
        let tileset = Tileset::parse_str(xml).unwrap();
        let tile = tileset.tile(72).unwrap();
        let objects = tile.objects().unwrap();
        assert_eq!(2, objects.objects().len());
        assert_eq!(8.37916, objects.objects()[1].x());
    }
}
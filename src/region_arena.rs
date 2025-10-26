use crate::region::{Region, RegionId};

pub struct RegionArena { regions: Vec<Region> }
impl RegionArena {
    pub fn new() -> Self { Self { regions: vec![] } }
    pub fn alloc(&mut self, r: Region) -> RegionId { let id = self.regions.len(); self.regions.push(r); id}
    pub fn get(&self, id: RegionId) -> &Region { &self.regions[id] }
}
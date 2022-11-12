

#[derive(Debug)]
pub struct PositionsUpdate {
    pub ateampositions: Vec<(i32, f32, f32)>,
    pub bteampositions: Vec<(i32, f32, f32)>,
}

impl PositionsUpdate {

    pub fn new() -> PositionsUpdate {
        PositionsUpdate { ateampositions: Vec::new(), bteampositions: Vec::new() }
    }

}


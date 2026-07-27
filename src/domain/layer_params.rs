use crate::domain::LayerEntry;

#[derive(Clone, Debug)]
pub struct LayerParams {
    pub cena: String,
    pub layers: Vec<LayerEntry>,
    pub selected: usize,
}

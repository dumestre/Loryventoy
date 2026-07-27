#[derive(Clone, Debug)]
pub struct AnimParams {
    pub alvo: u8,
    pub loop_mode: u8,
    pub segmentos: Vec<crate::domain::AnimSeg>,
}

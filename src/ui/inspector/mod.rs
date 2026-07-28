// Re-exports from node_component to begin the inspector modularization.
// Individual node editors will be split into separate files in subsequent iterations.
pub use crate::ui::node_component::{
    AcaoInspector,
    show_content,
    render_layer_header,
    render_layer_row,
    linha_y,
    registrar_medida,
    escalar_estilo,
    content_size,
    draggable_value,
    PRESETS_RESOLUCAO,
    MARGEM_X,
    MARGEM_Y,
    CABECALHO_H,
    FONTE_TITULO,
};
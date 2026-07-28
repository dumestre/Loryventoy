// Re-exports from node_component to begin the inspector modularization.
// Individual node editors will be split into separate files in subsequent iterations.
pub use crate::ui::node_component::{
    content_size, draggable_value, escalar_estilo, linha_y, registrar_medida, render_layer_header,
    render_layer_row, show_content, AcaoInspector, CABECALHO_H, FONTE_TITULO, MARGEM_X, MARGEM_Y,
    PRESETS_RESOLUCAO,
};

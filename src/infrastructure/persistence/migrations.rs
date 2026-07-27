use super::format::ProjetoArquivo;

pub const VERSAO_ATUAL: u32 = 1;

pub fn migrate(arquivo: &mut ProjetoArquivo) {
    while arquivo.versao < VERSAO_ATUAL {
        match arquivo.versao {
            0 => migrar_v0_para_v1(arquivo),
            _ => break,
        }
    }
    arquivo.versao = VERSAO_ATUAL;
}

fn migrar_v0_para_v1(_arquivo: &mut ProjetoArquivo) {
    // Placeholder: versão inicial não precisa de migração.
}

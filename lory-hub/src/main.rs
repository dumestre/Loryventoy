slint::include_modules!();

mod hub_data;

use hub_data::{HubState, VERSOES, VERSAO_ATUAL};
use slint::{ModelRc, VecModel};
use std::rc::Rc;
use std::cell::RefCell;

fn make_projeto_model(state: &HubState) -> ModelRc<ProjetoInfo> {
    let items: Vec<ProjetoInfo> = state.projetos.iter().map(|p| {
        let nome = p.nome.trim_end_matches(".lory").to_string();
        ProjetoInfo {
            nome: nome.into(),
            caminho: p.caminho.clone().into(),
            modified: p.modified.clone().into(),
            size_label: hub_data::format_size(p.size_bytes).into(),
        }
    }).collect();
    ModelRc::new(VecModel::from(items))
}

fn make_versao_model(state: &HubState) -> ModelRc<VersaoInfo> {
    let items: Vec<VersaoInfo> = VERSOES.iter().map(|v| {
        let is_installed = state.installed_versions.iter().any(|x| x == v.numero);
        let is_atual = v.numero == VERSAO_ATUAL;
        let itens = v.itens;
        VersaoInfo {
            numero: v.numero.into(),
            titulo: v.titulo.into(),
            item0: itens.get(0).copied().unwrap_or("").into(),
            item1: itens.get(1).copied().unwrap_or("").into(),
            item2: itens.get(2).copied().unwrap_or("").into(),
            item3: itens.get(3).copied().unwrap_or("").into(),
            extra_count: (itens.len().saturating_sub(4)) as i32,
            is_installed,
            is_atual,
        }
    }).collect();
    ModelRc::new(VecModel::from(items))
}

fn sync_ui(window: &AppWindow, state: &HubState) {
    window.set_projetos(make_projeto_model(state));
    window.set_versoes(make_versao_model(state));
    window.set_versao_atual(VERSAO_ATUAL.into());
}

fn main() -> Result<(), slint::PlatformError> {
    let window = AppWindow::new()?;
    let mut collection = slint::fontique_010::shared_collection();
    let fonts_dir = std::path::Path::new("fonts");
    for name in &["Poppins-Regular.ttf", "Poppins-Medium.ttf", "Poppins-SemiBold.ttf", "Poppins-Bold.ttf", "Poppins-ExtraBold.ttf"] {
        if let Ok(data) = std::fs::read(fonts_dir.join(name)) {
            let blob = slint::fontique_010::fontique::Blob::new(std::sync::Arc::new(data));
            let _ = collection.register_fonts(blob, None);
        }
    }

    let state = Rc::new(RefCell::new(HubState::new()));

    // Initial sync
    sync_ui(&window, &state.borrow());

    // select-page
    {
        let window_weak = window.as_weak();
        window.on_select_page(move |page| {
            if let Some(w) = window_weak.upgrade() {
                w.set_current_page(page);
            }
        });
    }

    // refresh
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_refresh(move || {
            state.borrow_mut().refresh_projects();
            if let Some(w) = window_weak.upgrade() {
                sync_ui(&w, &state.borrow());
            }
        });
    }

    // set-query
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_set_query(move |q| {
            let q_str = q.to_string();
            let mut s = state.borrow_mut();
            s.query = q_str;
            let projetos = make_projeto_model(&s);
            drop(s);
            if let Some(w) = window_weak.upgrade() {
                w.set_projetos(projetos);
            }
        });
    }

    // open-project
    {
        let state = state.clone();
        window.on_open_project(move |caminho| {
            state.borrow_mut().open_project(&caminho.to_string());
        });
    }

    // delete-project
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_delete_project(move |caminho| {
            state.borrow_mut().delete_project(&caminho.to_string());
            if let Some(w) = window_weak.upgrade() {
                sync_ui(&w, &state.borrow());
            }
        });
    }

    // new-project (abre modal — por ora cria com nome padrão)
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_new_project(move || {
            state.borrow_mut().create_project("novo_projeto");
            if let Some(w) = window_weak.upgrade() {
                sync_ui(&w, &state.borrow());
            }
        });
    }

    // install-version
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_install_version(move |numero| {
            state.borrow_mut().install_version(&numero.to_string());
            if let Some(w) = window_weak.upgrade() {
                sync_ui(&w, &state.borrow());
            }
        });
    }

    // uninstall-version
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_uninstall_version(move |numero| {
            state.borrow_mut().uninstall_version(&numero.to_string());
            if let Some(w) = window_weak.upgrade() {
                sync_ui(&w, &state.borrow());
            }
        });
    }

    window.run()
}
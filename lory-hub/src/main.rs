slint::include_modules!();

mod hub_data;

use hub_data::{HubState, VERSAO_ATUAL, VERSOES};
use slint::{ModelRc, VecModel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn make_projeto_model(state: &HubState) -> ModelRc<ProjetoInfo> {
    let items: Vec<ProjetoInfo> = state
        .projetos
        .iter()
        .map(|p| {
            let nome = p.nome.trim_end_matches(".lory").to_string();
            ProjetoInfo {
                nome: nome.into(),
                caminho: p.caminho.clone().into(),
                modified: p.modified.clone().into(),
                size_label: hub_data::format_size(p.size_bytes).into(),
            }
        })
        .collect();
    ModelRc::new(VecModel::from(items))
}

fn make_versao_model(state: &HubState) -> ModelRc<VersaoInfo> {
    let items: Vec<VersaoInfo> = VERSOES
        .iter()
        .map(|v| {
            let is_installed = state.installed_versions.iter().any(|x| x == v.numero);
            let is_atual = v.numero == VERSAO_ATUAL;
            let itens = v.itens;
            let install_status = state
                .install_status
                .get(v.numero)
                .cloned()
                .unwrap_or_else(|| "idle".to_string());
            let install_progress = state.install_progress.get(v.numero).cloned().unwrap_or(0.0);
            let download_size = state
                .install_size
                .get(v.numero)
                .cloned()
                .unwrap_or_else(|| "".to_string());
            let download_percent = if install_status == "downloading" {
                format!("{:.0}%", install_progress * 100.0)
            } else {
                String::new()
            };
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
                install_status: install_status.into(),
                download_size: download_size.into(),
                download_percent: download_percent.into(),
                install_progress,
            }
        })
        .collect();
    ModelRc::new(VecModel::from(items))
}

fn sync_ui(window: &AppWindow, state: &HubState) {
    window.set_projetos(make_projeto_model(state));
    window.set_versoes(make_versao_model(state));
    window.set_versao_atual(VERSAO_ATUAL.into());
    window.set_pasta_projetos(state.pasta_projetos.clone().into());
    window.set_show_new_modal(state.show_new_modal);
    window.set_new_project_name(state.new_project_name.clone().into());
    window.set_pasta_instalacoes(state.pasta_instalacoes.clone().into());
    let has_updates = VERSOES
        .iter()
        .any(|v| !state.installed_versions.contains(&v.numero.to_string()));
    window.set_has_install_updates(has_updates);
}

fn main() -> Result<(), slint::PlatformError> {
    let window = AppWindow::new()?;
    let mut collection = slint::fontique_010::shared_collection();
    let fonts_dir = std::path::Path::new("fonts");
    for name in &[
        "Poppins-Regular.ttf",
        "Poppins-Medium.ttf",
        "Poppins-SemiBold.ttf",
        "Poppins-Bold.ttf",
        "Poppins-ExtraBold.ttf",
    ] {
        if let Ok(data) = std::fs::read(fonts_dir.join(name)) {
            let blob = slint::fontique_010::fontique::Blob::new(std::sync::Arc::new(data));
            let _ = collection.register_fonts(blob, None);
        }
    }

    let state = Arc::new(Mutex::new(HubState::new()));

    // Initial sync
    sync_ui(&window, &state.lock().unwrap());

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
            state.lock().unwrap().refresh_projects();
            if let Some(w) = window_weak.upgrade() {
                sync_ui(&w, &state.lock().unwrap());
            }
        });
    }

    // set-query
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_set_query(move |q| {
            let q_str = q.to_string();
            let mut s = state.lock().unwrap();
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
            state.lock().unwrap().open_project(&caminho.to_string());
        });
    }

    // delete-project
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_delete_project(move |caminho| {
            state.lock().unwrap().delete_project(&caminho.to_string());
            if let Some(w) = window_weak.upgrade() {
                sync_ui(&w, &state.lock().unwrap());
            }
        });
    }

    // new-project (abre modal — por ora cria com nome padrão)
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_new_project(move || {
            state.lock().unwrap().create_project("novo_projeto");
            if let Some(w) = window_weak.upgrade() {
                sync_ui(&w, &state.lock().unwrap());
            }
        });
    }

    // close-new-modal
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_close_new_modal(move || {
            state.lock().unwrap().show_new_modal = false;
            state.lock().unwrap().new_project_name.clear();
            if let Some(w) = window_weak.upgrade() {
                sync_ui(&w, &state.lock().unwrap());
            }
        });
    }

    // pick-folder
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_pick_folder(move || {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                let folder_str = folder.to_string_lossy().to_string();
                state.lock().unwrap().pasta_projetos = folder_str;
                state.lock().unwrap().refresh_projects();
                if let Some(w) = window_weak.upgrade() {
                    sync_ui(&w, &state.lock().unwrap());
                }
            }
        });
    }

    // open-project-file
    {
        let state = state.clone();
        window.on_open_project_file(move || {
            if let Some(file) = rfd::FileDialog::new()
                .add_filter("Lory Project", &["lory"])
                .pick_file()
            {
                let caminho = file.to_string_lossy().to_string();
                state.lock().unwrap().open_project(&caminho);
            }
        });
    }

    // create-named-project
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_create_named_project(move |nome| {
            let nome_str = nome.to_string();
            state.lock().unwrap().create_project(&nome_str);
            state.lock().unwrap().show_new_modal = false;
            state.lock().unwrap().new_project_name.clear();
            if let Some(w) = window_weak.upgrade() {
                sync_ui(&w, &state.lock().unwrap());
            }
        });
    }

    // install-version
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_install_version(move |numero| {
            let numero_str = numero.to_string();
            let w = window_weak.clone();
            let s = state.clone();

            {
                let mut state_guard = s.lock().unwrap();
                state_guard
                    .install_status
                    .insert(numero_str.clone(), "downloading".to_string());
                state_guard.install_progress.insert(numero_str.clone(), 0.0);
                let base_size =
                    12.0 + (numero_str.chars().map(|c| c as u32).sum::<u32>() % 40) as f32;
                state_guard
                    .install_size
                    .insert(numero_str.clone(), format!("{:.0} MB", base_size));
            }

            if let Some(win) = w.upgrade() {
                sync_ui(&win, &s.lock().unwrap());
            }

            thread::spawn(move || {
                let steps = 20u32;
                for i in 1..=steps {
                    thread::sleep(Duration::from_millis(150));
                    let progress = i as f32 / steps as f32;
                    let w2 = w.clone();
                    let s2 = s.clone();
                    let n = numero_str.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        let mut state_guard = s2.lock().unwrap();
                        state_guard.install_progress.insert(n.clone(), progress);
                        drop(state_guard);
                        if let Some(win) = w2.upgrade() {
                            sync_ui(&win, &s2.lock().unwrap());
                        }
                    });
                }

                let w3 = w.clone();
                let s3 = s.clone();
                let n3 = numero_str.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    let mut state_guard = s3.lock().unwrap();
                    state_guard
                        .install_status
                        .insert(n3.clone(), "installing".to_string());
                    state_guard.install_progress.insert(n3.clone(), 1.0);
                    drop(state_guard);
                    if let Some(win) = w3.upgrade() {
                        sync_ui(&win, &s3.lock().unwrap());
                    }
                });

                thread::sleep(Duration::from_millis(1200));

                let w4 = w.clone();
                let s4 = s.clone();
                let n4 = numero_str.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    let mut state_guard = s4.lock().unwrap();
                    state_guard.install_version(&n4);
                    state_guard.install_status.remove(&n4);
                    state_guard.install_progress.remove(&n4);
                    state_guard.install_size.remove(&n4);
                    drop(state_guard);
                    if let Some(win) = w4.upgrade() {
                        sync_ui(&win, &s4.lock().unwrap());
                    }
                });
            });
        });
    }

    // uninstall-version
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_uninstall_version(move |numero| {
            state.lock().unwrap().uninstall_version(&numero.to_string());
            if let Some(w) = window_weak.upgrade() {
                sync_ui(&w, &state.lock().unwrap());
            }
        });
    }

    // pick-install-folder
    {
        let window_weak = window.as_weak();
        let state = state.clone();
        window.on_pick_install_folder(move || {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                let folder_str = folder.to_string_lossy().to_string();
                state.lock().unwrap().pasta_instalacoes = folder_str;
                if let Some(w) = window_weak.upgrade() {
                    sync_ui(&w, &state.lock().unwrap());
                }
            }
        });
    }

    window.run()
}

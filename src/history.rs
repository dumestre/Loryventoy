use serde::Serialize;

pub struct History<T> {
    stack: Vec<T>,
    redo_stack: Vec<T>,
    limit: usize,
}

impl<T: PartialEq + Clone> History<T> {
    pub fn new(limit: usize) -> Self {
        Self {
            stack: Vec::new(),
            redo_stack: Vec::new(),
            limit,
        }
    }

    /// Empurra um novo estado no histórico.
    /// Ignora se for igual ao topo atual (evita dupes de repintura).
    /// Limpa a pilha redo (nova alteração invalira redo).
    pub fn push(&mut self, state: T) {
        if self.stack.last() == Some(&state) {
            return;
        }
        self.stack.push(state);
        if self.stack.len() > self.limit {
            self.stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Desfaz a última alteração. Retorna o estado desfeito se existir.
    pub fn undo(&mut self) -> Option<T> {
        let current = self.stack.pop()?;
        self.redo_stack.push(current.clone());
        Some(current)
    }

    /// Refaz a próxima alteração. Retorna o estado refeito se existir.
    pub fn redo(&mut self) -> Option<T> {
        let next = self.redo_stack.pop()?;
        self.stack.push(next.clone());
        Some(next)
    }

    pub fn pode_undo(&self) -> bool {
        !self.stack.is_empty()
    }

    pub fn pode_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn lim(&self) -> usize {
        self.limit
    }
}

impl<T: PartialEq + Clone + Serialize> History<T> {
    pub fn stack_json(&self) -> String {
        serde_json::to_string(&self.stack).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn novo_historico_vazio() {
        let h: History<String> = History::new(10);
        assert!(!h.pode_undo());
        assert!(!h.pode_redo());
        assert_eq!(h.len(), 0);
    }

    #[test]
    fn push_e_undo() {
        let mut h = History::new(10);
        h.push("state1".to_string());
        h.push("state2".to_string());
        assert!(h.pode_undo());
        assert_eq!(h.undo(), Some("state2".to_string()));
        assert_eq!(h.undo(), Some("state1".to_string()));
        assert_eq!(h.undo(), None);
    }

    #[test]
    fn redo_apos_undo() {
        let mut h = History::new(10);
        h.push("a".to_string());
        h.push("b".to_string());
        h.undo();
        assert!(h.pode_redo());
        assert_eq!(h.redo(), Some("b".to_string()));
        assert!(!h.pode_redo());
    }

    #[test]
    fn novo_push_limpa_redo() {
        let mut h = History::new(10);
        h.push("a".to_string());
        h.push("b".to_string());
        h.undo();
        assert!(h.pode_redo());
        h.push("c".to_string());
        assert!(!h.pode_redo());
    }

    #[test]
    fn nao_duplica_estado_identico() {
        let mut h = History::new(10);
        h.push("a".to_string());
        h.push("a".to_string());
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn respeita_limite() {
        let mut h = History::new(3);
        h.push("a".to_string());
        h.push("b".to_string());
        h.push("c".to_string());
        h.push("d".to_string());
        assert_eq!(h.len(), 3);
        assert_eq!(h.undo(), Some("d".to_string()));
        assert_eq!(h.undo(), Some("c".to_string()));
        assert_eq!(h.undo(), Some("b".to_string()));
        assert_eq!(h.undo(), None);
    }

    #[test]
    fn undo_nao_perde_historico() {
        let mut h = History::new(10);
        h.push("a".to_string());
        h.push("b".to_string());
        h.undo();
        assert!(h.pode_undo());
        assert_eq!(h.len(), 1);
    }
}
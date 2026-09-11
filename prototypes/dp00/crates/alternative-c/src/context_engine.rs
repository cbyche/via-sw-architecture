use bench_core::UserTurn;

pub struct ContextPackage {
    pub turn: UserTurn,
}

pub fn package(mut turn: UserTurn) -> ContextPackage {
    if !turn.context_evidence.is_empty() {
        let normalized = turn
            .context_evidence
            .iter()
            .map(|item| {
                format!(
                    "source={};kind={};value={};revision={:?}",
                    item.source, item.kind, item.value, item.observed_at_product_revision
                )
            })
            .collect::<Vec<_>>()
            .join("|");
        turn.content = format!("{};context=[{}]", turn.content, normalized);
    }
    ContextPackage { turn }
}

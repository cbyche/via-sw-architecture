use bench_core::UserTurn;

pub struct ContextPackage {
    pub turn: UserTurn,
}

pub fn package(turn: UserTurn) -> ContextPackage {
    ContextPackage { turn }
}

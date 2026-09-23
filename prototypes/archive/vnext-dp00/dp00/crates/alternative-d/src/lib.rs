#![forbid(unsafe_code)]

mod agent_client;
mod architecture;
mod context_engine;
mod execution_path_selector;
mod fast_executor;
mod intent_refiner;
mod task_manager;

pub use architecture::AdaptiveVia;

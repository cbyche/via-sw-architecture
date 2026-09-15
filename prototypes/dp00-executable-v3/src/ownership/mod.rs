pub mod device_adapters;
pub mod r1_agent_adapter;
pub mod r3_primary_extensions;
pub mod z1_interaction;
pub mod z2_context;
pub mod z3_semantics;
pub mod z4_agent_contract;
pub mod z5_task_lifecycle;
pub mod z6_delivery;
pub mod z7_runtime_resource;
pub mod z8_device;

pub fn common_regression_probe() -> usize {
    z1_interaction::VERSION
        + z2_context::VERSION
        + z3_semantics::VERSION
        + z4_agent_contract::VERSION
        + z5_task_lifecycle::VERSION
        + z6_delivery::VERSION
        + z7_runtime_resource::VERSION
        + z8_device::VERSION
        + r1_agent_adapter::VERSION
        + r3_primary_extensions::VERSION
        + device_adapters::PC
        + device_adapters::MOBILE
        + device_adapters::TV
        + device_adapters::ROBOT
}

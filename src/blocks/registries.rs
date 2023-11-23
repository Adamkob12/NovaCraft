use super::block_descriptor::{PassiveProperties, PerceptibleProperties, PhysicalProperties};
pub trait RegistryType {}

impl RegistryType for PassiveProperties {}
impl RegistryType for PhysicalProperties {}
impl RegistryType for PerceptibleProperties {}

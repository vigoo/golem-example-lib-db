use golem_rust::{agent_definition, agent_implementation};

#[agent_definition(ephemeral)]
trait Frontend {
    fn new() -> Self;
    fn index(&self) -> Vec<u8>;
}

struct FrontendImpl {}

#[agent_implementation]
impl Frontend for FrontendImpl {
    fn new() -> Self {
        Self {}
    }

    fn index(&self) -> Vec<u8> {
        let bytes = include_bytes!("../index.html");
        bytes.to_vec()
    }
}

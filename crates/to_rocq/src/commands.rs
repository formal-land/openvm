use clap::Args;
use eyre::Result;

#[derive(Args)]
pub struct PrintCircuitArgs {
    /// The type of circuit to print
    #[arg(long, short, value_enum)]
    circuit_type: CircuitType,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub(crate) enum CircuitType {
    BranchEq,
}

pub fn print_circuit(args: PrintCircuitArgs) -> Result<()> {
    match args.circuit_type {
        CircuitType::BranchEq => {
            crate::circuit_printer::print_branch_eq::<4>();
        }
    }

    Ok(())
}

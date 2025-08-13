use eyre::Result;
use openvm_circuit::arch::{AdapterAirContext, BasicAdapterInterface, ImmInstruction, VmCoreAir};
use openvm_stark_backend::air_builders::symbolic::symbolic_expression::SymbolicExpression;
use openvm_stark_backend::air_builders::symbolic::symbolic_variable::{Entry, SymbolicVariable};
use openvm_stark_backend::interaction::{BusIndex, Interaction, InteractionBuilder};
use openvm_stark_backend::p3_air::AirBuilder;
use openvm_stark_backend::p3_matrix::dense::RowMajorMatrix;
use openvm_stark_sdk::p3_goldilocks::Goldilocks;
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use tracing::info;

use crate::commands::OutputFormat;

pub struct CircuitPrinter;

#[derive(Deserialize, Default)]
struct JsonBuilder(Vec<SymbolicExpression<Goldilocks>>);

impl AirBuilder for JsonBuilder {
    type F = Goldilocks;

    type Expr = SymbolicExpression<Self::F>;

    type Var = SymbolicVariable<Self::F>;

    type M = RowMajorMatrix<Self::Var>;

    fn main(&self) -> <Self as AirBuilder>::M {
        unimplemented!()
    }

    fn is_first_row(&self) -> <Self as AirBuilder>::Expr {
        unimplemented!()
    }

    fn is_last_row(&self) -> <Self as AirBuilder>::Expr {
        unimplemented!()
    }

    fn is_transition_window(&self, _: usize) -> <Self as AirBuilder>::Expr {
        unimplemented!()
    }

    fn assert_zero<I>(&mut self, expr: I)
    where
        I: Into<Self::Expr>,
    {
        self.0.push(expr.into());
    }
}

impl InteractionBuilder for JsonBuilder {
    fn push_interaction<E: Into<Self::Expr>>(
        &mut self,
        _bus_index: BusIndex,
        _fields: impl IntoIterator<Item = E>,
        _count: impl Into<Self::Expr>,
        _count_weight: u32,
    ) {
        unimplemented!()
    }

    fn num_interactions(&self) -> usize {
        unimplemented!()
    }

    fn all_interactions(&self) -> &[Interaction<Self::Expr>] {
        unimplemented!()
    }
}

impl CircuitPrinter {
    pub fn new() -> Self {
        Self
    }

    pub fn print_poseidon2_circuit(
        &self,
        format: OutputFormat,
        output: Option<&str>,
    ) -> Result<()> {
        info!("Printing Poseidon2 circuit");

        let circuit_info = self.get_poseidon2_circuit_info()?;
        self.output_circuit_info(&circuit_info, format, output)
    }

    pub fn print_sha256_circuit(&self, format: OutputFormat, output: Option<&str>) -> Result<()> {
        info!("Printing SHA256 circuit");

        let circuit_info = self.get_sha256_circuit_info()?;
        self.output_circuit_info(&circuit_info, format, output)
    }

    pub fn print_keccak256_circuit(
        &self,
        format: OutputFormat,
        output: Option<&str>,
    ) -> Result<()> {
        info!("Printing Keccak256 circuit");

        let circuit_info = self.get_keccak256_circuit_info()?;
        self.output_circuit_info(&circuit_info, format, output)
    }

    pub fn print_primitives_circuit(
        &self,
        format: OutputFormat,
        output: Option<&str>,
    ) -> Result<()> {
        info!("Printing primitives circuit");

        let circuit_info = self.get_primitives_circuit_info()?;
        self.output_circuit_info(&circuit_info, format, output)
    }

    fn get_poseidon2_circuit_info(&self) -> Result<serde_json::Value> {
        // This would inspect the actual Poseidon2 circuit implementation
        // For now, return a placeholder structure
        Ok(json!({
            "circuit_type": "Poseidon2",
            "description": "Poseidon2 hash function circuit implementation",
            "field": "BabyBear",
            "rounds": 8,
            "width": 3,
            "constraints": "Multiple rounds of poseidon2 permutation",
            "implementation": "Based on Plonky3 poseidon2-air"
        }))
    }

    fn get_sha256_circuit_info(&self) -> Result<serde_json::Value> {
        Ok(json!({
            "circuit_type": "SHA256",
            "description": "SHA256 hash function circuit implementation",
            "block_size": 512,
            "rounds": 64,
            "constraints": "SHA256 compression function constraints",
            "implementation": "Custom SHA256 AIR implementation"
        }))
    }

    fn get_keccak256_circuit_info(&self) -> Result<serde_json::Value> {
        Ok(json!({
            "circuit_type": "Keccak256",
            "description": "Keccak256 hash function circuit implementation",
            "block_size": 1088,
            "rounds": 24,
            "constraints": "Keccak-f[1600] permutation constraints",
            "implementation": "Based on Plonky3 keccak-air"
        }))
    }

    fn get_primitives_circuit_info(&self) -> Result<serde_json::Value> {
        Ok(json!({
            "circuit_type": "Primitives",
            "description": "General purpose circuit primitives",
            "components": [
                "Field arithmetic",
                "Boolean logic",
                "Range checks",
                "Lookup tables"
            ],
            "implementation": "OpenVM circuit primitives library"
        }))
    }

    fn output_circuit_info(
        &self,
        circuit_info: &serde_json::Value,
        format: OutputFormat,
        output: Option<&str>,
    ) -> Result<()> {
        let output_content = match format {
            OutputFormat::Text => self.format_as_text(circuit_info)?,
            OutputFormat::Json => serde_json::to_string_pretty(circuit_info)?,
            OutputFormat::Rocq => self.format_as_rocq(circuit_info)?,
        };

        if let Some(output_path) = output {
            let mut file = File::create(output_path)?;
            file.write_all(output_content.as_bytes())?;
            info!("Circuit information written to: {}", output_path);
        } else {
            println!("{}", output_content);
        }

        Ok(())
    }

    fn format_as_text(&self, circuit_info: &serde_json::Value) -> Result<String> {
        let mut output = String::new();

        if let Some(circuit_type) = circuit_info.get("circuit_type") {
            output.push_str(&format!("Circuit Type: {}\n", circuit_type));
        }

        if let Some(description) = circuit_info.get("description") {
            output.push_str(&format!("Description: {}\n", description));
        }

        // Add other fields as they exist
        if let Some(obj) = circuit_info.as_object() {
            for (key, value) in obj {
                if key != "circuit_type" && key != "description" {
                    output.push_str(&format!("{}: {}\n", key, value));
                }
            }
        }

        Ok(output)
    }

    fn format_as_rocq(&self, circuit_info: &serde_json::Value) -> Result<String> {
        // This would format the circuit information in a Rocq-compatible format
        // For now, return a basic structure
        let mut output = String::new();
        output.push_str("// Rocq circuit format\n");
        output.push_str("// Generated by OpenVM to-rocq tool\n\n");

        if let Some(circuit_type) = circuit_info.get("circuit_type") {
            output.push_str(&format!("circuit {} {{\n", circuit_type));
        }

        // Add circuit-specific information in Rocq format
        output.push_str("    // Circuit implementation details would go here\n");
        output.push_str("    // This is a placeholder for the actual Rocq format\n");
        output.push_str("}\n");

        Ok(output)
    }

    pub(crate) fn print_to_json<const NUM_LIMBS: usize>() {
        let air: openvm_rv32im_circuit::BranchEqualCoreAir<NUM_LIMBS> =
            openvm_rv32im_circuit::BranchEqualCoreChip::new(12, 23).air;
        let mut builder = JsonBuilder::default();

        let adapter_air_context: AdapterAirContext<
            SymbolicExpression<Goldilocks>,
            BasicAdapterInterface<
                SymbolicExpression<Goldilocks>,
                ImmInstruction<SymbolicExpression<Goldilocks>>,
                2,
                0,
                NUM_LIMBS,
                NUM_LIMBS,
            >,
        > = air.eval(
            &mut builder,
            &[
                SymbolicVariable::new(Entry::Public, 0),
                SymbolicVariable::new(Entry::Public, 1),
                SymbolicVariable::new(Entry::Public, 2),
                SymbolicVariable::new(Entry::Public, 3),
                SymbolicVariable::new(Entry::Public, 4),
                SymbolicVariable::new(Entry::Public, 5),
                SymbolicVariable::new(Entry::Public, 6),
                SymbolicVariable::new(Entry::Public, 7),
                SymbolicVariable::new(Entry::Public, 8),
                SymbolicVariable::new(Entry::Public, 9),
                SymbolicVariable::new(Entry::Public, 10),
                SymbolicVariable::new(Entry::Public, 11),
                SymbolicVariable::new(Entry::Public, 12),
                SymbolicVariable::new(Entry::Public, 13),
                SymbolicVariable::new(Entry::Public, 14),
                SymbolicVariable::new(Entry::Public, 15),
            ],
            SymbolicVariable::new(Entry::Public, 16),
        );

        let json = serde_json::to_string_pretty(&builder.0).unwrap();
        println!("{}", json);

        println!("{:#?}", adapter_air_context.to_pc);
        println!("{:#?}", adapter_air_context.reads);
        println!("{:#?}", adapter_air_context.writes);
        println!("{:#?}", adapter_air_context.instruction.is_valid);
        println!("{:#?}", adapter_air_context.instruction.opcode);
        println!("{:#?}", adapter_air_context.instruction.immediate);
    }
}

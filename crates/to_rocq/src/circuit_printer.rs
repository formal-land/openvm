use openvm_circuit::arch::{AdapterAirContext, BasicAdapterInterface, ImmInstruction, VmCoreAir};
use openvm_stark_backend::air_builders::symbolic::symbolic_expression::SymbolicExpression;
use openvm_stark_backend::air_builders::symbolic::symbolic_variable::{Entry, SymbolicVariable};
use openvm_stark_backend::interaction::{BusIndex, Interaction, InteractionBuilder};
use openvm_stark_backend::p3_air::AirBuilder;
use openvm_stark_backend::p3_matrix::dense::RowMajorMatrix;
use openvm_stark_sdk::p3_goldilocks::Goldilocks;
use p3_field::Field;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Default)]
struct RocqAirBuilder(Vec<SymbolicExpression<Goldilocks>>);

impl AirBuilder for RocqAirBuilder {
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

impl InteractionBuilder for RocqAirBuilder {
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

pub(crate) fn print_branch_eq<const NUM_LIMBS: usize>() {
    let air: openvm_rv32im_circuit::BranchEqualCoreAir<NUM_LIMBS> =
        openvm_rv32im_circuit::BranchEqualCoreChip::new(12, 23).air;
    let mut builder = RocqAirBuilder::default();

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

    builder.to_rocq(0);
    println!("Result 🛍️");
    adapter_air_context.to_rocq(2);
}

trait ToRocq {
    fn to_rocq(&self, indent: usize);
}

impl<
        T,
        PI,
        const BASIC_NUM_READS: usize,
        const BASIC_NUM_WRITES: usize,
        const READ_SIZE: usize,
        const WRITE_SIZE: usize,
    > ToRocq
    for AdapterAirContext<
        T,
        BasicAdapterInterface<T, PI, BASIC_NUM_READS, BASIC_NUM_WRITES, READ_SIZE, WRITE_SIZE>,
    >
where
    T: ToRocq,
    PI: ToRocq,
{
    fn to_rocq(&self, indent: usize) {
        println!("{}{}", " ".repeat(indent), "AdapterAirContext:");
        let indent = indent + 2;
        println!("{}{}", " ".repeat(indent), "to_pc:");
        self.to_pc.to_rocq(indent + 2);
        println!("{}{}", " ".repeat(indent), "reads:");
        self.reads.to_rocq(indent + 2);
        println!("{}{}", " ".repeat(indent), "writes:");
        self.writes.to_rocq(indent + 2);
        println!("{}{}", " ".repeat(indent), "instruction:");
        self.instruction.to_rocq(indent + 2);
    }
}

impl<T: ToRocq> ToRocq for ImmInstruction<T> {
    fn to_rocq(&self, indent: usize) {
        println!("{}{}", " ".repeat(indent), "ImmInstruction:");
        println!("{}{}", " ".repeat(indent + 2), "is_valid:");
        self.is_valid.to_rocq(indent + 4);
        println!("{}{}", " ".repeat(indent + 2), "opcode:");
        self.opcode.to_rocq(indent + 4);
        println!("{}{}", " ".repeat(indent + 2), "immediate:");
        self.immediate.to_rocq(indent + 4);
    }
}

enum FlatSymbolicExpression<'a, F> {
    Variable(&'a SymbolicVariable<F>),
    IsFirstRow,
    IsLastRow,
    IsTransition,
    Constant(&'a F),
    Add(Vec<Arc<FlatSymbolicExpression<'a, F>>>),
    Sub {
        x: Arc<FlatSymbolicExpression<'a, F>>,
        y: Arc<FlatSymbolicExpression<'a, F>>,
    },
    Neg {
        x: Arc<FlatSymbolicExpression<'a, F>>,
    },
    Mul(Vec<Arc<FlatSymbolicExpression<'a, F>>>),
}

impl<'a, F> FlatSymbolicExpression<'a, F> {
    fn from_symbolic_expression(expr: &'a SymbolicExpression<F>) -> Self {
        match expr {
            SymbolicExpression::Variable(v) => Self::Variable(v),
            SymbolicExpression::IsFirstRow => Self::IsFirstRow,
            SymbolicExpression::IsLastRow => Self::IsLastRow,
            SymbolicExpression::IsTransition => Self::IsTransition,
            SymbolicExpression::Constant(c) => Self::Constant(c),
            SymbolicExpression::Add { x, y, .. } => Self::Add(vec![
                Arc::new(Self::from_symbolic_expression(x)),
                Arc::new(Self::from_symbolic_expression(y)),
            ]),
            SymbolicExpression::Sub { x, y, .. } => Self::Sub {
                x: Arc::new(Self::from_symbolic_expression(x)),
                y: Arc::new(Self::from_symbolic_expression(y)),
            },
            SymbolicExpression::Neg { x, .. } => Self::Neg {
                x: Arc::new(Self::from_symbolic_expression(x)),
            },
            SymbolicExpression::Mul { x, y, .. } => Self::Mul(vec![
                Arc::new(Self::from_symbolic_expression(x)),
                Arc::new(Self::from_symbolic_expression(y)),
            ]),
        }
    }
}

impl<'a, F> ToRocq for FlatSymbolicExpression<'a, F>
where
    F: Debug,
{
    fn to_rocq(&self, indent: usize) {
        match self {
            FlatSymbolicExpression::Variable(v) => {
                println!("{}{} {:?}", " ".repeat(indent), "Variable:", v.index);
            }
            FlatSymbolicExpression::IsFirstRow => {
                println!("{}{}", " ".repeat(indent), "IsFirstRow");
            }
            FlatSymbolicExpression::IsLastRow => {
                println!("{}{}", " ".repeat(indent), "IsLastRow");
            }
            FlatSymbolicExpression::IsTransition => {
                println!("{}{}", " ".repeat(indent), "IsTransition");
            }
            FlatSymbolicExpression::Constant(c) => {
                println!("{}{} {:?}", " ".repeat(indent), "Constant:", c);
            }
            FlatSymbolicExpression::Add(xs) => {
                println!("{}{}", " ".repeat(indent), "Add:");
                for x in xs {
                    x.to_rocq(indent + 2);
                }
            }
            FlatSymbolicExpression::Sub { x, y } => {
                println!("{}{}", " ".repeat(indent), "Sub:");
                x.to_rocq(indent + 2);
                y.to_rocq(indent + 2);
            }
            FlatSymbolicExpression::Neg { x } => {
                println!("{}{}", " ".repeat(indent), "Neg:");
                x.to_rocq(indent + 2);
            }
            FlatSymbolicExpression::Mul(xs) => {
                println!("{}{}", " ".repeat(indent), "Mul:");
                for x in xs {
                    x.to_rocq(indent + 2);
                }
            }
        }
    }
}

impl<F> ToRocq for SymbolicExpression<F>
where
    F: Field,
    F: Debug,
{
    fn to_rocq(&self, indent: usize) {
        FlatSymbolicExpression::from_symbolic_expression(self).to_rocq(indent);
    }
}

impl<T: ToRocq> ToRocq for Option<T> {
    fn to_rocq(&self, indent: usize) {
        match self {
            Some(t) => {
                println!("{}{}", " ".repeat(indent), "Some:");
                t.to_rocq(indent + 2);
            }
            None => {
                println!("{}{}", " ".repeat(indent), "None");
            }
        }
    }
}

impl<T: ToRocq, const N: usize> ToRocq for [T; N] {
    fn to_rocq(&self, indent: usize) {
        println!("{}{}", " ".repeat(indent), "Array:");
        for item in self {
            item.to_rocq(indent + 2);
        }
    }
}

impl ToRocq for RocqAirBuilder {
    fn to_rocq(&self, indent: usize) {
        println!("{}{}", " ".repeat(indent), "Trace 🐾");
        for item in &self.0 {
            println!("{}{}", " ".repeat(indent + 2), "AssertZero:");
            item.to_rocq(indent + 4);
        }
    }
}

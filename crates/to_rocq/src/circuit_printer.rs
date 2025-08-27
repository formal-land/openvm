use openvm_circuit::arch::{AdapterAirContext, BasicAdapterInterface, ImmInstruction, VmCoreAir};
use openvm_circuit_primitives::utils::LoggingAirBuilder;
use openvm_circuit_primitives::SubAir;
use openvm_stark_backend::air_builders::symbolic::symbolic_expression::SymbolicExpression;
use openvm_stark_backend::air_builders::symbolic::symbolic_variable::{Entry, SymbolicVariable};
use openvm_stark_backend::interaction::{BusIndex, Interaction, InteractionBuilder};
use openvm_stark_backend::p3_air::AirBuilder;
use openvm_stark_backend::p3_matrix::dense::RowMajorMatrix;
use openvm_stark_sdk::p3_goldilocks::Goldilocks;
use p3_field::Field;
use std::fmt::Debug;
use std::sync::Arc;

enum Constraint<E> {
    AssertZero(E),
    Message(String),
    Interaction(Interaction<SymbolicExpression<Goldilocks>>),
}

struct RocqAirBuilder {
    main: RowMajorMatrix<SymbolicVariable<Goldilocks>>,
    constraints: Vec<Constraint<SymbolicExpression<Goldilocks>>>,
}

impl RocqAirBuilder {
    fn new(width: usize, height: usize) -> Self {
        let mut main = RowMajorMatrix::new(
            vec![SymbolicVariable::new(Entry::Public, 0); width * height],
            width,
        );

        for h in 0..height {
            for w in 0..width {
                main.values[h * width + w] = SymbolicVariable::new(Entry::Public, h * width + w);
            }
        }

        Self {
            main,
            constraints: Vec::new(),
        }
    }
}

impl AirBuilder for RocqAirBuilder {
    type F = Goldilocks;

    type Expr = SymbolicExpression<Self::F>;

    type Var = SymbolicVariable<Self::F>;

    type M = RowMajorMatrix<Self::Var>;

    fn main(&self) -> <Self as AirBuilder>::M {
        self.main.clone()
    }

    fn is_first_row(&self) -> <Self as AirBuilder>::Expr {
        SymbolicExpression::IsFirstRow
    }

    fn is_last_row(&self) -> <Self as AirBuilder>::Expr {
        SymbolicExpression::IsLastRow
    }

    fn is_transition_window(&self, _: usize) -> <Self as AirBuilder>::Expr {
        SymbolicExpression::IsTransition
    }

    fn assert_zero<I>(&mut self, expr: I)
    where
        I: Into<Self::Expr>,
    {
        self.constraints.push(Constraint::AssertZero(expr.into()));
    }
}

impl InteractionBuilder for RocqAirBuilder {
    fn push_interaction<E: Into<Self::Expr>>(
        &mut self,
        bus_index: BusIndex,
        fields: impl IntoIterator<Item = E>,
        count: impl Into<Self::Expr>,
        count_weight: u32,
    ) {
        self.constraints.push(Constraint::Interaction(Interaction {
            bus_index,
            message: fields.into_iter().map(|f| f.into()).collect(),
            count: count.into(),
            count_weight,
        }));
    }

    fn num_interactions(&self) -> usize {
        unimplemented!()
    }

    fn all_interactions(&self) -> &[Interaction<Self::Expr>] {
        unimplemented!()
    }
}

impl LoggingAirBuilder for RocqAirBuilder {
    fn log_in_constraints(&mut self, message: &str) {
        self.constraints
            .push(Constraint::Message(message.to_string()));
    }
}

pub(crate) fn print_branch_eq<const NUM_LIMBS: usize>() {
    let air: openvm_rv32im_circuit::BranchEqualCoreAir<NUM_LIMBS> =
        openvm_rv32im_circuit::BranchEqualCoreChip::new(12, 23).air;
    let mut builder = RocqAirBuilder::new(0, 0);

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

pub(crate) fn print_sha256() {
    let air: openvm_sha256_air::Sha256Air = openvm_sha256_air::Sha256Air::new(
        openvm_circuit_primitives::bitwise_op_lookup::BitwiseOperationLookupBus::new(8000),
        8001,
    );
    let mut builder = RocqAirBuilder::new(
        openvm_sha256_air::SHA256_DIGEST_WIDTH.max(openvm_sha256_air::SHA256_ROUND_WIDTH),
        2,
    );

    air.eval(&mut builder, 0);

    builder.to_rocq(0);
    println!("Result 🛍️");
    println!("  tt");
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
            SymbolicExpression::Add { x, y, .. } => {
                let x = Self::from_symbolic_expression(x);
                let y = Self::from_symbolic_expression(y);
                match (&x, &y) {
                    (Self::Add(xs), Self::Add(ys)) => {
                        Self::Add([xs.to_vec(), ys.to_vec()].concat())
                    }
                    (Self::Add(xs), _) => Self::Add([xs.to_vec(), vec![Arc::new(y)]].concat()),
                    (_, Self::Add(ys)) => Self::Add([vec![Arc::new(x)], ys.to_vec()].concat()),
                    (_, _) => Self::Add(vec![Arc::new(x), Arc::new(y)]),
                }
            }
            SymbolicExpression::Sub { x, y, .. } => Self::Sub {
                x: Arc::new(Self::from_symbolic_expression(x)),
                y: Arc::new(Self::from_symbolic_expression(y)),
            },
            SymbolicExpression::Neg { x, .. } => Self::Neg {
                x: Arc::new(Self::from_symbolic_expression(x)),
            },
            SymbolicExpression::Mul { x, y, .. } => {
                let x = Self::from_symbolic_expression(x);
                let y = Self::from_symbolic_expression(y);
                match (&x, &y) {
                    (Self::Mul(xs), Self::Mul(ys)) => {
                        Self::Mul([xs.to_vec(), ys.to_vec()].concat())
                    }
                    (Self::Mul(xs), _) => Self::Mul([xs.to_vec(), vec![Arc::new(y)]].concat()),
                    (_, Self::Mul(ys)) => Self::Mul([vec![Arc::new(x)], ys.to_vec()].concat()),
                    (_, _) => Self::Mul(vec![Arc::new(x), Arc::new(y)]),
                }
            }
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

impl ToRocq for Interaction<SymbolicExpression<Goldilocks>> {
    fn to_rocq(&self, indent: usize) {
        println!("{}{}", " ".repeat(indent), "Interaction:");
        println!("{}{}", " ".repeat(indent + 2), "message:");
        for item in &self.message {
            item.to_rocq(indent + 4);
        }
        println!("{}{}", " ".repeat(indent + 2), "count:");
        self.count.to_rocq(indent + 4);
        println!("{}{}", " ".repeat(indent + 2), "bus_index:");
        println!("{}{}", " ".repeat(indent + 4), self.bus_index);
        println!("{}{}", " ".repeat(indent + 2), "count_weight:");
        println!("{}{}", " ".repeat(indent + 4), self.count_weight);
    }
}

impl ToRocq for RocqAirBuilder {
    fn to_rocq(&self, indent: usize) {
        println!("{}{}", " ".repeat(indent), "Trace 🐾");
        for item in &self.constraints {
            match item {
                Constraint::AssertZero(expr) => {
                    println!("{}{}", " ".repeat(indent + 2), "AssertZero:");
                    expr.to_rocq(indent + 4);
                }
                Constraint::Message(message) => {
                    println!("{}{}", " ".repeat(indent + 2), "Message 🦜");
                    println!("{}{}", " ".repeat(indent + 4), message);
                }
                Constraint::Interaction(interaction) => {
                    interaction.to_rocq(indent + 2);
                }
            }
        }
    }
}

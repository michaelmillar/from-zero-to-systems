// ============================================================
//  YOUR CHALLENGE - implement a stack-based bytecode VM.
//
//  The VM executes a sequence of `Instruction` values.
//  It has a value stack (Vec<i64>) and a program counter.
//
//  Instructions:
//    Push(n)  - push integer n onto the stack
//    Pop      - discard the top of the stack
//    Add      - pop two values, push their sum
//    Sub      - pop two values, push (second - top)
//    Mul      - pop two values, push their product
//    Div      - pop two values, push (second / top); error if top is 0
//    Dup      - duplicate the top of the stack
//    Swap     - swap the top two values
//    Halt     - stop execution, return the top of the stack
//
//  Execution stops at Halt or when instructions are exhausted.
//  Return Err(VmError) for stack underflow or division by zero.
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Push(i64),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Dup,
    Swap,
    Halt,
}

#[derive(Debug, PartialEq)]
pub enum VmError {
    StackUnderflow,
    DivisionByZero,
    EmptyStack,
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmError::StackUnderflow  => write!(f, "stack underflow"),
            VmError::DivisionByZero  => write!(f, "division by zero"),
            VmError::EmptyStack      => write!(f, "halted with empty stack"),
        }
    }
}

pub struct Vm {
    pub stack: Vec<i64>,
}

impl Vm {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Execute a program (slice of instructions).
    /// Returns the value on top of the stack when Halt is reached,
    /// or when the program ends.
    pub fn run(&mut self, program: &[Instruction]) -> Result<i64, VmError> {
        todo!()
    }
}

impl Default for Vm {
    fn default() -> Self { Self::new() }
}

// ============================================================
//  TESTS - they ARE the spec. Make them all pass.
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use Instruction::*;

    fn run(program: Vec<Instruction>) -> Result<i64, VmError> {
        Vm::new().run(&program)
    }

    mod arithmetic {
        use super::*;

        #[test]
        fn push_then_halt_returns_pushed_value() {
            assert_eq!(run(vec![Push(42), Halt]), Ok(42));
        }

        #[test]
        fn add_two_numbers() {
            assert_eq!(run(vec![Push(3), Push(4), Add, Halt]), Ok(7));
        }

        #[test]
        fn sub_computes_second_minus_top() {
            // stack after pushes: [10, 3] (3 on top)
            // sub: 10 - 3 = 7
            assert_eq!(run(vec![Push(10), Push(3), Sub, Halt]), Ok(7));
        }

        #[test]
        fn mul_two_numbers() {
            assert_eq!(run(vec![Push(6), Push(7), Mul, Halt]), Ok(42));
        }

        #[test]
        fn div_computes_integer_division() {
            assert_eq!(run(vec![Push(10), Push(3), Div, Halt]), Ok(3));
        }

        #[test]
        fn chained_operations_evaluate_correctly() {
            // (2 + 3) * 4 = 20
            assert_eq!(run(vec![Push(2), Push(3), Add, Push(4), Mul, Halt]), Ok(20));
        }
    }

    mod stack_operations {
        use super::*;

        #[test]
        fn dup_doubles_top_of_stack() {
            // Push 5, dup -> [5, 5], add -> [10]
            assert_eq!(run(vec![Push(5), Dup, Add, Halt]), Ok(10));
        }

        #[test]
        fn swap_reverses_top_two_elements() {
            // Push 1, Push 2, Swap -> [2, 1], Sub -> 2-1=1
            assert_eq!(run(vec![Push(1), Push(2), Swap, Sub, Halt]), Ok(1));
        }

        #[test]
        fn pop_discards_top_leaving_next_value() {
            assert_eq!(run(vec![Push(99), Push(1), Pop, Halt]), Ok(99));
        }
    }

    mod error_handling {
        use super::*;

        #[test]
        fn add_with_empty_stack_returns_underflow() {
            assert_eq!(run(vec![Add]), Err(VmError::StackUnderflow));
        }

        #[test]
        fn div_by_zero_returns_error() {
            assert_eq!(run(vec![Push(10), Push(0), Div, Halt]), Err(VmError::DivisionByZero));
        }

        #[test]
        fn halt_on_empty_stack_returns_empty_stack_error() {
            assert_eq!(run(vec![Halt]), Err(VmError::EmptyStack));
        }

        #[test]
        fn program_ending_without_halt_returns_top_of_stack() {
            // No explicit Halt - should still return the top value
            assert_eq!(run(vec![Push(7), Push(8), Add]), Ok(15));
        }
    }
}

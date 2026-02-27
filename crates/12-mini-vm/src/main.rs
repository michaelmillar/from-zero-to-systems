use mini_vm::{Instruction::*, Vm};

fn run_and_print(label: &str, program: Vec<mini_vm::Instruction>) {
    let mut vm = Vm::new();
    match vm.run(&program) {
        Ok(result) => println!("  {label:<40} → {result}"),
        Err(e)     => println!("  {label:<40} → ERROR: {e}"),
    }
}

fn main() {
    println!("=== Mini VM: Stack-Based Bytecode Interpreter ===\n");

    println!("  Arithmetic programs:\n");
    run_and_print("PUSH 6, PUSH 7, MUL, HALT  (6×7)", vec![Push(6), Push(7), Mul, Halt]);
    run_and_print("PUSH 2, PUSH 3, ADD, PUSH 4, MUL (=(2+3)×4)", vec![Push(2), Push(3), Add, Push(4), Mul, Halt]);
    run_and_print("PUSH 100, PUSH 7, DIV  (100÷7)", vec![Push(100), Push(7), Div, Halt]);
    run_and_print("PUSH 5, DUP, MUL  (5²)", vec![Push(5), Dup, Mul, Halt]);

    println!("\n  Error cases:\n");
    run_and_print("PUSH 10, PUSH 0, DIV  (÷0)", vec![Push(10), Push(0), Div, Halt]);
    run_and_print("ADD on empty stack", vec![Add]);
    run_and_print("HALT on empty stack", vec![Halt]);

    println!("\n  Encoding note:");
    println!("  Instructions could be serialised to bytes via crate 09 (bit-manipulator)");
    println!("  and stored in an arena from crate 10 (memory-arena) — that's the tier 3 chain.");
}

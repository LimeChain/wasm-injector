use wasm_instrument::parity_wasm::elements::{
    BlockType, FuncBody, Instruction, Instructions, Local, Module, ValueType,
};

use super::injector::FunctionMapper;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Injection {
    Nothing,
    InfiniteLoop,
    JibberishReturnValue,
    StackOverflow,
    Noops,
    HeapOverflow,
}

impl Injection {
    pub fn inject(self, module: &mut Module) -> Result<(), String> {
        get_injection(self)(module)
    }
}

type InjectionFn = dyn FnMut(&mut Module) -> Result<(), String>;

pub fn get_injection(injection: Injection) -> Box<InjectionFn> {
    Box::new(match injection {
        Injection::Nothing => |_| Ok(()),
        Injection::InfiniteLoop => inject_infinite_loop,
        Injection::JibberishReturnValue => inject_jibberish_return_value,
        Injection::StackOverflow => inject_stack_overflow,
        Injection::Noops => inject_noops,
        Injection::HeapOverflow => inject_heap_overflow,
    })
}

pub fn inject_infinite_loop(module: &mut Module) -> Result<(), String> {
    module.map_function("validate_block", |func_body: &mut FuncBody| {
        let code = func_body.code_mut();

        let mut code_with_loop = vec![
            // Loop never ends
            Instruction::Loop(BlockType::NoResult),
            Instruction::Nop,
            Instruction::Br(0),
            Instruction::End,
        ];
        code_with_loop.append(code.elements_mut());

        *code.elements_mut() = code_with_loop;
    })
}

fn inject_jibberish_return_value(module: &mut Module) -> Result<(), String> {
    module.map_function("validate_block", |func_body: &mut FuncBody| {
        *func_body.code_mut() = Instructions::new(vec![
            // Last value on the stack gets returned
            Instruction::I64Const(123456789),
            Instruction::End,
        ]);
    })
}

fn inject_stack_overflow(module: &mut Module) -> Result<(), String> {
    module.map_function("validate_block", |func_body: &mut FuncBody| {
        func_body.locals_mut().append(&mut vec![
            // Creating 100 `i64`s should cause the stack to overflow
            Local::new(100, ValueType::I64),
        ]);
    })
}

fn inject_noops(module: &mut Module) -> Result<(), String> {
    module.map_function("validate_block", |func_body: &mut FuncBody| {
        // Add half a billion NoOperations to (hopefully) slow down interpretation-time
        let code = func_body.code_mut();

        let mut nops = vec![Instruction::Nop; 500_000_000];
        nops.append(code.elements_mut());

        *code.elements_mut() = nops;
    })
}

fn inject_heap_overflow(module: &mut Module) -> Result<(), String> {
    module.map_function("validate_block", |func_body: &mut FuncBody| {
        let code = func_body.code_mut();

        let mut code_with_allocation = vec![
            // Try to allocate 255 pages
            Instruction::Block(BlockType::NoResult),
            Instruction::I32Const(i32::MAX),
            Instruction::GrowMemory(0),
            Instruction::Drop,
            Instruction::End,
        ];
        code_with_allocation.append(code.elements_mut());

        *code.elements_mut() = code_with_allocation;
    })
}

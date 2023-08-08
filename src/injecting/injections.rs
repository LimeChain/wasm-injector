use std::fmt::{Display, Formatter};
use wasm_instrument::parity_wasm::elements::{
    BlockType, FuncBody, Instruction, Instructions, Module,
};

use super::injector::FunctionMapper;

#[derive(clap::ValueEnum, PartialEq, Eq, Clone, Debug)]
pub enum Injection {
    InfiniteLoop,
    BadReturnValue,
    StackOverflow,
    Noops,
    HeapOverflow,
}

impl Injection {
    pub fn inject(self, module: &mut Module) -> Result<(), String> {
        get_injection(self)(module)
    }
}

impl Display for Injection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Injection::InfiniteLoop => write!(f, "infinite-loop"),
            Injection::BadReturnValue => write!(f, "bad-return-value"),
            Injection::StackOverflow => write!(f, "stack-overflow"),
            Injection::Noops => write!(f, "noops"),
            Injection::HeapOverflow => write!(f, "heap-overflow"),
        }
    }
}
type InjectionFn = dyn FnMut(&mut Module) -> Result<(), String>;

pub fn get_injection(injection: Injection) -> Box<InjectionFn> {
    Box::new(match injection {
        Injection::InfiniteLoop => inject_infinite_loop,
        Injection::BadReturnValue => inject_bad_return_value,
        Injection::StackOverflow => inject_stack_overflow,
        Injection::Noops => inject_noops,
        Injection::HeapOverflow => inject_heap_overflow,
    })
}

pub fn inject_infinite_loop(module: &mut Module) -> Result<(), String> {
    module.map_function("validate_block", |func_body: &mut FuncBody, _| {
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

fn inject_bad_return_value(module: &mut Module) -> Result<(), String> {
    module.map_function("validate_block", |func_body: &mut FuncBody, _| {
        *func_body.code_mut() = Instructions::new(vec![
            // Last value on the stack gets returned
            Instruction::I64Const(123456789),
            Instruction::End,
        ]);
    })
}

fn inject_stack_overflow(module: &mut Module) -> Result<(), String> {
    module.map_function("validate_block", |func_body: &mut FuncBody, _| {
        let code = func_body.code_mut();

        let mut code_with_allocation = vec![
            Instruction::GetLocal(2),
            Instruction::I32Const(2147483647),
            Instruction::I32Store(2, 81000000),
        ];

        code_with_allocation.append(code.elements_mut());

        *code.elements_mut() = code_with_allocation;
    })
}

fn inject_noops(module: &mut Module) -> Result<(), String> {
    module.map_function("validate_block", |func_body: &mut FuncBody, _| {
        // Add half a billion NoOperations to (hopefully) slow down interpretation-time
        let code = func_body.code_mut();

        let mut nops = vec![Instruction::Nop; 500_000_000];
        nops.append(code.elements_mut());

        *code.elements_mut() = nops;
    })
}

fn inject_heap_overflow(module: &mut Module) -> Result<(), String> {
    module.map_function(
        "validate_block",
        |func_body: &mut FuncBody, malloc_index| {
            let code = func_body.code_mut();
            let index: u32 = malloc_index as u32;

            let mut code_with_allocation =
                vec![[Instruction::I32Const(33_554_431), Instruction::Call(index)]; 8]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<Instruction>>();

            code_with_allocation.append(code.elements_mut());

            *code.elements_mut() = code_with_allocation;
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::load_module_from_wasm;
    use std::path::Path;

    const WASM_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/test-wasm/test.wasm");

    fn get_function_body(module: &mut Module) -> &mut FuncBody {
        let function_name = "validate_block";
        let global_function_index = module.get_global_function_index(function_name).unwrap();
        let import_section_len = module.get_import_section_len().unwrap();
        let local_function_index = global_function_index - import_section_len;
        let function_body = module
            .get_function_body(local_function_index, function_name)
            .unwrap();

        function_body
    }

    fn load_module() -> Module {
        let module_path = Path::new(WASM_PATH);
        let module = load_module_from_wasm(module_path).unwrap();
        module
    }

    #[test]
    fn test_inject_infinite_loop() {
        let mut module = load_module();

        let injection = Injection::InfiniteLoop;
        assert!(injection.inject(&mut module).is_ok());

        let function_body = get_function_body(&mut module);

        let expected = vec![
            Instruction::Loop(BlockType::NoResult),
            Instruction::Nop,
            Instruction::Br(0),
            Instruction::End,
        ];
        assert!(function_body.code_mut().elements().starts_with(&expected))
    }

    #[test]
    fn test_inject_jibberish_return_value() {
        let mut module = load_module();
        let injection = Injection::BadReturnValue;

        assert!(injection.inject(&mut module).is_ok());

        let function_body = get_function_body(&mut module);

        let expected = vec![Instruction::I64Const(123456789), Instruction::End];
        assert!(function_body.code_mut().elements().starts_with(&expected))
    }

    #[test]
    fn test_inject_stack_overflow() {
        let mut module = load_module();

        let injection = Injection::StackOverflow;
        assert!(injection.inject(&mut module).is_ok());

        let function_body = get_function_body(&mut module);

        let expected = vec![
            Instruction::GetLocal(2),
            Instruction::I32Const(2147483647),
            Instruction::I32Store(2, 81000000),
        ];
        assert!(function_body.code_mut().elements().starts_with(&expected))
    }

    #[test]
    fn test_inject_noops() {
        let mut module = load_module();

        let injection = Injection::Noops;
        assert!(injection.inject(&mut module).is_ok());

        let function_body = get_function_body(&mut module);

        let expected = vec![Instruction::Nop; 50]; // 500_000_000 is the actual number of injected instructions
        assert!(function_body.code_mut().elements().starts_with(&expected))
    }

    #[test]
    fn test_inject_heap_overflow() {
        let mut module = load_module();

        let injection = Injection::HeapOverflow;
        assert!(injection.inject(&mut module).is_ok());

        let index = module.get_malloc_index().unwrap() as u32;
        let function_body = get_function_body(&mut module);

        let expected = vec![Instruction::I32Const(33_554_431), Instruction::Call(index)]
            .into_iter()
            .cycle()
            .take(16)
            .collect::<Vec<_>>();

        assert!(function_body.code_mut().elements().starts_with(&expected))
    }
}

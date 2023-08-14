use std::fmt::{Display, Formatter};
use wasm_instrument::parity_wasm::elements::{
    BlockType, FuncBody, Instruction, Instructions, Module,
};

use super::injector::FunctionMapper;

/// # Injection enum
///
/// This enum is used to select which injection to perform i.e what instructions to insert in the beginning of a specified export function in the WASM module.
/// # Example
///
/// ```
/// use std::path::Path;
/// use wasm_injector::{ Injection, util::load_module_from_wasm };
///
/// # fn main() -> Result<(), String> {
/// let source = Path::new("samples/example.wasm");
/// let mut module = load_module_from_wasm(source)?;   
/// let injection = Injection::InfiniteLoop;
/// injection.inject(&mut module, "validate_block")?;
/// # Ok(())
/// # }
/// ```

#[derive(clap::ValueEnum, PartialEq, Eq, Clone, Debug)]
pub enum Injection {
    InfiniteLoop,
    BadReturnValue,
    StackOverflow,
    Noops,
    HeapOverflow,
}

impl Injection {
    /// # Takes a module and injects the selected injection into the module.
    pub fn inject(self, module: &mut Module, function: &str) -> Result<(), String> {
        get_injection(self)(module, function)
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
type InjectionFn = dyn FnMut(&mut Module, &str) -> Result<(), String>;

/// # Takes an injection and returns a function that performs the injection.
fn get_injection(injection: Injection) -> Box<InjectionFn> {
    Box::new(match injection {
        Injection::InfiniteLoop => inject_infinite_loop,
        Injection::BadReturnValue => inject_bad_return_value,
        Injection::StackOverflow => inject_stack_overflow,
        Injection::Noops => inject_noops,
        Injection::HeapOverflow => inject_heap_overflow,
    })
}

/// # Takes a module and injects an infinite loop in the beginning of the module.
fn inject_infinite_loop(module: &mut Module, function_name: &str) -> Result<(), String> {
    module.map_function(function_name, |func_body: &mut FuncBody| {
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

/// # Takes a module and injects a store and  return on the last value from the stack in the beginning of the module.
fn inject_bad_return_value(module: &mut Module, function_name: &str) -> Result<(), String> {
    module.map_function(function_name, |func_body: &mut FuncBody| {
        *func_body.code_mut() = Instructions::new(vec![
            // Last value on the stack gets returned
            Instruction::I64Const(123456789),
            Instruction::End,
        ]);
    })
}

/// # Tries to store memory at an address that is out of bounds.
fn inject_stack_overflow(module: &mut Module, function_name: &str) -> Result<(), String> {
    module.map_function(function_name, |func_body: &mut FuncBody| {
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

/// # Takes a module and injects 500 million NoOperations in the beginning of the module.
fn inject_noops(module: &mut Module, function_name: &str) -> Result<(), String> {
    module.map_function(function_name, |func_body: &mut FuncBody| {
        // Add half a billion NoOperations to (hopefully) slow down interpretation-time
        let code = func_body.code_mut();

        let mut nops = vec![Instruction::Nop; 500_000_000];
        nops.append(code.elements_mut());

        *code.elements_mut() = nops;
    })
}

/// # Takes a module and tries to allocate a lot of memory multiple times in the beginning of the module.
fn inject_heap_overflow(module: &mut Module, function_name: &str) -> Result<(), String> {
    let malloc_index = &module.get_malloc_index().expect("No malloc function");

    module.map_function(function_name, |func_body: &mut FuncBody| {
        let code = func_body.code_mut();

        let mut code_with_allocation = vec![
            [
                Instruction::I32Const(33_554_431),
                Instruction::Call(*malloc_index as u32)
            ];
            8
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<Instruction>>();

        code_with_allocation.append(code.elements_mut());

        *code.elements_mut() = code_with_allocation;
    })
}

#[cfg(test)]
mod injections_tests {
    use super::*;
    use crate::util::load_module_from_wasm;
    use std::path::Path;

    const FUNCTION_NAME: &'static str = "validate_block";
    const WASM_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/samples/example.wasm");

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
        assert!(injection.inject(&mut module, FUNCTION_NAME).is_ok());

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

        assert!(injection.inject(&mut module, FUNCTION_NAME).is_ok());

        let function_body = get_function_body(&mut module);

        let expected = vec![Instruction::I64Const(123456789), Instruction::End];
        assert!(function_body.code_mut().elements().starts_with(&expected))
    }

    #[test]
    fn test_inject_stack_overflow() {
        let mut module = load_module();

        let injection = Injection::StackOverflow;
        assert!(injection.inject(&mut module, FUNCTION_NAME).is_ok());

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
        assert!(injection.inject(&mut module, FUNCTION_NAME).is_ok());

        let function_body = get_function_body(&mut module);

        let expected = vec![Instruction::Nop; 50]; // 500_000_000 is the actual number of injected instructions
        assert!(function_body.code_mut().elements().starts_with(&expected))
    }

    #[test]
    fn test_inject_heap_overflow() {
        let mut module = load_module();

        let injection = Injection::HeapOverflow;
        assert!(injection.inject(&mut module, FUNCTION_NAME).is_ok());

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

use wasm_instrument::parity_wasm::elements::{FuncBody, ImportSection, Internal::Function, Module};

/// # This trait extends the module with helper functions used for injecting code into the module.
pub trait FunctionMapper {
    fn map_function(
        &mut self,
        function_name: &str,
        body_mapper: impl Fn(&mut FuncBody, usize),
    ) -> Result<(), String>;

    fn map_functions(
        &mut self,
        function_name_body_mapper_pairs: Vec<(&str, impl Fn(&mut FuncBody, usize))>,
    ) -> Result<(), String> {
        function_name_body_mapper_pairs
            .into_iter()
            .try_for_each(|(function_name, body_mapper)| {
                self.map_function(function_name, body_mapper)
            })
    }

    fn get_global_function_index(&mut self, function_name: &str) -> Result<usize, String>;
    fn get_import_section_len(&mut self) -> Result<usize, String>;
    fn get_function_body(
        &mut self,
        local_function_index: usize,
        function_name: &str,
    ) -> Result<&mut FuncBody, String>;
    fn get_malloc_index(&mut self) -> Result<usize, String>;
}

impl FunctionMapper for Module {
    /// # Takes a module and a function name and returns the global function index of the function.
    /// 
    /// # Errors
    /// 
    /// - Returns an error if the function is not found in the export section.
    /// - Returns an error if the u32 cannot be mapped to usize.
    fn get_global_function_index(&mut self, function_name: &str) -> Result<usize, String> {
        let global_function_index: usize = self
            .export_section()
            .ok_or("No export section")?
            .entries()
            .iter()
            .find_map(|export| match export.internal() {
                Function(index) if export.field() == function_name => Some(index),
                _ => None,
            })
            .ok_or(format!(
                "Function '{}' not found in the export section",
                function_name
            ))?
            .to_owned()
            .try_into()
            .map_err(|err| format!("Couldn't map u32 to usize: {}", err))?;

        Ok(global_function_index)
    }

    /// # Takes a module and returns the length of the import section.
    /// If there is no import section, 0 is returned.
    fn get_import_section_len(&mut self) -> Result<usize, String> {
        let import_section_len: usize = self
            .import_section()
            .map(ImportSection::functions)
            .unwrap_or(0);

        Ok(import_section_len)
    }

    /// # Takes a module, a local function index and a function name and returns the function body.
    /// 
    /// # Errors
    /// - Returns an error if the code section is not found.
    /// - Returns an error if the local function index is not in the code section.
    fn get_function_body(
        &mut self,
        local_function_index: usize,
        function_name: &str,
    ) -> Result<&mut FuncBody, String> {
        let function_body = self
            .code_section_mut()
            .ok_or("No code section")?
            .bodies_mut()
            .get_mut(local_function_index)
            .ok_or(format!(
                "Function '{}' not found in the code section",
                function_name
            ))?;

        Ok(function_body)
    }

    /// # Takes a module and returns the index of the malloc function in the import section.
    fn get_malloc_index(&mut self) -> Result<usize, String> {
        let import_section = self.import_section_mut().ok_or("No table section")?;
        let malloc_index = import_section
            .entries()
            .iter()
            .enumerate()
            .find_map(|(index, entry)| {
                if entry.field().starts_with("ext_allocator_malloc") {
                    Some(index)
                } else {
                    None
                }
            })
            .ok_or("No malloc function in table section")?;

        Ok(malloc_index)
    }

    /// # Takes a module, a function name and a body mapper function and maps over the function body.
    fn map_function(
        &mut self,
        function_name: &str,
        body_mapper: impl Fn(&mut FuncBody, usize),
    ) -> Result<(), String> {
        // NOTE:
        // Indexing for local functions includes both imported and own (local)
        // functions. So we actually need the imported function count to resolve the
        // actual index of the given function in own functions list.

        // Find the global function index
        let global_function_index = self.get_global_function_index(function_name)?;

        // Count the number of imported functions listed by the module
        let import_section_len: usize = self.get_import_section_len()?;

        // get the global function index of the dynamic memory allocation method
        let malloc_index = self.get_malloc_index()?;

        // Compute the local function index (for the code section) by subtracting
        // the number of imported functions from the global function index
        let local_function_index = global_function_index - import_section_len;

        // Extract the `${function_name}` function_body
        let function_body = self.get_function_body(local_function_index, function_name)?;

        // Map over the function_body
        body_mapper(function_body, malloc_index);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::load_module_from_wasm;
    use std::path::Path;

    /// WARNING: VALUES ARE FOR TEST WASM ONLY AND WILL DIFFER FOR DIFFERENT WASM BLOBS!!!
    const VALIDATE_BLOCK_GLOBAL_INDEX: usize = 1733;
    const IMPORT_SECTION_LENGTH: usize = 39;
    const MALLOC_INDEX: usize = 25;
    const WASM_INSTRUCTION_COUNT: usize = 358;
    const WASM_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/test-wasm/test.wasm");

    fn load_module() -> Module {
        let module_path = Path::new(WASM_PATH);
        let module = load_module_from_wasm(module_path).unwrap();
        module
    }

    #[test]
    fn test_get_global_function_index() {
        let mut module = load_module();
        let global_function_index = module.get_global_function_index("validate_block").unwrap();
        assert_eq!(global_function_index, VALIDATE_BLOCK_GLOBAL_INDEX);
    }

    #[test]
    fn test_get_import_section_len() {
        let mut module = load_module();
        let import_section_len = module.get_import_section_len().unwrap();
        assert_eq!(import_section_len, IMPORT_SECTION_LENGTH);
    }

    #[test]
    fn test_get_function_body() {
        let mut module = load_module();
        let function_name = "validate_block";
        let local_function_index = module.get_global_function_index(function_name).unwrap()
            - module.get_import_section_len().unwrap();
        let function_body = module
            .get_function_body(local_function_index, function_name)
            .unwrap();
        assert_eq!(
            function_body.code().elements().len(),
            WASM_INSTRUCTION_COUNT
        );
    }

    #[test]
    fn test_get_malloc_index() {
        let mut module = load_module();
        let malloc_index = module.get_malloc_index().unwrap();
        assert_eq!(malloc_index, MALLOC_INDEX);
    }
}

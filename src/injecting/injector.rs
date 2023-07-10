use wasm_instrument::parity_wasm::elements::{FuncBody, ImportSection, Internal::Function, Module};

pub trait FunctionMapper {
    fn map_function(
        &mut self,
        function_name: &str,
        body_mapper: impl Fn(&mut FuncBody, &usize),
    ) -> Result<(), String>;

    fn map_functions(
        &mut self,
        function_name_body_mapper_pairs: Vec<(&str, impl Fn(&mut FuncBody, &usize))>,
    ) -> Result<(), String> {
        function_name_body_mapper_pairs
            .into_iter()
            .try_for_each(|(function_name, body_mapper)| {
                self.map_function(function_name, body_mapper)
            })
    }
}

impl FunctionMapper for Module {
    fn map_function(
        &mut self,
        function_name: &str,
        body_mapper: impl Fn(&mut FuncBody, &usize),
    ) -> Result<(), String> {
        // NOTE:
        // Indexing for local functions includes both imported and own (local)
        // functions. So we actually need the imported function count to resolve the
        // actual index of the given function in own functions list.

        // Find the global function index
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

        // Count the number of imported functions listed by the module
        let import_section_len: usize = self
            .import_section()
            .map(ImportSection::functions)
            .unwrap_or(0);

        // Compute the local function index (for the code section) by subtracting
        // the number of imported functions from the global function index
        let local_function_index = global_function_index - import_section_len;

        // Extract the `${function_name}` function_body
        let function_body = self
            .code_section_mut()
            .ok_or("No code section")?
            .bodies_mut()
            .get_mut(local_function_index)
            .ok_or(format!(
                "Function '{}' not found in the code section",
                function_name
            ))?;

        // Map over the function_body
        body_mapper(function_body, &global_function_index);

        Ok(())
    }
}

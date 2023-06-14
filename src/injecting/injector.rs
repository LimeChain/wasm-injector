use wasm_instrument::parity_wasm::elements::{External, FuncBody, Internal::Function, Module};

pub trait ModuleMapper {
    fn map_function(
        &mut self,
        function_name: &str,
        body_mapper: impl Fn(&mut FuncBody),
    ) -> Result<(), String>;

    fn map_functions(
        &mut self,
        function_name_body_mapper_pairs: Vec<(&str, impl Fn(&mut FuncBody))>,
    ) -> Result<(), String> {
        function_name_body_mapper_pairs
            .into_iter()
            .try_for_each(|(function_name, body_mapper)| {
                self.map_function(function_name, body_mapper)
            })
    }
}

impl ModuleMapper for Module {
    fn map_function(
        &mut self,
        function_name: &str,
        body_mapper: impl Fn(&mut FuncBody),
    ) -> Result<(), String> {
        // Find the function index
        let function_index: usize = self
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

        // This counts the number of _function_ imports listed by the module, excluding
        // the globals, since indexing for actual functions for `call` and `export` purposes
        // includes both imported and own functions. So we actually need the imported function count
        // to resolve actual index of the given function in own functions list.
        let import_section_len: usize = match self.import_section() {
            Some(import) => import
                .entries()
                .iter()
                .filter(|entry| matches!(entry.external(), &External::Function(_)))
                .count(),
            None => 0,
        };

        // Subtract the value queried in the previous step from the provided index
        // to get own function index from which we can query type next.
        let function_index_in_section = function_index - import_section_len;

        // Extract the `validate_block` instructions
        let function_body = self
            .code_section_mut()
            .ok_or("No code section")?
            .bodies_mut()
            .get_mut(function_index_in_section)
            .ok_or(format!(
                "Function '{}' not found in the code section",
                function_name
            ))?;

        // Map over the function_body
        body_mapper(function_body);

        Ok(())
    }
}

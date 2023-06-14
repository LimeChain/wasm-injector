use wasm_instrument::parity_wasm::elements::{FuncBody, Internal::Function, Module};

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
        let mut function_index: usize = self
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

        // Readjust the function index
        let index_count = self
            .import_section()
            .ok_or("No import section")?
            .entries()
            .len();
        function_index -= index_count;

        // Extract the `validate_block` instructions
        let function_body = self
            .code_section_mut()
            .ok_or("No code section")?
            .bodies_mut()
            .get_mut(function_index)
            .ok_or(format!(
                "Function '{}' not found in the code section",
                function_name
            ))?;

        // Map over the function_body
        body_mapper(function_body);

        Ok(())
    }
}

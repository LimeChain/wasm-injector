use sp_maybe_compressed_blob::{decompress, CODE_BLOB_BOMB_LIMIT};
use wasm_instrument::parity_wasm::{
    deserialize_buffer,
    elements::{FuncBody, Internal::Function, Module},
    serialize,
};

// Extract the module from the (maybe compressed) WASM bytes
pub fn module_from_blob(blob_bytes: &[u8]) -> Option<Module> {
    let blob_bytes = decompress(blob_bytes, CODE_BLOB_BOMB_LIMIT).expect("Couldn't decompress");

    deserialize_buffer(blob_bytes.as_ref()).ok()
}

pub fn blob_from_module(module: Module) -> Option<Vec<u8>> {
    serialize(module).ok()
}

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
        let function_index = self
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
            .to_owned();

        // Extract the `validate_block` instructions
        let function_body = self
            .code_section_mut()
            .ok_or("No code section")?
            .bodies_mut()
            .get_mut(function_index as usize)
            .ok_or(format!(
                "Function '{}' not found in the code section",
                function_name
            ))?;

        // Map over the function_body
        body_mapper(function_body);

        Ok(())
    }
}

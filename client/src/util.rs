/// Converts a RISC Zero image ID from [u32; 8] format to [u8; 32] format.
///
/// This conversion is necessary because RISC Zero generates image IDs as 8 32-bit integers,
/// but the on-chain Solana verifier expects a 32-byte array.
///
/// # Arguments
///
/// * `input` - The RISC Zero image ID as [u32; 8]
///
/// # Returns
///
/// * `[u8; 32]` - The converted image ID as a 32-byte array
pub fn convert_array(input: [u32; 8]) -> [u8; 32] {
    let bytes: Vec<u8> = input.iter().flat_map(|&x| x.to_le_bytes()).collect();
    bytes.try_into().expect("Failed to convert array")
}

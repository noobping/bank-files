use super::{LocalAiDraft, LocalAiError, LocalAiInput, LocalAiPatternHint};

#[cfg(feature = "local-ai")]
pub(super) fn runtime_generate_configuration_draft(
    _input: &LocalAiInput,
) -> std::result::Result<Option<LocalAiDraft>, LocalAiError> {
    let _ = std::any::type_name::<candle_core::Device>();
    let _ = std::any::type_name::<tokenizers::Tokenizer>();
    let _ = tract_onnx::onnx();
    Ok(None)
}

#[cfg(not(feature = "local-ai"))]
pub(super) fn runtime_generate_configuration_draft(
    _input: &LocalAiInput,
) -> std::result::Result<Option<LocalAiDraft>, LocalAiError> {
    Ok(None)
}

#[cfg(feature = "local-ai")]
pub(super) fn runtime_pattern_hints(
    _input: &LocalAiInput,
) -> std::result::Result<Vec<LocalAiPatternHint>, LocalAiError> {
    let _ = std::any::type_name::<candle_core::Device>();
    let _ = std::any::type_name::<tokenizers::Tokenizer>();
    let _ = tract_onnx::onnx();
    Ok(Vec::new())
}

#[cfg(not(feature = "local-ai"))]
pub(super) fn runtime_pattern_hints(
    _input: &LocalAiInput,
) -> std::result::Result<Vec<LocalAiPatternHint>, LocalAiError> {
    Ok(Vec::new())
}

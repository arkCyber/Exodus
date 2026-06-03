// Gemma4 E4B Inference Test
// Tests the newly downloaded Gemma4 E4B model with basic inference

use std::path::PathBuf;

#[tokio::test]
async fn test_gemma4_model_path_exists() {
    let model_path = PathBuf::from("../allama/models/gemma-4-E4B/gemma-4-E4B-it-Q4_K_M.gguf");
    assert!(model_path.exists(), "Gemma4 model file should exist");
    
    // Check file size is reasonable (around 4.6GB)
    let metadata = std::fs::metadata(&model_path).expect("Should be able to read file metadata");
    let size_gb = metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0);
    assert!(size_gb > 4.0 && size_gb < 5.0, "Model size should be around 4.6GB, got {:.2}GB", size_gb);
}

#[tokio::test] 
async fn test_gemma4_modelfile_exists() {
    let modelfile_path = PathBuf::from("../allama/models/gemma-4-E4B/Modelfile");
    assert!(modelfile_path.exists(), "Modelfile should exist");
    
    let content = std::fs::read_to_string(&modelfile_path).expect("Should be able to read Modelfile");
    assert!(content.contains("gemma-4-E4B-it-Q4_K_M.gguf"), "Modelfile should reference the model file");
    assert!(content.contains("PARAMETER temperature"), "Modelfile should have temperature parameter");
    assert!(content.contains("PARAMETER num_ctx"), "Modelfile should have context length parameter");
}

#[tokio::test]
async fn test_gemma4_readme_exists() {
    let readme_path = PathBuf::from("../allama/models/gemma-4-E4B/README.md");
    assert!(readme_path.exists(), "README should exist");
    
    let content = std::fs::read_to_string(&readme_path).expect("Should be able to read README");
    assert!(content.contains("Gemma4"), "README should mention Gemma4");
    assert!(content.contains("Q4_K_M"), "README should mention quantization");
}

#[tokio::test]
async fn test_gemma4_configuration_completeness() {
    let modelfile_path = PathBuf::from("../allama/models/gemma-4-E4B/Modelfile");
    let content = std::fs::read_to_string(&modelfile_path).expect("Should be able to read Modelfile");
    
    // Check for essential parameters
    let required_params = vec![
        "temperature",
        "top_p", 
        "top_k",
        "repeat_penalty",
        "num_ctx",
        "num_predict",
        "num_thread",
        "num_gpu"
    ];
    
    for param in required_params {
        assert!(content.contains(param), "Modelfile should have {} parameter", param);
    }
    
    // Check for template
    assert!(content.contains("TEMPLATE"), "Modelfile should have chat template");
    assert!(content.contains("SYSTEM"), "Modelfile should have system prompt");
}

#[tokio::test]
async fn test_gemma4_model_integrity() {
    let model_path = PathBuf::from("../allama/models/gemma-4-E4B/gemma-4-E4B-it-Q4_K_M.gguf");
    
    // Check file is readable
    let mut file = std::fs::File::open(&model_path).expect("Should be able to open model file");
    let mut buffer = [0u8; 4];
    std::io::Read::read_exact(&mut file, &mut buffer).expect("Should be able to read file header");
    
    // Check GGUF magic bytes
    let magic = std::str::from_utf8(&buffer).unwrap_or("");
    assert_eq!(magic, "GGUF", "Model file should have GGUF magic bytes");
}

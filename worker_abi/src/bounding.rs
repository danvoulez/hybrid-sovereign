use sovereign_core::{Cid, Signature};

#[derive(Debug, Clone)]
pub struct SiliconReceipt {
    pub task_cid: Cid,
    pub result_vector: Vec<i32>,
    pub hardware_signature: Signature,
}

pub fn verify_silicon_execution(
    expected_receipt: &SiliconReceipt,
    recomputed_receipt: &SiliconReceipt,
    epsilon: f32,
) -> bool {
    if expected_receipt.result_vector.len() != recomputed_receipt.result_vector.len() {
        return false;
    }
    let mut distance = 0.0f32;
    for (a, b) in expected_receipt
        .result_vector
        .iter()
        .zip(recomputed_receipt.result_vector.iter())
    {
        distance += (*a - *b).abs() as f32;
    }
    distance <= epsilon
}

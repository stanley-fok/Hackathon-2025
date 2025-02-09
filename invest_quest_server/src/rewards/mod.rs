#[derive(serde::Serialize)]
pub struct Reward {
    #[serde(rename="type")]
    reward_type: String,
    amount: u64
}
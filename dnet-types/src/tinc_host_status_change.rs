#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HostStatusChange {
    TincUp,
    TincDown,
    HostUp(String),
    HostDown(String),
}

#[test]
fn test_to_json() {
    let a = HostStatusChange::HostUp("123".to_string());
    let b = serde_json::to_string(&a);
    println!("{}", b);

    let a = HostStatusChange::HostDown("123".to_string());
    let b = serde_json::to_string(&a);
    println!("{}", b);

    let a = HostStatusChange::TincUp;
    let b = serde_json::to_string(&a);
    println!("{}", b);

    let a = HostStatusChange::TincDown;
    let b = serde_json::to_string(&a);
    println!("{}", b);
}
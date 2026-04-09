use tempfile::tempdir;
use thoughts::db::Store;

#[test]
fn test_full_save_and_search_flow() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.sqlite");
    let store = Store::open(&db_path).unwrap();
    let embedding = vec![0.1f32; 384];

    store
        .save_thought("borrow checker is strict", Some("rust"), &embedding)
        .unwrap();
    let results = store.search_vector(&embedding, 5).unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].0, 1); // first inserted row
}

use parsett_rust::parse_title;

#[test]
fn test_edge_cases() {
    let test_cases = vec![(
        "Мстители Эра Альтрона--Avengers Age of Ultron..фантастика.США.. Remux ..RUS.UKR.ENG.mkv",
        false,
    )];

    for (release_name, expected_trash) in test_cases {
        let result = parse_title(release_name).unwrap();
        assert_eq!(result.trash, expected_trash, "Failed for {}", release_name);
    }
}

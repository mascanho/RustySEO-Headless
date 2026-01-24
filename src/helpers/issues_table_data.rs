pub fn issues() -> Vec<Vec<String>> {
    vec![
        vec![" 404 Errors".to_string(), "0".to_string(), "0".to_string()],
        vec![
            " Pages Titles Above 60 Charactes".to_string(),
            "0".to_string(),
            "0".to_string(),
        ],
        vec![
            " Page Titles Below 30 Characters".to_string(),
            "0".to_string(),
            "0".to_string(),
        ],
        vec![
            " Page Descriptions > 160 Characters".to_string(),
            "0".to_string(),
            "0".to_string(),
        ],
        vec![
            " Missing Page Descriptions".to_string(),
            "0".to_string(),
            "0".to_string(),
        ],
        vec![" Missing H1".to_string(), "0".to_string(), "0".to_string()],
        vec![
            " Missing Alt Text".to_string(),
            "0".to_string(),
            "0".to_string(),
        ],
        vec![
            " Low Content Pages".to_string(),
            "0".to_string(),
            "0".to_string(),
        ],
    ]
}

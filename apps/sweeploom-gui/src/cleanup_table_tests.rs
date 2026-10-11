use super::*;
#[test]
fn unmeasured_rows_sort_last_in_both_directions_and_filter_uses_origin_path() {
    let item = |id: &str, bytes| Item {
        id: id.into(),
        name: id.into(),
        kind: Kind::Cache,
        bytes,
        usage: None,
        status: String::new(),
        scope: None,
        path: None,
        selected: false,
        enabled: false,
    };
    let mut listing = Listing {
        items: vec![
            item("/cache/unknown", None),
            item("/cache/empty", Some(0)),
            item("/cache/large", Some(100)),
        ],
        ..Default::default()
    };
    visible_indices(&mut listing, Sort::size_desc(), "");
    assert_eq!(
        listing
            .items
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        vec!["/cache/large", "/cache/empty", "/cache/unknown"]
    );
    visible_indices(
        &mut listing,
        Sort {
            col: Col::Size,
            desc: false,
        },
        "",
    );
    assert_eq!(
        listing
            .items
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        vec!["/cache/empty", "/cache/large", "/cache/unknown"]
    );
    let filtered = visible_indices(&mut listing, Sort::size_desc(), "/cache/large");
    assert_eq!(filtered.len(), 1);
    assert_eq!(listing.items[filtered[0]].id, "/cache/large");
}

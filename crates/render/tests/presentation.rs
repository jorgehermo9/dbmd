use dbmd_render::{code_block, inline_code, object_file_name, text};

#[test]
fn inline_code_uses_a_safe_fence_and_table_cell_escaping() {
    assert_eq!(inline_code("plain"), "`plain`");
    assert_eq!(inline_code("`value`"), "`` `value` ``");
    assert_eq!(inline_code("a|b\nc"), "`a\\|b<br>c`");
}

#[test]
fn code_blocks_expand_the_fence_beyond_stored_backtick_runs() {
    assert_eq!(
        code_block("sql", "SELECT ``` AS marker;"),
        "````sql\nSELECT ``` AS marker;\n````"
    );
}

#[test]
fn table_text_normalizes_newlines_and_escapes_pipes() {
    assert_eq!(text("a|b\r\nc\rd"), "a\\|b<br>c<br>d");
}

#[test]
fn object_filenames_percent_encode_every_non_slug_utf8_byte() {
    assert_eq!(
        object_file_name("weird/schema", "naïve table"),
        "weird%2Fschema.na%C3%AFve%20table.md"
    );
}

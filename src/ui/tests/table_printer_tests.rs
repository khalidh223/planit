use crate::ui::table_printer::TablePrinter;
use std::fs;

#[test]
fn table_printer_renders_expected_table_output() {
    let printer = TablePrinter::new();
    let headers = ["ID", "NAME"];
    let rows = vec![
        vec!["1".to_string(), "Alpha".to_string()],
        vec!["2".to_string(), "Beta".to_string()],
    ];
    let mut buf = Vec::new();
    printer
        .render_table("Blah", &headers, &rows, None, None, &mut buf)
        .unwrap();
    let output = String::from_utf8(buf).unwrap();
    let expected = fs::read_to_string("src/ui/tests/fixtures/blah_table.txt").unwrap();
    assert_eq!(output, expected);
}

#[test]
fn table_printer_computes_table_width() {
    let printer = TablePrinter::new();
    let headers = ["ID", "NAME"];
    let rows = vec![vec!["1", "Bob"], vec!["10", "Alice"]];
    // widths: col1 max 2, col2 max 5, plus separator spaces (3) = 10
    assert_eq!(printer.compute_table_width(&headers, &rows), 10);
}

#[test]
fn table_printer_renders_banner() {
    let printer = TablePrinter::new();
    let mut buf = Vec::new();
    printer.render_banner("abc", 5, &mut buf).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "-----\nABC\n-----\n");
}

use crate::ui::width_util::WidthUtil;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct TablePrinter {
    util: WidthUtil,
    left_pad: usize,
}

impl Default for TablePrinter {
    fn default() -> Self {
        Self {
            util: WidthUtil::default(),
            left_pad: 0,
        }
    }
}

impl TablePrinter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a clone that indents every printed line by `pad` spaces.
    pub fn with_left_pad(&self, pad: usize) -> Self {
        let mut c = self.clone();
        c.left_pad = pad;
        c
    }

    fn write_indented<W: Write + ?Sized>(&self, out: &mut W, s: &str) -> std::io::Result<()> {
        if self.left_pad > 0 {
            write!(out, "{}", " ".repeat(self.left_pad))?;
        }
        writeln!(out, "{s}")
    }

    fn write_separator<W: Write + ?Sized>(&self, out: &mut W, width: usize) -> std::io::Result<()> {
        let line = if width == 0 {
            "-".into()
        } else {
            "-".repeat(width)
        };
        self.write_indented(out, &line)
    }

    pub fn render_banner<W: Write + ?Sized>(
        &self,
        title: &str,
        width: usize,
        out: &mut W,
    ) -> std::io::Result<()> {
        let w = width.max(self.util.visible_width(title));
        self.write_separator(out, w)?;
        self.write_indented(out, &title.to_uppercase())?;
        self.write_separator(out, w)
    }

    pub fn compute_table_width<T: AsRef<str>>(&self, headers: &[&str], rows: &[Vec<T>]) -> usize {
        let col_widths = self.compute_col_widths(headers, rows);
        self.table_natural_width(&col_widths)
    }

    /// One function; `min_width` is optional.
    pub fn print_table<T: AsRef<str>>(
        &self,
        table_name: &str,
        headers: &[&str],
        rows: &[Vec<T>],
        empty_message: Option<&str>,
        min_width: Option<usize>,
    ) {
        let mut stdout = std::io::stdout();
        let _ = self.render_table(
            table_name,
            headers,
            rows,
            empty_message,
            min_width,
            &mut stdout,
        );
    }

    /// Render into any writer (used by tests to capture output).
    pub fn render_table<T: AsRef<str>, W: Write + ?Sized>(
        &self,
        table_name: &str,
        headers: &[&str],
        rows: &[Vec<T>],
        empty_message: Option<&str>,
        min_width: Option<usize>,
        out: &mut W,
    ) -> std::io::Result<()> {
        let min_w = min_width.unwrap_or(0);
        let col_widths = self.compute_col_widths(headers, rows);
        let total_width = self.table_total_width(&col_widths, min_w);

        // Empty path
        if rows.is_empty() {
            if let Some(msg) = empty_message {
                self.render_empty_state(out, table_name, msg, total_width)?;
                return Ok(());
            }
        }

        // Banner
        self.write_banner(out, table_name, total_width)?;

        // Header
        self.render_header(out, headers, &col_widths, total_width)?;

        self.render_rows(out, rows, headers.is_empty(), &col_widths, total_width)?;

        Ok(())
    }

    fn compute_col_widths<T: AsRef<str>>(
        &self,
        headers: &[&str],
        rows: &[Vec<T>],
    ) -> Vec<usize> {
        let col_count = headers.len();
        let mut col_widths = vec![0usize; col_count];
        for (i, h) in headers.iter().enumerate() {
            col_widths[i] = col_widths[i].max(self.util.visible_width(h));
        }
        for r in rows {
            for (i, cell) in r.iter().enumerate().take(col_count) {
                col_widths[i] = col_widths[i].max(self.util.visible_width(cell.as_ref()));
            }
        }
        col_widths
    }

    fn table_natural_width(&self, col_widths: &[usize]) -> usize {
        if col_widths.is_empty() {
            0
        } else {
            col_widths.iter().copied().sum::<usize>() + (col_widths.len() - 1) * 3
        }
    }

    fn table_total_width(&self, col_widths: &[usize], min_width: usize) -> usize {
        self.table_natural_width(col_widths).max(min_width)
    }

    fn render_empty_state<W: Write + ?Sized>(
        &self,
        out: &mut W,
        table_name: &str,
        msg: &str,
        total_width: usize,
    ) -> std::io::Result<()> {
        let width = total_width
            .max(self.util.visible_width(table_name))
            .max(self.util.visible_width(msg));
        self.write_banner(out, table_name, width)?;
        self.write_indented(out, msg)?;
        self.write_separator(out, width)?;
        Ok(())
    }

    fn write_banner<W: Write + ?Sized>(
        &self,
        out: &mut W,
        table_name: &str,
        width: usize,
    ) -> std::io::Result<()> {
        self.write_separator(out, width)?;
        self.write_indented(out, &table_name.to_uppercase())?;
        self.write_separator(out, width)
    }

    fn render_header<W: Write + ?Sized>(
        &self,
        out: &mut W,
        headers: &[&str],
        col_widths: &[usize],
        total_width: usize,
    ) -> std::io::Result<()> {
        if headers.is_empty() {
            return Ok(());
        }
        let line = self.build_header_line(headers, col_widths);
        self.write_indented(out, &line)?;
        self.write_separator(out, total_width)?;
        Ok(())
    }

    fn render_rows<T: AsRef<str>, W: Write + ?Sized>(
        &self,
        out: &mut W,
        rows: &[Vec<T>],
        unpadded: bool,
        col_widths: &[usize],
        total_width: usize,
    ) -> std::io::Result<()> {
        for row in rows {
            let line = if unpadded {
                self.join_row_unpadded(row)
            } else {
                self.build_row_line(row, col_widths)
            };
            self.write_indented(out, &line)?;
        }
        self.write_separator(out, total_width)
    }

    fn build_header_line(&self, headers: &[&str], col_widths: &[usize]) -> String {
        headers
            .iter()
            .enumerate()
            .map(|(i, h)| self.util.pad_visible(h, col_widths[i]))
            .collect::<Vec<_>>()
            .join(" | ")
    }

    fn build_row_line<T: AsRef<str>>(&self, row: &[T], col_widths: &[usize]) -> String {
        row.iter()
            .enumerate()
            .take(col_widths.len())
            .map(|(i, cell)| self.util.pad_visible(cell.as_ref(), col_widths[i]))
            .collect::<Vec<_>>()
            .join(" | ")
    }

    fn join_row_unpadded<T: AsRef<str>>(&self, row: &[T]) -> String {
        row.iter().map(|c| c.as_ref()).collect::<Vec<_>>().join(" | ")
    }
}

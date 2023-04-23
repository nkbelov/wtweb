use pulldown_cmark::{Event, Event::*, Tag, CowStr, Alignment, CodeBlockKind, LinkType};
use std::collections::HashMap;
use std::io::{self, Write};

enum TableState {
    Head,
    Body,
}

struct HtmlWriter<'a, I, W> {
    /// Iterator supplying events.
    iter: I,

    /// Writer to write to.
    writer: W,

    /// Whether or not the last write wrote a newline.
    end_newline: bool,

    table_state: TableState,
    table_alignments: Vec<Alignment>,
    table_cell_index: usize,
    numbers: HashMap<CowStr<'a>, usize>,
}

impl<'a, I, W> HtmlWriter<'a, I, W>
where
    I: Iterator<Item = Event<'a>>,
    W: StrWrite,
{
    fn new(iter: I, writer: W) -> Self {
        Self {
            iter,
            writer,
            end_newline: true,
            table_state: TableState::Head,
            table_alignments: vec![],
            table_cell_index: 0,
            numbers: HashMap::new(),
        }
    }

    /// Writes a new line.
    fn write_newline(&mut self) -> io::Result<()> {
        self.end_newline = true;
        self.writer.write_str("\n")
    }

    /// Writes a buffer, and tracks whether or not a newline was written.
    #[inline]
    fn write(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_str(s)?;

        if !s.is_empty() {
            self.end_newline = s.ends_with('\n');
        }
        Ok(())
    }

    fn run(mut self) -> io::Result<()> {
        while let Some(event) = self.iter.next() {
            match event {
                Start(tag) => {
                    self.start_tag(tag)?;
                }
                End(tag) => {
                    self.end_tag(tag)?;
                }
                Text(text) => {
                    escape_html(&mut self.writer, &text)?;
                    self.end_newline = text.ends_with('\n');
                }
                Code(text) => {
                    self.write("<code>")?;
                    escape_html(&mut self.writer, &text)?;
                    self.write("</code>")?;
                }
                Html(html) => {
                    self.write(&html)?;
                }
                SoftBreak => {
                    self.write_newline()?;
                }
                HardBreak => {
                    self.write("<br />\n")?;
                }
                Rule => {
                    if self.end_newline {
                        self.write("<hr />\n")?;
                    } else {
                        self.write("\n<hr />\n")?;
                    }
                }
                FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    self.write("<sup class=\"footnote-reference\"><a href=\"#")?;
                    escape_html(&mut self.writer, &name)?;
                    self.write("\">")?;
                    let number = *self.numbers.entry(name).or_insert(len);
                    write!(&mut self.writer, "{}", number)?;
                    self.write("</a></sup>")?;
                }
                TaskListMarker(true) => {
                    self.write("<input disabled=\"\" type=\"checkbox\" checked=\"\"/>\n")?;
                }
                TaskListMarker(false) => {
                    self.write("<input disabled=\"\" type=\"checkbox\"/>\n")?;
                }
            }
        }
        Ok(())
    }

    /// Writes the start of an HTML tag.
    fn start_tag(&mut self, tag: Tag<'a>) -> io::Result<()> {
        match tag {
            Tag::Paragraph => {
                if self.end_newline {
                    self.write("<p class=\"test\">")
                } else {
                    self.write("\n<p>")
                }
            }
            Tag::Heading(level, id, classes) => {
                if self.end_newline {
                    self.end_newline = false;
                    self.write("<")?;
                } else {
                    self.write("\n<")?;
                }
                write!(&mut self.writer, "{}", level)?;
                if let Some(id) = id {
                    self.write(" id=\"")?;
                    escape_html(&mut self.writer, id)?;
                    self.write("\"")?;
                }
                let mut classes = classes.iter();
                if let Some(class) = classes.next() {
                    self.write(" class=\"")?;
                    escape_html(&mut self.writer, class)?;
                    for class in classes {
                        self.write(" ")?;
                        escape_html(&mut self.writer, class)?;
                    }
                    self.write("\"")?;
                }
                self.write(">")
            }
            Tag::Table(alignments) => {
                self.table_alignments = alignments;
                self.write("<table>")
            }
            Tag::TableHead => {
                self.table_state = TableState::Head;
                self.table_cell_index = 0;
                self.write("<thead><tr>")
            }
            Tag::TableRow => {
                self.table_cell_index = 0;
                self.write("<tr>")
            }
            Tag::TableCell => {
                match self.table_state {
                    TableState::Head => {
                        self.write("<th")?;
                    }
                    TableState::Body => {
                        self.write("<td")?;
                    }
                }
                match self.table_alignments.get(self.table_cell_index) {
                    Some(&Alignment::Left) => self.write(" style=\"text-align: left\">"),
                    Some(&Alignment::Center) => self.write(" style=\"text-align: center\">"),
                    Some(&Alignment::Right) => self.write(" style=\"text-align: right\">"),
                    _ => self.write(">"),
                }
            }
            Tag::BlockQuote => {
                if self.end_newline {
                    self.write("<blockquote>\n")
                } else {
                    self.write("\n<blockquote>\n")
                }
            }
            Tag::CodeBlock(info) => {
                if !self.end_newline {
                    self.write_newline()?;
                }
                match info {
                    CodeBlockKind::Fenced(info) => {
                        let lang = info.split(' ').next().unwrap();
                        if lang.is_empty() {
                            self.write("<pre><code>")
                        } else {
                            self.write("<pre><code class=\"language-")?;
                            escape_html(&mut self.writer, lang)?;
                            self.write("\">")
                        }
                    }
                    CodeBlockKind::Indented => self.write("<pre><code>"),
                }
            }
            Tag::List(Some(1)) => {
                if self.end_newline {
                    self.write("<ol>\n")
                } else {
                    self.write("\n<ol>\n")
                }
            }
            Tag::List(Some(start)) => {
                if self.end_newline {
                    self.write("<ol start=\"")?;
                } else {
                    self.write("\n<ol start=\"")?;
                }
                write!(&mut self.writer, "{}", start)?;
                self.write("\">\n")
            }
            Tag::List(None) => {
                if self.end_newline {
                    self.write("<ul>\n")
                } else {
                    self.write("\n<ul>\n")
                }
            }
            Tag::Item => {
                if self.end_newline {
                    self.write("<li>")
                } else {
                    self.write("\n<li>")
                }
            }
            Tag::Emphasis => self.write("<em>"),
            Tag::Strong => self.write("<strong>"),
            Tag::Strikethrough => self.write("<del>"),
            Tag::Link(LinkType::Email, dest, title) => {
                self.write("<a href=\"mailto:")?;
                escape_href(&mut self.writer, &dest)?;
                if !title.is_empty() {
                    self.write("\" title=\"")?;
                    escape_html(&mut self.writer, &title)?;
                }
                self.write("\">")
            }
            Tag::Link(_link_type, dest, title) => {
                self.write("<a href=\"")?;
                escape_href(&mut self.writer, &dest)?;
                if !title.is_empty() {
                    self.write("\" title=\"")?;
                    escape_html(&mut self.writer, &title)?;
                }
                self.write("\">")
            }
            Tag::Image(_link_type, dest, title) => {
                self.write("<img src=\"")?;
                escape_href(&mut self.writer, &dest)?;
                self.write("\" alt=\"")?;
                self.raw_text()?;
                if !title.is_empty() {
                    self.write("\" title=\"")?;
                    escape_html(&mut self.writer, &title)?;
                }
                self.write("\" />")
            }
            Tag::FootnoteDefinition(name) => {
                if self.end_newline {
                    self.write("<div class=\"footnote-definition\" id=\"")?;
                } else {
                    self.write("\n<div class=\"footnote-definition\" id=\"")?;
                }
                escape_html(&mut self.writer, &*name)?;
                self.write("\"><sup class=\"footnote-definition-label\">")?;
                let len = self.numbers.len() + 1;
                let number = *self.numbers.entry(name).or_insert(len);
                write!(&mut self.writer, "{}", number)?;
                self.write("</sup>")
            }
        }
    }

    fn end_tag(&mut self, tag: Tag) -> io::Result<()> {
        match tag {
            Tag::Paragraph => {
                self.write("</p>\n")?;
            }
            Tag::Heading(level, _id, _classes) => {
                self.write("</")?;
                write!(&mut self.writer, "{}", level)?;
                self.write(">\n")?;
            }
            Tag::Table(_) => {
                self.write("</tbody></table>\n")?;
            }
            Tag::TableHead => {
                self.write("</tr></thead><tbody>\n")?;
                self.table_state = TableState::Body;
            }
            Tag::TableRow => {
                self.write("</tr>\n")?;
            }
            Tag::TableCell => {
                match self.table_state {
                    TableState::Head => {
                        self.write("</th>")?;
                    }
                    TableState::Body => {
                        self.write("</td>")?;
                    }
                }
                self.table_cell_index += 1;
            }
            Tag::BlockQuote => {
                self.write("</blockquote>\n")?;
            }
            Tag::CodeBlock(_) => {
                self.write("</code></pre>\n")?;
            }
            Tag::List(Some(_)) => {
                self.write("</ol>\n")?;
            }
            Tag::List(None) => {
                self.write("</ul>\n")?;
            }
            Tag::Item => {
                self.write("</li>\n")?;
            }
            Tag::Emphasis => {
                self.write("</em>")?;
            }
            Tag::Strong => {
                self.write("</strong>")?;
            }
            Tag::Strikethrough => {
                self.write("</del>")?;
            }
            Tag::Link(_, _, _) => {
                self.write("</a>")?;
            }
            Tag::Image(_, _, _) => (), // shouldn't happen, handled in start
            Tag::FootnoteDefinition(_) => {
                self.write("</div>\n")?;
            }
        }
        Ok(())
    }

    // run raw text, consuming end tag
    fn raw_text(&mut self) -> io::Result<()> {
        let mut nest = 0;
        while let Some(event) = self.iter.next() {
            match event {
                Start(_) => nest += 1,
                End(_) => {
                    if nest == 0 {
                        break;
                    }
                    nest -= 1;
                }
                Html(text) | Code(text) | Text(text) => {
                    escape_html(&mut self.writer, &text)?;
                    self.end_newline = text.ends_with('\n');
                }
                SoftBreak | HardBreak | Rule => {
                    self.write(" ")?;
                }
                FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    let number = *self.numbers.entry(name).or_insert(len);
                    write!(&mut self.writer, "[{}]", number)?;
                }
                TaskListMarker(true) => self.write("[x]")?,
                TaskListMarker(false) => self.write("[ ]")?,
            }
        }
        Ok(())
    }
}

/// Iterate over an `Iterator` of `Event`s, generate HTML for each `Event`, and
/// push it to a `String`.
///
/// # Examples
///
/// ```
/// use pulldown_cmark::{html, Parser};
///
/// let markdown_str = r#"
/// hello
/// =====
///
/// * alpha
/// * beta
/// "#;
/// let parser = Parser::new(markdown_str);
///
/// let mut html_buf = String::new();
/// html::push_html(&mut html_buf, parser);
///
/// assert_eq!(html_buf, r#"<h1>hello</h1>
/// <ul>
/// <li>alpha</li>
/// <li>beta</li>
/// </ul>
/// "#);
/// ```
pub fn push_html<'a, I>(s: &mut String, iter: I)
where
    I: Iterator<Item = Event<'a>>,
{
    HtmlWriter::new(iter, s).run().unwrap();
}

/// Iterate over an `Iterator` of `Event`s, generate HTML for each `Event`, and
/// write it out to a writable stream.
///
/// **Note**: using this function with an unbuffered writer like a file or socket
/// will result in poor performance. Wrap these in a
/// [`BufWriter`](https://doc.rust-lang.org/std/io/struct.BufWriter.html) to
/// prevent unnecessary slowdowns.
///
/// # Examples
///
/// ```
/// use pulldown_cmark::{html, Parser};
/// use std::io::Cursor;
///
/// let markdown_str = r#"
/// hello
/// =====
///
/// * alpha
/// * beta
/// "#;
/// let mut bytes = Vec::new();
/// let parser = Parser::new(markdown_str);
///
/// html::write_html(Cursor::new(&mut bytes), parser);
///
/// assert_eq!(&String::from_utf8_lossy(&bytes)[..], r#"<h1>hello</h1>
/// <ul>
/// <li>alpha</li>
/// <li>beta</li>
/// </ul>
/// "#);
/// ```
#[allow(unused)]
pub fn write_html<'a, I, W>(writer: W, iter: I) -> io::Result<()>
where
    I: Iterator<Item = Event<'a>>,
    W: Write,
{
    HtmlWriter::new(iter, WriteWrapper(writer)).run()
}


use std::fmt::{Arguments, Write as FmtWrite};
use std::io::{ErrorKind};
use std::str::from_utf8;

#[rustfmt::skip]
static HREF_SAFE: [u8; 128] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0,
];

static HEX_CHARS: &[u8] = b"0123456789ABCDEF";
static AMP_ESCAPE: &str = "&amp;";
static SINGLE_QUOTE_ESCAPE: &str = "&#x27;";

/// This wrapper exists because we can't have both a blanket implementation
/// for all types implementing `Write` and types of the for `&mut W` where
/// `W: StrWrite`. Since we need the latter a lot, we choose to wrap
/// `Write` types.
pub struct WriteWrapper<W>(pub W);

/// Trait that allows writing string slices. This is basically an extension
/// of `std::io::Write` in order to include `String`.
pub trait StrWrite {
    fn write_str(&mut self, s: &str) -> io::Result<()>;

    fn write_fmt(&mut self, args: Arguments) -> io::Result<()>;
}

impl<W> StrWrite for WriteWrapper<W>
where
    W: Write,
{
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.0.write_all(s.as_bytes())
    }

    #[inline]
    fn write_fmt(&mut self, args: Arguments) -> io::Result<()> {
        self.0.write_fmt(args)
    }
}

impl<'w> StrWrite for String {
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.push_str(s);
        Ok(())
    }

    #[inline]
    fn write_fmt(&mut self, args: Arguments) -> io::Result<()> {
        // FIXME: translate fmt error to io error?
        FmtWrite::write_fmt(self, args).map_err(|_| ErrorKind::Other.into())
    }
}

impl<W> StrWrite for &'_ mut W
where
    W: StrWrite,
{
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        (**self).write_str(s)
    }

    #[inline]
    fn write_fmt(&mut self, args: Arguments) -> io::Result<()> {
        (**self).write_fmt(args)
    }
}

/// Writes an href to the buffer, escaping href unsafe bytes.
pub fn escape_href<W>(mut w: W, s: &str) -> io::Result<()>
where
    W: StrWrite,
{
    let bytes = s.as_bytes();
    let mut mark = 0;
    for i in 0..bytes.len() {
        let c = bytes[i];
        if c >= 0x80 || HREF_SAFE[c as usize] == 0 {
            // character needing escape

            // write partial substring up to mark
            if mark < i {
                w.write_str(&s[mark..i])?;
            }
            match c {
                b'&' => {
                    w.write_str(AMP_ESCAPE)?;
                }
                b'\'' => {
                    w.write_str(SINGLE_QUOTE_ESCAPE)?;
                }
                _ => {
                    let mut buf = [0u8; 3];
                    buf[0] = b'%';
                    buf[1] = HEX_CHARS[((c as usize) >> 4) & 0xF];
                    buf[2] = HEX_CHARS[(c as usize) & 0xF];
                    let escaped = from_utf8(&buf).unwrap();
                    w.write_str(escaped)?;
                }
            }
            mark = i + 1; // all escaped characters are ASCII
        }
    }
    w.write_str(&s[mark..])
}

const fn create_html_escape_table() -> [u8; 256] {
    let mut table = [0; 256];
    table[b'"' as usize] = 1;
    table[b'&' as usize] = 2;
    table[b'<' as usize] = 3;
    table[b'>' as usize] = 4;
    table
}

static HTML_ESCAPE_TABLE: [u8; 256] = create_html_escape_table();

static HTML_ESCAPES: [&str; 5] = ["", "&quot;", "&amp;", "&lt;", "&gt;"];

/// Writes the given string to the Write sink, replacing special HTML bytes
/// (<, >, &, ") by escape sequences.
pub fn escape_html<W: StrWrite>(w: W, s: &str) -> io::Result<()> {
    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    {
        simd::escape_html(w, s)
    }
    #[cfg(not(all(target_arch = "x86_64", feature = "simd")))]
    {
        escape_html_scalar(w, s)
    }
}

fn escape_html_scalar<W: StrWrite>(mut w: W, s: &str) -> io::Result<()> {
    let bytes = s.as_bytes();
    let mut mark = 0;
    let mut i = 0;
    while i < s.len() {
        match bytes[i..]
            .iter()
            .position(|&c| HTML_ESCAPE_TABLE[c as usize] != 0)
        {
            Some(pos) => {
                i += pos;
            }
            None => break,
        }
        let c = bytes[i];
        let escape = HTML_ESCAPE_TABLE[c as usize];
        let escape_seq = HTML_ESCAPES[escape as usize];
        w.write_str(&s[mark..i])?;
        w.write_str(escape_seq)?;
        i += 1;
        mark = i; // all escaped characters are ASCII
    }
    w.write_str(&s[mark..])
}

#[cfg(all(target_arch = "x86_64", feature = "simd"))]
mod simd {
    use super::StrWrite;
    use std::arch::x86_64::*;
    use std::io;
    use std::mem::size_of;

    const VECTOR_SIZE: usize = size_of::<__m128i>();

    pub(super) fn escape_html<W: StrWrite>(mut w: W, s: &str) -> io::Result<()> {
        // The SIMD accelerated code uses the PSHUFB instruction, which is part
        // of the SSSE3 instruction set. Further, we can only use this code if
        // the buffer is at least one VECTOR_SIZE in length to prevent reading
        // out of bounds. If either of these conditions is not met, we fall back
        // to scalar code.
        if is_x86_feature_detected!("ssse3") && s.len() >= VECTOR_SIZE {
            let bytes = s.as_bytes();
            let mut mark = 0;

            unsafe {
                foreach_special_simd(bytes, 0, |i| {
                    let escape_ix = *bytes.get_unchecked(i) as usize;
                    let replacement =
                        super::HTML_ESCAPES[super::HTML_ESCAPE_TABLE[escape_ix] as usize];
                    w.write_str(&s.get_unchecked(mark..i))?;
                    mark = i + 1; // all escaped characters are ASCII
                    w.write_str(replacement)
                })?;
                w.write_str(&s.get_unchecked(mark..))
            }
        } else {
            super::escape_html_scalar(w, s)
        }
    }

    /// Creates the lookup table for use in `compute_mask`.
    const fn create_lookup() -> [u8; 16] {
        let mut table = [0; 16];
        table[(b'<' & 0x0f) as usize] = b'<';
        table[(b'>' & 0x0f) as usize] = b'>';
        table[(b'&' & 0x0f) as usize] = b'&';
        table[(b'"' & 0x0f) as usize] = b'"';
        table[0] = 0b0111_1111;
        table
    }

    #[target_feature(enable = "ssse3")]
    /// Computes a byte mask at given offset in the byte buffer. Its first 16 (least significant)
    /// bits correspond to whether there is an HTML special byte (&, <, ", >) at the 16 bytes
    /// `bytes[offset..]`. For example, the mask `(1 << 3)` states that there is an HTML byte
    /// at `offset + 3`. It is only safe to call this function when
    /// `bytes.len() >= offset + VECTOR_SIZE`.
    unsafe fn compute_mask(bytes: &[u8], offset: usize) -> i32 {
        debug_assert!(bytes.len() >= offset + VECTOR_SIZE);

        let table = create_lookup();
        let lookup = _mm_loadu_si128(table.as_ptr() as *const __m128i);
        let raw_ptr = bytes.as_ptr().offset(offset as isize) as *const __m128i;

        // Load the vector from memory.
        let vector = _mm_loadu_si128(raw_ptr);
        // We take the least significant 4 bits of every byte and use them as indices
        // to map into the lookup vector.
        // Note that shuffle maps bytes with their most significant bit set to lookup[0].
        // Bytes that share their lower nibble with an HTML special byte get mapped to that
        // corresponding special byte. Note that all HTML special bytes have distinct lower
        // nibbles. Other bytes either get mapped to 0 or 127.
        let expected = _mm_shuffle_epi8(lookup, vector);
        // We compare the original vector to the mapped output. Bytes that shared a lower
        // nibble with an HTML special byte match *only* if they are that special byte. Bytes
        // that have either a 0 lower nibble or their most significant bit set were mapped to
        // 127 and will hence never match. All other bytes have non-zero lower nibbles but
        // were mapped to 0 and will therefore also not match.
        let matches = _mm_cmpeq_epi8(expected, vector);

        // Translate matches to a bitmask, where every 1 corresponds to a HTML special character
        // and a 0 is a non-HTML byte.
        _mm_movemask_epi8(matches)
    }

    /// Calls the given function with the index of every byte in the given byteslice
    /// that is either ", &, <, or > and for no other byte.
    /// Make sure to only call this when `bytes.len() >= 16`, undefined behaviour may
    /// occur otherwise.
    #[target_feature(enable = "ssse3")]
    unsafe fn foreach_special_simd<F>(
        bytes: &[u8],
        mut offset: usize,
        mut callback: F,
    ) -> io::Result<()>
    where
        F: FnMut(usize) -> io::Result<()>,
    {
        // The strategy here is to walk the byte buffer in chunks of VECTOR_SIZE (16)
        // bytes at a time starting at the given offset. For each chunk, we compute a
        // a bitmask indicating whether the corresponding byte is a HTML special byte.
        // We then iterate over all the 1 bits in this mask and call the callback function
        // with the corresponding index in the buffer.
        // When the number of HTML special bytes in the buffer is relatively low, this
        // allows us to quickly go through the buffer without a lookup and for every
        // single byte.

        debug_assert!(bytes.len() >= VECTOR_SIZE);
        let upperbound = bytes.len() - VECTOR_SIZE;
        while offset < upperbound {
            let mut mask = compute_mask(bytes, offset);
            while mask != 0 {
                let ix = mask.trailing_zeros();
                callback(offset + ix as usize)?;
                mask ^= mask & -mask;
            }
            offset += VECTOR_SIZE;
        }

        // Final iteration. We align the read with the end of the slice and
        // shift off the bytes at start we have already scanned.
        let mut mask = compute_mask(bytes, upperbound);
        mask >>= offset - upperbound;
        while mask != 0 {
            let ix = mask.trailing_zeros();
            callback(offset + ix as usize)?;
            mask ^= mask & -mask;
        }
        Ok(())
    }

    #[cfg(test)]
    mod html_scan_tests {
        #[test]
        fn multichunk() {
            let mut vec = Vec::new();
            unsafe {
                super::foreach_special_simd("&aXaaaa.a'aa9a<>aab&".as_bytes(), 0, |ix| {
                    Ok(vec.push(ix))
                })
                .unwrap();
            }
            assert_eq!(vec, vec![0, 14, 15, 19]);
        }

        // only match these bytes, and when we match them, match them VECTOR_SIZE times
        #[test]
        fn only_right_bytes_matched() {
            for b in 0..255u8 {
                let right_byte = b == b'&' || b == b'<' || b == b'>' || b == b'"';
                let vek = vec![b; super::VECTOR_SIZE];
                let mut match_count = 0;
                unsafe {
                    super::foreach_special_simd(&vek, 0, |_| {
                        match_count += 1;
                        Ok(())
                    })
                    .unwrap();
                }
                assert!((match_count > 0) == (match_count == super::VECTOR_SIZE));
                assert_eq!(
                    (match_count == super::VECTOR_SIZE),
                    right_byte,
                    "match_count: {}, byte: {:?}",
                    match_count,
                    b as char
                );
            }
        }
    }
}

#[cfg(test)]
mod test {
    pub use super::escape_href;

    #[test]
    fn check_href_escape() {
        let mut s = String::new();
        escape_href(&mut s, "&^_").unwrap();
        assert_eq!(s.as_str(), "&amp;^_");
    }
}
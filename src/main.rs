use lol_html::{element, HtmlRewriter, Settings, OutputSink};
use std::io::Write;

struct RewriterWriteWrap<'h, O: OutputSink>(HtmlRewriter<'h, O>);

impl<'h, O: OutputSink> std::io::Write for RewriterWriteWrap<'h, O> {
    fn write(&mut self, slice: &[u8]) -> Result<usize, std::io::Error> {
        let len = slice.len();
        self.0.write(slice)
            .map(|_| len)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.0
            .end()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();

    if args.len() < 2 {
        eprintln!(r#"
usage: hq [[selector] [replacement]]...

    Replace HTML in stdin at a given CSS selector with the provided replacement.

    Example:

    hq .foo "<div>bar</div>"

    hq .foo @baz.html # inline source from baz.html
        "#);
        std::process::exit(1);
    }

    let selector = args[0].to_string();
    let replacement = if args[1].starts_with("@") {
        std::fs::read_to_string(&args[1][1..])?
    } else {
        args[1].to_string()
    };

    let rewriter = HtmlRewriter::try_new(
        Settings {
            element_content_handlers: vec![
                // Rewrite insecure hyperlinks
                element!(selector, |el| {
                    el.set_inner_content(replacement.as_str(), lol_html::html_content::ContentType::Html);
                    Ok(())
                })
            ],
            ..Settings::default()
        },
        |c: &[u8]| { std::io::stdout().write(c).expect("Failed to flush output"); }
    ).unwrap();

    std::io::copy(&mut std::io::stdin(), &mut RewriterWriteWrap(rewriter))?;

    Ok(())
}

use std::path::{Component, Path, PathBuf};

use gpui::{ImageSource, Resource, SharedUri};

const NUMBERED_PREFIXES_1: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const NUMBERED_PREFIXES_2: &str = "abcdefghijklmnopqrstuvwxyz";

const BULLETS: [&str; 5] = ["•", "◦", "▪", "‣", "⁃"];

/// Returns the prefix for a list item.
pub(super) fn list_item_prefix(ix: usize, ordered: bool, depth: usize) -> String {
    if ordered {
        if depth == 0 {
            return format!("{}. ", ix + 1);
        }

        if depth == 1 {
            return format!(
                "{}. ",
                NUMBERED_PREFIXES_1
                    .chars()
                    .nth(ix % NUMBERED_PREFIXES_1.len())
                    .unwrap()
            );
        } else {
            return format!(
                "{}. ",
                NUMBERED_PREFIXES_2
                    .chars()
                    .nth(ix % NUMBERED_PREFIXES_2.len())
                    .unwrap()
            );
        }
    } else {
        let depth = depth.min(BULLETS.len() - 1);
        let bullet = BULLETS[depth];
        return format!("{} ", bullet);
    }
}

/// Converts a document image URL into an [`ImageSource`] without granting
/// implicit filesystem access.
///
/// Document-provided values remain URI-backed, including `file://` and
/// scheme-less strings, unless the app names a `base` directory
/// ([`MarkdownExtensions::image_base_dir`](super::MarkdownExtensions::image_base_dir)):
/// then a plain relative path is read from under it.
pub(super) fn image_source(url: &SharedUri, base: Option<&Path>) -> ImageSource {
    if let Some(path) = base.and_then(|base| relative_image_path(url, base)) {
        return ImageSource::Resource(Resource::Path(path.into()));
    }
    url.clone().into()
}

/// The file a relative image URL names under `base`.
///
/// `None` for anything that is not a plain relative path: a URL with a
/// scheme (which also covers a drive letter, `C:`), a rooted path, or a UNC
/// path. The query and fragment are dropped and `%XX` escapes decoded, the
/// way a browser resolves `![](docs/my%20shot.png#dark)`.
pub(super) fn relative_image_path(url: &str, base: &Path) -> Option<PathBuf> {
    let url = url.split(['#', '?']).next().unwrap_or_default();
    if url.is_empty() || has_scheme(url) || url.starts_with(['/', '\\']) {
        return None;
    }
    let decoded = percent_decoded(url)?;
    let path = Path::new(&decoded);
    if path
        .components()
        .any(|part| matches!(part, Component::Prefix(_) | Component::RootDir))
    {
        return None;
    }
    Some(base.join(path))
}

/// RFC 3986: `ALPHA *( ALPHA / DIGIT / "+" / "-" / "." ) ":"`.
fn has_scheme(url: &str) -> bool {
    let Some((scheme, _)) = url.split_once(':') else {
        return false;
    };
    let mut chars = scheme.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// `%XX` escapes decoded; `None` when the result is not UTF-8.
fn percent_decoded(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut ix = 0;
    while ix < bytes.len() {
        let hex = bytes
            .get(ix + 1..ix + 3)
            .filter(|hex| hex.iter().all(u8::is_ascii_hexdigit))
            .and_then(|hex| std::str::from_utf8(hex).ok())
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match (bytes[ix], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                ix += 3;
            }
            (byte, _) => {
                out.push(byte);
                ix += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use gpui::{ImageSource, Resource};

    use crate::text::utils::{image_source, list_item_prefix, relative_image_path};

    #[test]
    fn relative_image_urls_resolve_under_the_base_and_nothing_else_does() {
        let base = Path::new("repo");
        let resolved = |url: &str| relative_image_path(url, base);

        assert_eq!(resolved("assets/hero.png"), Some(base.join("assets/hero.png")));
        assert_eq!(resolved("./a.png"), Some(base.join("./a.png")));
        assert_eq!(resolved("../up.png"), Some(base.join("../up.png")));
        assert_eq!(resolved("my%20shot.png#dark"), Some(base.join("my shot.png")));
        assert_eq!(resolved("a.png?raw=true"), Some(base.join("a.png")));
        assert_eq!(resolved("docs/a:b.png"), Some(base.join("docs/a:b.png")));
        assert_eq!(resolved("100%.png"), Some(base.join("100%.png")), "a lone % stays");

        for url in [
            "https://example.com/logo.png",
            "data:image/png;base64,iVBORw0KGgo=",
            "file:///C:/secret.png",
            r"C:\images\logo.png",
            "C:logo.png",
            "/absolute/logo.png",
            r"\\attacker\share\x.png",
            "//attacker/share/x.png",
            "",
            "#only-a-fragment",
            "%ff%fe.png",
        ] {
            assert_eq!(resolved(url), None, "{url:?} must not resolve");
        }
    }

    #[test]
    fn an_image_source_reads_the_disk_only_under_a_base() {
        let url = "assets/hero.png".to_string().into();
        assert!(matches!(
            image_source(&url, None),
            ImageSource::Resource(Resource::Uri(_))
        ));
        match image_source(&url, Some(Path::new("repo"))) {
            ImageSource::Resource(Resource::Path(path)) => {
                assert_eq!(path.as_ref(), PathBuf::from("repo").join("assets/hero.png"))
            }
            _ => panic!("expected a path under the base"),
        }
        let remote = "https://example.com/a.png".to_string().into();
        assert!(matches!(
            image_source(&remote, Some(Path::new("repo"))),
            ImageSource::Resource(Resource::Uri(_))
        ));
    }

    #[test]
    fn test_image_source() {
        fn source(url: &str) -> Resource {
            match image_source(&url.to_string().into(), None) {
                ImageSource::Resource(resource) => resource,
                _ => panic!("expected a resource for {url:?}"),
            }
        }
        fn assert_uri(url: &str) {
            match source(url) {
                Resource::Uri(uri) => assert_eq!(uri.as_ref(), url),
                other => panic!("expected Uri for {url:?}, got {other:?}"),
            }
        }
        assert_uri("https://example.com/logo.png");
        assert_uri("http://example.com/logo.png");
        assert_uri("data:image/png;base64,iVBORw0KGgo=");

        assert_uri("website/public/logo.svg");
        assert_uri("./images/a.png");
        assert_uri("../images/a.png");
        assert_uri("/absolute/path/logo.svg");
        assert_uri("file:///absolute/path/logo.svg");
        assert_uri(r"C:\images\logo.png");
        assert_uri("docs/a:b.png");
    }

    #[test]
    fn test_list_item_prefix() {
        assert_eq!(list_item_prefix(0, true, 0), "1. ");
        assert_eq!(list_item_prefix(1, true, 0), "2. ");
        assert_eq!(list_item_prefix(2, true, 0), "3. ");
        assert_eq!(list_item_prefix(10, true, 0), "11. ");
        assert_eq!(list_item_prefix(0, true, 1), "A. ");
        assert_eq!(list_item_prefix(1, true, 1), "B. ");
        assert_eq!(list_item_prefix(2, true, 1), "C. ");
        assert_eq!(list_item_prefix(0, true, 2), "a. ");
        assert_eq!(list_item_prefix(1, true, 2), "b. ");
        assert_eq!(list_item_prefix(6, true, 2), "g. ");
        assert_eq!(list_item_prefix(0, true, 1), "A. ");
        assert_eq!(list_item_prefix(0, true, 2), "a. ");
        assert_eq!(list_item_prefix(0, false, 0), "• ");
        assert_eq!(list_item_prefix(0, false, 1), "◦ ");
        assert_eq!(list_item_prefix(0, false, 2), "▪ ");
        assert_eq!(list_item_prefix(0, false, 3), "‣ ");
        assert_eq!(list_item_prefix(0, false, 4), "⁃ ");
    }
}

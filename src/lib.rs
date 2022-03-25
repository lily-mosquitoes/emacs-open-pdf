use emacs::{defun, Env, Value};
use emacs_org_link_parser as org;
use lopdf::Document;
use anyhow::Result;
use std::path::Path;

emacs::plugin_is_GPL_compatible!();

#[emacs::module(name = "open-pdf")]
fn init(_: &Env) -> Result<()> {
    Ok(())
}

/// Returns as a tuple (reference, page_number) the first match of a Link of the form:
/// Link.link == Some("#somereference") &&
/// Link.description == Some("pSOMENUMBER")
fn first_reference_and_page(links: Vec<org::Link>) -> Option<(String, u32)> {
    for link in links {
        match (link.link, link.description) {
            (Some(reference), Some(description)) => {
                if reference.starts_with("#") && description.starts_with("p") {
                    if let Ok(n) = description[1..].parse::<u32>() {
                        return Some((reference, n));
                    }
                }
            }
            _ => continue,
        }
    }

    None
}

/// Extracts the given page (+1) from the given PDF file, and saves a new PDF file in ./tmp
fn create_new_pdf_from(path: &Path, page_to_keep: &u32) -> Result<()> {
    let mut doc = Document::load(path)?;

    let page_num = doc.get_pages().len() as u32;
    let pages_to_delete: Vec<u32> = (1_u32..=page_num).into_iter()
        .filter( |p| p < page_to_keep || p > &(page_to_keep+1) )
        .collect();

    doc.delete_pages(&pages_to_delete);

    doc.compress();

    let modified_path = Path::new("./tmp/open-pdf-tmp.pdf");
    doc.save(&modified_path)?;

    Ok(())
}

/// Retrieves page from PDF file from link in line, for example passing in the line [[#filename][p32]] would open 'filename.pdf' as a temp file containing only pages 32 and 33
#[defun]
fn from(env: &Env, line: String) -> Result<Value<'_>> {
    let links: Vec<org::Link> = org::parse_line(&line);

    let (reference, page) = match first_reference_and_page(links) {
        Some((reference, page)) => (reference, page),
        None => return env.message("no page reference link in line, links must be of the form [[#reference][pNUMBER]]".to_string()),
    };

    let path = format!("./bibtex-pdfs/{}.pdf", &reference[1..]);
    let path = Path::new(&path);

    create_new_pdf_from(&path, &page)?;

    env.call("org-open-file", (path.to_str(),))
}

use emacs::{defun, Env, Value};
use emacs_org_link_parser as org;
use lopdf::Document;
use anyhow::{Result, Error};
use std::path::Path;

emacs::plugin_is_GPL_compatible!();

#[emacs::module(name = "open-pdf")]
fn init(_: &Env) -> Result<()> {
    Ok(())
}

/// Returns as a tuple (reference, page, num_of_pages) the first match of a Link of the form:
/// Link.link == Some("#somereference") &&
/// Link.description == Some("pSOMENUMBER+num_of_pages")
fn first_reference_and_pages(links: Vec<org::Link>) -> Option<(String, u32, u32)> {
    for link in links {
        match (link.link, link.description) {
            (Some(reference), Some(description)) => {
                if reference.starts_with("#") && description.starts_with("p") {
                    let v: Vec<&str> = description.split('+').collect();
                    match v.len() {
                        1 => if let Ok(page) = v[0][1..].parse::<u32>() {
                            return Some((reference, page, 1));
                        },
                        2 => {
                            let page = v[0][1..].parse::<u32>();
                            let num_of_pages = v[1].parse::<u32>();
                            if page.is_ok() && num_of_pages.is_ok() {
                                return Some((reference, page.unwrap(), num_of_pages.unwrap()));
                            }
                        },
                        _ => continue,
                    }
                }
            }
            _ => continue,
        }
    }

    None
}

/// Extracts the given page (+1) from the given PDF file, and saves a new PDF file in ./tmp
fn create_new_pdf_from(env: &Env, work_dir: String, path: &Path, page_to_keep: &u32, num_of_pages: &u32) -> Result<String> {
    env.message(&format!("attempting to read: {:?}", path))?;
    let mut doc = Document::load(path)?;

    let page_num = doc.get_pages().len() as u32;
    let pages_to_delete: Vec<u32> = (1_u32..=page_num).into_iter()
        .filter( |p| p < page_to_keep || p > &(page_to_keep+num_of_pages) )
        .collect();

    doc.delete_pages(&pages_to_delete);

    doc.compress();

    let modified_path = format!("{}/tmp/open-pdf-tmp.pdf", work_dir);
    let modified_path = Path::new(&modified_path);
    env.message(&format!("attempting to save: {:?}", modified_path))?;
    doc.save(&modified_path)?;

    Ok(modified_path.to_str()
        .ok_or(Error::msg("path could not be parsed"))?
        .to_owned())
}

/// Retrieves page from PDF file from link in line, for example passing in the line [[#filename][p32]] would open 'filename.pdf' as a temp file containing only pages 32 and 33
#[defun]
fn from(env: &Env, work_dir: String, line: String) -> Result<Value<'_>> {
    let links: Vec<org::Link> = org::parse_line(&line);

    let (reference, page, num) = match first_reference_and_pages(links) {
        Some((reference, page, num)) => (reference, page, num),
        None => return env.message("no page reference link in line, links must be of the form [[#reference][pNUMBER(+num_of_pages)]] where (+num_of_pages) is optional".to_string()),
    };

    let path = format!("{}/bibtex-pdfs/{}.pdf", work_dir, &reference[1..]);
    let path = Path::new(&path);

    let modified_path = create_new_pdf_from(env, work_dir, &path, &page, &num)?;

    env.call("org-open-file", (modified_path.as_str(),))
}

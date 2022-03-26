# Emacs Open PDF

This is a dynamic module for Emacs, built with Rust using the [emacs crate](https://crates.io/crates/emacs). The objective of this module is to allow one to open specific pages of a PDF from a link in [Org-mode](https://orgmode.org/).

## how to use:

First make sure your Emacs was built with [dynamic module support](https://www.gnu.org/software/emacs/manual/html_node/elisp/Dynamic-Modules.html).

You can compile the shared object library from source using `cargo build --release`, in which case it will be located in `./target/release` with the name `libemacs_open_pdf.so`. Because the module name exposed to Emacs is actually `open-pdf`, if you want Emacs to auto-discover the file from a loaded path you can either create a symlink (e.g. `ln -s libemacs_open_pdf.so open-pdf.so`) or simply rename the file to the correct name (e.g. `mv libemacs_open_pdf.so open-pdf.so`).

Alternatively you may download a pre-compiled binary from [releases](https://github.com/lily-mosquitoes/emacs-open-pdf/releases/).

When attempting to load the module, make sure Org-mode loads before it, as this module uses an Org-mode specific function (`'org-open-file`). For example, you can simply `(require 'org)` before loading this module as shown below.

Then you may load the module, wrap it with an interactive function, and bind that to some key combination (e.g. `C-c f`) as shown below:
```lisp
(require 'open-pdf "/absolute/path/to/file/open-pdf.so")

(defun open-pdf-wrapper ()
  (interactive)
  (open-pdf-from "/absolute/path/to/storage/dir" (thing-at-point 'line)))

(define-key org-mode-map "\C-cf" 'open-pdf-wrapper)
```
where the given storage directory is assumed to contain a directory called `bibtex-pdfs`, where your PDFs are contained, and a directory called `tmp`, where a temporary PDF file will be created with only the requested pages (see [example](#example)).

It is necessary to wrap the module function `open-pdf-from` into another function because it is not interactive and would not have access to calls to `thing-at-point`, which is desirable for opening links at the cursor.

### example

Example `notes.org`:
```emacs
* recipes
** favorites
*** [[#my_recipe_book][p53+4]], dumplings, [[https://www.sauce.com][sauce to go with]]
```
![image of example notes.org](https://imgur.com/SKk4MLU.png)

where our desired link is `[[#my_recipe_book][p53+4]]` and `my_recipe_book.pdf` is in `bibtex-pdfs`.

When you press the configured key combination (e.g. `C-c f`) on the line containing the desired link, a PDF with the specified pages (e.g. page 53 plus 4 pages) should open with whatever program was configured for opening PDF files in Org.

use printpdf::*;
pub fn test() {
    let (doc, p1, l1) = PdfDocument::new("Doc title", Mm(210.0), Mm(297.0), "Layer 1");
}

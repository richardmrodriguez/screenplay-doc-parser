use std::fmt::Error;

use crate::pdf_document;
use crate::pdf_document::TextPosition;
use crate::pdf_parser;
use crate::screenplay_document;
use mupdf_basic_text_extractor;

pub fn get_screenplay_doc_from_filepath(path: String) -> Result<screenplay_document::ScreenplayDocument, Box<dyn std::error::Error>>{
    use mupdf_basic_text_extractor:: {Doc, Fragment, Line, Page};
    let doc_result: Result<Doc, Box<dyn std::error::Error>> = mupdf_basic_text_extractor::get_structured_document_from_filepath(path);

    match doc_result {
        Err(e) => {
            return Err(e);

        }
        Ok(old_pdf_doc) => {

            let mut new_doc = pdf_document::PDFDocument::default();
        
            for page in old_pdf_doc.pages {
                let mut new_page = pdf_document::Page::default();
        
                for line in page.lines {
                    let mut new_line = pdf_document::Line::default();
                    for frag in line.text_fragments {
                        let mut new_word = pdf_document::Word {
                            text: frag.text,
                            bbox_height: frag.bbox_height,
                            bbox_width: frag.bbox_width,
                            position: TextPosition {
                                x: frag.x,
                                y: frag.y
                            },
                            font_name: frag.font_name,
                            font_size: frag.font_size,
                            font_character_width: 7.2
                        };
                        new_line.words.push(new_word);
        
                    }
                    new_page.lines.push(new_line);
                }
                new_doc.pages.push(new_page);
            }
        
            if let Some(parsed_screenplay) = pdf_parser::get_screenplay_doc_from_pdf_obj(
                new_doc, 
                None, 
                None, 
                None, 
                None
            ) {
                return Ok(parsed_screenplay);
            }
            return Err(Box::new(Error));
        }
    }


}
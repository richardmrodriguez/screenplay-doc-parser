use std::fmt::Error;

pub mod screenplay_document;
pub mod pdf_document;

pub mod pdf_parser;

#[cfg(test)]
mod tests {

    use std::default;

    use crate::{pdf_document::{ElementIndentationsPoints, PDFDocument, TextPosition}, screenplay_document::SPType};

    use super::*;

    fn _create_pdfline_with_word(text: String, element_indentation: f64, y_height_inches: Option<f64>) -> pdf_document::Line {
        let mut new_word = pdf_document::Word::default();

        if let Some(inches) = y_height_inches {
            new_word = _create_pdfword(text, element_indentation, y_height_inches);
        }
        else {
            new_word = _create_pdfword(text, element_indentation, None);
        }

        let new_line:pdf_document::Line = pdf_document::Line { 
            words: vec![new_word] 
        };
        new_line
    }

    fn _create_pdfword(text: String, element_indentation: f64, y_height_inches: Option<f64>) -> pdf_document::Word {
        let mut y_height_pts = 0.0;
        if let Some(inches) = y_height_inches {
            y_height_pts = 72.0 * inches;
        }
        else {
            y_height_pts = 3.0 * 72.0;
        }
        
        let new_word: pdf_document::Word = pdf_document::Word {
            text: text.clone(), 
            text_bbox_width: text.len() as f64 * 7.2 as f64, 
            position: TextPosition {
                x: element_indentation,
                y: y_height_pts
            }, 
            font_name: None, 
            font_size: 12.0, 
            font_character_width: 7.2 
        };
        new_word
    }

    //#[test]
    fn it_works() {
        let mut mock_pdf:pdf_document::PDFDocument = PDFDocument::default();
        let mut new_page = pdf_document::Page::default();
        
        let action_word  = _create_pdfword(
            "Action!".to_string(), 72.0*1.5, None);
        let mut new_line: pdf_document::Line = pdf_document::Line::default();
        new_line.words.push(action_word);
        new_page.lines.push(new_line);
        mock_pdf.pages.push(new_page);
        //println!("Adding!...");
        let parse_result_doc = pdf_parser::get_screenplay_doc_from_pdf_obj(mock_pdf, 
        None,
        None,
        screenplay_document::TimeOfDay::default());
        if let Some(document) = parse_result_doc {
            if let Some(first_page) = document.pages.first() {
                println!("First page exists!");
                if let Some(first_line) = first_page.lines.first() {
                    println!("First line exists!");
                    if let Some (first_word) = first_line.text_elements.first() {
                        println!("First Word exists!");
                    }
                } 
            }

            let word_text = document.pages.first().unwrap()
            .lines.first().unwrap()
            .text_elements.first().unwrap().text.clone();
            println!("Text: {}", word_text);
        }

    }

    #[test]
    fn all_screenplay_element_types() {

        //TODO: Use a LineCounter variable to decrement the line height for 
        // each subsequent line
        // double spacing for unrelated lines types,
        // Single spacing for CHARACTER, PARENTHETICAL, and DIALOGUE since
        // they are close together like that
        // single spacing is 12.0 since the default font is 12-point courier

        let indentations = ElementIndentationsPoints::us_letter_default(&None);


        println!(" ------ Testing Screenplay Element Types ------ ");
        println!("");

        let mut mock_pdf:pdf_document::PDFDocument = PDFDocument::default();
        let mut new_page = pdf_document::Page::default();
        
        new_page.lines.push(
            _create_pdfline_with_word("Action!".to_string(), 
            indentations.action, 
            None)
        );
        new_page.lines.push(
            _create_pdfline_with_word("CHARACTER".to_string(), 
            indentations.character, 
            None)
        );
        new_page.lines.push(
            _create_pdfline_with_word("(wryly)".to_string(), 
            indentations.parenthetical, 
            None)
        );
        new_page.lines.push(
            _create_pdfline_with_word("Dialogue".to_string(), 
            indentations.dialogue, 
            None)
        );

        let pn: String = "256ABC.".to_string();

        // Page Number / Page Revision Date
        // Rests at y-height of top margin
        // Is right-aligned to the right-hand margin
        let mut page_num_line = pdf_document::Line::default();
        page_num_line.words.push(
            _create_pdfword(
                "(26/04/25)".to_string(), 
                indentations.character, 
                Some(indentations.top))
        );
        page_num_line.words.push(
            _create_pdfword(pn.clone(), 
            (7.5*72.0) - (7.2 * pn.len() as f64), 
            Some(indentations.top))
        );

        new_page.lines.push(
            page_num_line
        );

        // Revised line
        let mut revised_line = pdf_document::Line::default();
        
        revised_line.words.push(_create_pdfword("revised_scn".to_string(), 
            indentations.action,
            None
        ));
        revised_line.words.push(_create_pdfword("*".to_string(), 
        (7.5*72.0)+(7.2*2.0), 
        None));
        new_page.lines.push(revised_line);

        
        //TODO: CONTINUED/MOREs
        // FIXME: How do we handle these?
        // They are part of the document content.
        // Also, we need to let the user pass in custom (MORE)/(CONTINUED) patterns
        // again, for non-english or non-standard support.
        new_page.lines.push(
            _create_pdfline_with_word("(MORE)".to_string(), 
            indentations.parenthetical, 
            Some(60.0))
        );
        
        // TODO: Scene heading elements
        let mut scene_heading_line = pdf_document::Line::default();
        scene_heading_line.words.push(
            _create_pdfword(
                "INT.".to_string(), indentations.action, 
                None)
        );
        let mut last_word: String = "INT.".to_string();
        let mut last_word_pos: f64 = scene_heading_line.words.last().unwrap().position.x;
        let mut _get_word_with_offset_from_previous = |text: String| {
            //println!("last_word: {}, len_in_pts: {:?}", last_word, last_word.len() as f64 * 7.2);
            let new_x_offset = (last_word.len() as f64 * 7.2) + 7.2
            + last_word_pos;
            
            //println!("offset x pos: {}", new_x_offset,);
            let new_word =_create_pdfword(
                text.clone(), 
                new_x_offset, 
                None);

            last_word = text.clone();
            last_word_pos = new_x_offset;
            return new_word;

        };

        let mut scene_heading_words = vec![
            "HOUSE",
            "-",
            "KITCHEN",
            "-",
            "DAY",
            "-",
            "CONTINUOUS"
        ];

        for word in scene_heading_words {
            scene_heading_line.words.push(
                _get_word_with_offset_from_previous(word.to_string())
            );
        }

        
        scene_heading_line.words.push(
            _create_pdfword(
                "*46G*".to_string(), indentations.right, None)
        );
        new_page.lines.push(scene_heading_line);

        // TODO: Add DEFAULT INDENTATIONS for A4
        // TODO: Test for A4 specifically 

        // maybe move the contents of this function to another function, which takes in the indentations 
        // as a parameter,
        // then call the new function with arbitrary indentation values
        
        // TODO: Revision LABEL (Blue:mm/dd/yyyy)
        
        // TODO: Title Page elements
        
        // TODO: Add TRANSITIONS??
        // Check if first word is x-position past like 3/4ths or 2/3rds of the page
        // if it's within the VCZ, and it's a farther-than-dialogue x-position, 
        //  AND it's the FIRST VALID TEXT ELEMENT, then it is likely a transition...
        // Transitions like CUT TO or FADE OUT or FADE TO BLACK are not handled currently at all...
        // Need to add to SP_TYPE enum...
        // --------------

        mock_pdf.pages.push(new_page);

        let parsed_doc = pdf_parser::get_screenplay_doc_from_pdf_obj(
            mock_pdf, 
            None,
            None,
            screenplay_document::TimeOfDay::default()
        ).unwrap();

        println!(
            "\n-----\n\nPage number: {:>8} | Rev. label/date(?): {:12} | {}\n", 
            format!("{:?}", parsed_doc.pages.first().unwrap().page_number),
            format!("{:?}",parsed_doc.pages.first().unwrap().revision_label),
            format!("{:?}",parsed_doc.pages.first().unwrap().revision_date),

        );

        let lines = &parsed_doc.pages.first().unwrap()
        .lines;

        // TODO: panic!() for each line type that doesn't fully pass
        // this means iterating manually... :<
        for line in lines {
            
            println!(
                "LT: {:-<70} \nScene Num: {:8} \nRevised: {}",
                
                
                if let Some(l_type) = line.line_type {
                    format!("{:?}",l_type).strip_prefix("SP_").unwrap().to_string()
                } else {
                    format!("{:?}",SPType::NONE)
                },
                if let Some(sc_num) = line.scene_number.clone() {
                    sc_num
                } else {
                    "None".to_string()
                }
                ,
                if line.revised {
                    "Y"
                } else {
                    "N"
                },
            );
            println!("{:^30}|{:^8}{:^8}|{:^8}", "Element",  "x","y", "Text");
            println!("{:-<58}", "  -");
            //println!("---");
            for el in &line.text_elements {
                println!("     {:24} | {:.2}, {:.2} | '{}'",
                    if let Some(l_type) = el.element_type.clone() {
                        format!("{:?}",l_type).strip_prefix("SP_").unwrap().to_string()
                    } else {
                        format!("{:?}",SPType::NONE)
                    },
                    el.element_position.unwrap().x,
                    el.element_position.unwrap().y,
                    el.text,
                );
            }
            println!("");


        }



    }
}

use std::fmt::Error;

use crate::pdf_document::PDFDocument;

pub mod pdf_document {


    // TODO: impl defaults for standard US-LETTER indentations
    #[derive(Default)]
    pub struct ElementIndentationsInches {
        pub pagewidth: f64,
        pub pageheight: f64,
        pub left: f64,
        pub right: f64,
        pub top: f64,
        pub bottom: f64,

        pub action: f64,
        pub character: f64,
        pub dialogue: f64,
        pub parenthetical: f64,
    }

    #[derive(Default)]
    pub struct ElementIndentationsPoints {
        pub pagewidth: f64,
        pub pageheight: f64,
        pub left: f64,
        pub right: f64,
        pub top: f64,
        pub bottom: f64,

        pub action: f64,
        pub character: f64,
        pub dialogue: f64,
        pub parenthetical: f64,
    }

    pub fn get_us_letter_default_indentations_inches() -> ElementIndentationsInches {
        ElementIndentationsInches {
            top: 10.0,
            bottom: 1.0,
            left: 1.5,
            right: 7.5,
            pageheight: 11.0,
            pagewidth: 8.5,
            action: 1.5,
            character: 3.7,
            dialogue:2.5,
            parenthetical: 3.1,
        }
    }

    pub fn get_indentations_inches_as_pts(indentations: &ElementIndentationsInches, resolution: &Option<f64>) -> ElementIndentationsPoints {
        let mut current_resolution: f64 = 72.0;
        if let Some(r) = resolution {
            current_resolution = r.clone()
        }
        
        ElementIndentationsPoints {
            top: indentations.top * current_resolution,
            bottom: indentations.bottom * current_resolution,
            pagewidth: indentations.pagewidth * current_resolution,
            pageheight: indentations.pageheight * current_resolution,
            left: indentations.left * current_resolution,
            right: indentations.right *current_resolution,
            action: indentations.action * current_resolution,
            character: indentations.character * current_resolution,
            dialogue: indentations.dialogue *current_resolution,
            parenthetical: indentations.parenthetical * current_resolution
        }
    }

    pub fn get_us_letter_default_indentation_pts() -> ElementIndentationsPoints {
        get_indentations_inches_as_pts(&get_us_letter_default_indentations_inches(), &None)
    }

    #[derive(Default, Clone, Copy)]
    pub struct TextPosition {
        pub x: f64,
        pub y: f64,
    }
    #[derive(Default)]
    pub struct PageSize {
        pub width: f64,
        pub height: f64,
    }
    #[derive(Default)]
    pub struct Word {
        pub text: String,
        pub text_bbox_width: f64,
        pub position:TextPosition,
        pub font_name: Option<String>,
        pub font_size: f64,
        pub font_character_width: f64,
    }
    #[derive(Default)]
    pub struct Line {
        pub words:Vec<Word>
    }
    #[derive(Default)]
    pub struct Page {
        pub lines: Vec<Line>,
        pub page_size: PageSize
    }
    #[derive(Default)]
    pub struct PDFDocument {
        pub pages: Vec<Page>,
        pub pdf_creator: Option<String>,
    }
}

pub mod screenplay_document {

    use std::{default, time::SystemTime};

    use super::pdf_document;

    pub enum PageFormat {
        US,
        A4,
        OTHER,
    }
    
    #[derive(Default, PartialEq, Clone, Copy, Debug)]
    #[allow(non_camel_case_types)]
    pub enum SPType {
        SP_ACTION = 0,

        SP_CHARACTER,
        SP_CHARACTER_EXTENSION, // require context to parse (previous word type)
        SP_DG_MORE_CONTINUED, // specifically has MORE or CONTINUED or CONT'D within parentheses
        SP_PARENTHETICAL,
        SP_DIALOGUE,
        SP_TRANSITION,

        // SCENE HEADING
        SP_SCENE_HEADING, // begins with INT. , EXT. , or I./E.
        SP_INT_EXT, //
        SP_LOCATION,
        SP_SCENE_HEADING_SUB_ELEMENT,
        SP_SCENE_HEADING_SEPARATOR, // Breaks up a slugline -- EXT. BASEBALL FIELD - PITCHER'S MOUND - PAST - NIGHT
        SP_SCENE_TIMEFRAME, // PAST, PRESENT, FUTURE, arbitrary timeframe "BEFORE DINNER", "AFTER THE EXPLOSION", etc.
        SP_SUBLOCATION,
        SP_TIME_OF_DAY,

        SP_SHOT_ANGLE, // SHOT or ANGLE on something, NOT a full scene heading / location

        SP_PAGENUM, // Nominal page number
        SP_SCENENUM, // Nominal scene number

        SP_PAGE_REVISION_LABEL, //may or may not include the date / color (I think it's two lines usually, but it could be one line potentially...?)
        SP_LINE_REVISION_MARKER, // asterisks in the left and/or right margins indicate a line or lines have been revised

        SP_MORE_CONTINUED,
        SP_FOOTER, // Not sure what footers are used for but....

        //DUAL DIALOGUE
        SP_DUAL_CHARACTERS,
        SP_DUAL_DIALOGUES,

        SP_DD_L_CHARACTER,
        SP_DD_L_CHARACTER_EXTENSION,
        SP_DD_L_PARENTHETICAL,
        SP_DD_L_DIALOGUE,
        SP_DD_L_MORE_CONTINUED,

        SP_DD_R_CHARACTER,
        SP_DD_R_CHARACTER_EXTENSION,
        SP_DD_R_PARENTHETICAL,
        SP_DD_R_DIALOGUE,
        SP_DD_R_MORE_CONTINUED,

        // TITLE PAGE
        TP_TITLE,
        TP_BYLINE,
        TP_AUTHOR,
        TP_DRAFT_DATE,
        TP_CONTACT,
        // -------------
        SP_OTHER,
        SP_BLANK, // BLANK element?
        SP_OMITTED,
        // Non- content text (asterisks and/or scene numbers in the margins, headers and footers, page numbers, etc.)
        NON_CONTENT_TOP,
        NON_CONTENT_BOTTOM,
        NON_CONTENT_LEFT,
        NON_CONTENT_RIGHT,

        #[default]
        NONE,
        _TYPECOUNT
    }

    #[derive(Default)]
    pub struct TextElement {
        pub text: String,
        pub element_type: Option<SPType>,
        pub preceding_whitespace_chars: u64,
        pub element_position: Option<pdf_document::TextPosition>,
    }

    #[derive(Default)]
    pub struct Line {
        pub text_elements: Vec<TextElement>,
        pub scene_number: Option<String>,
        pub line_type: Option<SPType>, // should default to NONE when initialized!!!
        pub preceding_empty_lines: u64,
        pub revised: bool,
        pub blank: bool,
    }


    #[derive(Default)]
    pub struct Page {
        pub lines: Vec<Line>,
        pub page_number: Option<String>,
        pub revised: bool,
        pub revision_label: Option<String>,
        pub revision_date: Option<String>,
        pub page_format: Option<PageFormat>,
    }

    #[derive(Default)]
    pub struct ScreenplayDocument {
        pub pages: Vec<Page>,
        pub revisions: Option<Vec<SystemTime>> // current (and possible previous) revision date(s) from the title page

    }

    

}

pub mod pdf_parser {
    //! This module is responsible for interpereting a (hopefully properlyformatted)
    //! PDF document into a usable, semantically-typed ScreenplayDocument structure.

    use std::ops::Not;

    use crate::pdf_document::get_indentations_inches_as_pts;
    use crate::pdf_document::get_us_letter_default_indentation_pts;
    use crate::pdf_document::ElementIndentationsInches;
    use crate::pdf_document::ElementIndentationsPoints;
    use crate::screenplay_document::SPType;

    use super::pdf_document;
    use super::screenplay_document;

    fn _is_int_ext_marker(text: &String, int_ext_markers: &Option<Vec<String>>) -> bool {
        if let None = int_ext_markers {
            if text.starts_with("INT.") | text.starts_with("EXT.") | text.starts_with("I./E.") {
                return true;
            }
            return false;
        }
        if let Some(int_ext_patterns) = int_ext_markers {
            for pat in int_ext_patterns {
                if text.starts_with(pat) {
                    return true;
                }
            }
        }
        return false;
    }

    fn _get_type_for_word(pdf_word: &pdf_document::Word,
    new_line: &screenplay_document::Line,
    previous_element_type: &SPType,
    element_indentaions_pts: &ElementIndentationsPoints) -> Option<SPType>{
        //TODO: FIXME: Calculate actual character width from font metrics...
        let char_width = pdf_word.font_size * 0.6; 
        let position_tolerance: f64 = 0.01;

        
        
        
        // check current line type ...
        if (new_line.line_type == Some(SPType::SP_SCENE_HEADING)) && (pdf_word.position.x >= element_indentaions_pts.right) {
            
            return Some(SPType::SP_SCENENUM);
        }
        // first pass of Word Type -- check previous element types first
        match previous_element_type {
            SPType::SP_PARENTHETICAL => Some(previous_element_type.clone()),
            SPType::SP_SUBLOCATION => {
                if pdf_word.text == "-".to_string() {
                    return Some(previous_element_type.clone());
                }
                return Some(SPType::SP_SUBLOCATION);
            },
            SPType::SP_LOCATION => {
                if pdf_word.text == "-".to_string() {
                    //panic!();
                    return Some(SPType::SP_LOCATION);
                }
                return Some(SPType::SP_SUBLOCATION);
                
            },
            SPType::SP_INT_EXT => {
                return Some(SPType::SP_LOCATION);
            },
            SPType::SP_CHARACTER => {
                if pdf_word.text.starts_with("(") {
                    return Some(SPType::SP_CHARACTER_EXTENSION);
                } else {
                    return Some(SPType::SP_CHARACTER);
                }
            },
            SPType::SP_DD_L_CHARACTER => {
                if pdf_word.text.starts_with("(")  {
                    return Some(SPType::SP_DD_L_CHARACTER_EXTENSION);
                } else {return Some(SPType::SP_DD_L_CHARACTER);}
            },
            SPType::SP_DD_R_CHARACTER => {
                if pdf_word.text.starts_with("(") {
                    return Some(SPType::SP_DD_R_CHARACTER_EXTENSION);
                } else {return Some(SPType::SP_DD_R_CHARACTER);}
            },
            SPType::SP_DD_L_CHARACTER_EXTENSION 
            | SPType::SP_DD_R_CHARACTER_EXTENSION 
            | SPType::SP_CHARACTER_EXTENSION => {
                return Some(previous_element_type.clone());
            },
            
            _ => {
                
                // Within Vertical Content Zone after this point
                
                println!("{}", pdf_word.position.y - element_indentaions_pts.top);
                if pdf_word.position.y < element_indentaions_pts.top 
                && pdf_word.position.y > element_indentaions_pts.bottom {
                    
                    //Check if it's a scene number
                    // TODO: This is a NAIVE implementation... probably need additional verification at some point...
                    
                    if pdf_word.position.x < element_indentaions_pts.left {
                        return Some(SPType::SP_SCENENUM);
                    } else if pdf_word.position.x >= element_indentaions_pts.right {
                        return Some(SPType::SP_SCENENUM);
                    } else {
                        
                        
                        let _within_tolerance  = |target| {
                            if (&pdf_word.position.x - &target).abs() > position_tolerance {
                                return false;
                            }
                            else {return true};
                        };
                        
                        //Within Vertical AND Horizontal Content Zone after this point
                        
                        //ACTION
                        if _within_tolerance(element_indentaions_pts.action){
                            //TODO: FIXME: Let user PASS IN INT_EXT PATTERNS (i.e. for non-english scripts)
                            
                            if _is_int_ext_marker(&pdf_word.text, &None){
                                return Some(SPType::SP_INT_EXT);
                            } else {
                                return Some(SPType::SP_ACTION);
                            };
                        }
                        if _within_tolerance(element_indentaions_pts.character) {
                            return Some(SPType::SP_CHARACTER);
                        }
                        else if _within_tolerance(element_indentaions_pts.dialogue) {
                            return Some(SPType::SP_DIALOGUE);
                        }
                        else if _within_tolerance(element_indentaions_pts.parenthetical) {
                            return Some(SPType::SP_PARENTHETICAL);
                        }
                        else {return None;}
                    };
                    
                }
                
                // Text is either ABOVE the top margin or BELOW the bottom margins...
                if pdf_word.text == "17A.".to_string() {println!("PAGENUMBER FOUND!----------------");}
                if pdf_word.position.y >= element_indentaions_pts.top {
                    let wordwidth: f64 = char_width * f64::from(pdf_word.text.len() as i32);
                    let rightedge: f64 = wordwidth + pdf_word.position.x;
                    if pdf_word.position.x < element_indentaions_pts.pagewidth / 3.0 {
                        return Some(SPType::NON_CONTENT_TOP);
                    } else if (rightedge - element_indentaions_pts.right) < position_tolerance
                    && (pdf_word.text.ends_with(".")) {
                        return Some(SPType::SP_PAGENUM);
                    }
                    else {
                        return Some(SPType::NON_CONTENT_TOP);
                    }
                }

                if pdf_word.text.contains("(MORE)") | pdf_word.text.contains("(CONTINUED)") | pdf_word.text.contains("(CONT'D)") {
                    return Some(SPType::SP_MORE_CONTINUED);
                }
                else {return Some(SPType::NON_CONTENT_BOTTOM);}
            },
        
        
        }
    }

    pub fn get_screenplay_doc_from_pdf_obj(doc: pdf_document::PDFDocument, 
    element_indentations: Option<ElementIndentationsInches>,
    revision_marker: Option<String>) -> Option<screenplay_document::ScreenplayDocument> {

        use screenplay_document::ScreenplayDocument;

        if doc.pages.len() < 1 {
            return None;
        }

        let mut r_marker = "*".to_string();
        if let Some(rm) = revision_marker{
            r_marker = rm;
        }

        let mut new_screenplay_doc: ScreenplayDocument = ScreenplayDocument::default();
        
        
        for pdf_page in doc.pages.iter(){
            if pdf_page.lines.len() < 1 {continue};

            let mut new_page = screenplay_document::Page::default();

            let mut prev_line_y_pos: f64 = 0.0;
            let mut line_height: f64 = 12.0; //This line height could be identified either here in-line or in
            // a pre-processing scan of the document
            // in-line might be better, to do it page-by-page as we go, rather than keep dictionaries/hashmaps


            //TODO: the current resolution and element_indentations_pts don't have to be defined here
            // in this for loop
            // UNLESS we need to set a different resolution or indentations for different pages
            // like a frankenscript from multiple writers
            // We should let the user pass in multiple ranges of indentations, optionally
            // but that's not necessary right now for basic functionality
            let mut current_resolution: f64 = 72.0;
            let mut element_indentaions_pts = ElementIndentationsPoints::default();
            if let Some(ref indentations) = element_indentations{
                element_indentaions_pts = get_indentations_inches_as_pts(indentations, &Some(current_resolution))
            } else {
                element_indentaions_pts = get_us_letter_default_indentation_pts();
            }
            for pdf_line in pdf_page.lines.iter() {
                if pdf_line.words.len() < 1 {continue};

                let mut new_line = screenplay_document::Line::default();
                let mut previous_element_type: SPType = SPType::NONE;
                let mut word_counter: usize = 0;
                for pdf_word in pdf_line.words.iter() {
                    //println!("Iterating over PDF WORDS!");
                    let mut new_text_element = screenplay_document::TextElement::default();
                    
                    let new_word_type: Option<SPType> = _get_type_for_word(&pdf_word, 
                    &new_line, 
                    &previous_element_type, 
                    &element_indentaions_pts);

                    println!("New type! {:?}", new_word_type);
                    new_text_element.element_position = Some(pdf_word.position.clone());

                    if let Some(nt) = new_word_type {
                        
                        match nt {
                            // Assign proper LINE TYPEs based on current WORD type
                            SPType::SP_DIALOGUE => {
                                new_line.line_type = Some(SPType::SP_DIALOGUE);
                            },
                            SPType::SP_PARENTHETICAL => {
                                new_line.line_type = Some(SPType::SP_PARENTHETICAL);
                            },
                            SPType::SP_DD_L_PARENTHETICAL
                            | SPType::SP_DD_R_PARENTHETICAL
                            | SPType::SP_DD_L_DIALOGUE
                            | SPType::SP_DD_R_DIALOGUE => {
                                new_line.line_type = Some(SPType::SP_DUAL_DIALOGUES);
                            },
                            SPType::SP_CHARACTER => {
                                new_line.line_type = Some(SPType::SP_CHARACTER);
                            },
                            SPType::SP_DD_L_CHARACTER 
                            | SPType::SP_DD_R_CHARACTER => {
                                new_line.line_type = Some(SPType::SP_DUAL_CHARACTERS);
                            },
                            SPType::SP_ACTION => {
                                if let Some(te) = new_line.text_elements.first() {
                                    if te.text == "revised_scn".to_string() {

                                        println!("COCK FUCK");
    
                                        panic!()
                                    }
                                }
                                new_line.line_type = Some(SPType::SP_ACTION);
                                
                            },
                            SPType::SP_INT_EXT => {
                                
                                new_line.line_type = Some(SPType::SP_SCENE_HEADING);
                            },

                            //SPECIAL CASES -- DON'T add the following as TEXT ELEMENTS later down

                            SPType::SP_PAGENUM => {
                                new_page.page_number = Some(pdf_word.text.clone());
                                continue;
                            },
                            SPType::SP_PAGE_REVISION_LABEL => {
                                // TODO: parse revision label for COLOR and DATE
                                // then ADD metadata to PAGE
                                new_page.revised = true;
                                continue;
                            },
                            SPType::NON_CONTENT_TOP 
                            | SPType::NON_CONTENT_BOTTOM
                            | SPType::NON_CONTENT_LEFT
                            | SPType::NON_CONTENT_RIGHT => {
                                //println!("Non-Content!!!!!");
                                //println!("Current action margin: {}", element_indentaions_pts.action);
                                //println!("{} | {}", new_text_element.element_position.unwrap().x, new_text_element.element_position.unwrap().y);
                                continue;
                            },
                            //FIXME: add separate case for LONE ASTERISK or REVISION MARKER,
                            // This is VERY fucking confusing otherwise...
                            SPType::SP_SCENENUM => {
                                //println!(" ---------SCENE NUMBER -------");
                                
                                if pdf_word.text.contains(&r_marker) { // TODO: pass in USER REVISION MARKERS
                                    new_line.revised = true;
                                }
                                let maybe_scene_num = Some(pdf_word.text.trim_matches('*')
                                .to_string()
                                .trim_matches('.')
                                .to_string());
                                if let Some(sn) = maybe_scene_num {
                                    if !sn.is_empty() {
                                        new_line.scene_number = Some(sn);
                                        new_line.line_type = Some(SPType::SP_SCENE_HEADING);
                                        previous_element_type = SPType::SP_SCENENUM;
                                    }
                                }
                                

                                continue; 
                            },
                            SPType::SP_LINE_REVISION_MARKER =>
                            {
                                new_line.revised = true;
                                previous_element_type = SPType::SP_LINE_REVISION_MARKER;
                                continue;
                            }

                            _ => {
    
                            }
                        }
                    }

                    new_text_element.element_type = new_word_type.clone();
                    new_text_element.text = pdf_word.text.clone();

                    // -------- WHITESPACING --------

                    // CALCULATE PRECEDING WHITESPACE CHARS, IF ANY

                    if word_counter > 0 {
                        if let Some(last_word)= pdf_line.words.last() {
                            let char_width: f64 = 7.2;
                            let whitespace_chars: u64 = u64::from(
                                ((pdf_word.position.x - (last_word.position.x + last_word.text_bbox_width))
                                / char_width)
                                .round() as u64
                            );

                            if whitespace_chars >= 1  {
                                match previous_element_type {
                                    SPType::SP_SCENENUM
                                    | SPType::SP_LINE_REVISION_MARKER => {
                                        new_text_element.preceding_whitespace_chars = 0;
                                    },
                                    _ => {
                                        new_text_element.preceding_whitespace_chars = whitespace_chars;
                                    }
                                }
                            } else {
                                println!("NEW TEXT ELEMENT OVERLAPS PREVIOUS ELEMENT! Assigned 1 unit of preceding whtiespace...");
                                new_text_element.preceding_whitespace_chars = 1
                            }
                            
                        };
                    }
                    if let Some(new_type) = new_word_type {
                        previous_element_type = new_type;
                        
                    }
                    else {
                        previous_element_type = SPType::NONE;
                    }
                    
                    new_line.text_elements.push(new_text_element);
                    //println!("Pushing new text element!");

                    word_counter += 1;
                    
                }
                if let Some(txt) = new_line.text_elements.first(){
                    if txt.text == "revised_scn".to_string() {
                            println!("LINE TYPE: ------- {:?}", new_line.line_type);
                            println!("ELEMENT TYPE: {:?}", txt.element_type);
                            //panic!();
                        }
                }
                //Add number of preceding blank lines to this line
                let cur_y_pos = pdf_line.words.first().unwrap().position.y;
                if prev_line_y_pos > 1.0 {
                    let y_delta = prev_line_y_pos - cur_y_pos;
                    if y_delta > line_height {
                        let blank_lines_count: u64 = (y_delta / line_height).ceil().round() as u64;
                        new_line.preceding_empty_lines = blank_lines_count;
                    }
                }

                prev_line_y_pos = cur_y_pos;
                if new_line.text_elements.is_empty(){
                    continue;
                }
                
                new_page.lines.push(new_line);
            }
            if new_page.lines.is_empty() {
                continue;
            }
            new_screenplay_doc.pages.push(new_page);

        }

        
        Some(new_screenplay_doc)
    }

}

pub fn get_screenplay_doc_from_file() -> Option<i32> {
    None
}

pub fn _get_screenplay_doc_from_pdfdoc_obj() -> Option<screenplay_document::ScreenplayDocument>{


    None
}

#[cfg(test)]
mod tests {
    use std::default;

    use crate::{pdf_document::{get_us_letter_default_indentation_pts, ElementIndentationsPoints, TextPosition}, screenplay_document::SPType};

    use super::*;

    fn _get_line_with_word(text: String, element_indentation: f64, y_height_inches: Option<f64>) -> pdf_document::Line {
        let mut new_word = pdf_document::Word::default();

        if let Some(inches) = y_height_inches {
            new_word = _get_test_pdfword(text, element_indentation, y_height_inches);
        }
        else {
            new_word = _get_test_pdfword(text, element_indentation, None);
        }

        let new_line:pdf_document::Line = pdf_document::Line { 
            words: vec![new_word] 
        };
        new_line
    }

    fn _get_test_pdfword(text: String, element_indentation: f64, y_height_inches: Option<f64>) -> pdf_document::Word {
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
        
        let action_word  = _get_test_pdfword(
            "Action!".to_string(), 72.0*1.5, None);
        let mut new_line: pdf_document::Line = pdf_document::Line::default();
        new_line.words.push(action_word);
        new_page.lines.push(new_line);
        mock_pdf.pages.push(new_page);
        //println!("Adding!...");
        let parse_result_doc = pdf_parser::get_screenplay_doc_from_pdf_obj(mock_pdf, 
        None,
        None);
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

        let indentations = get_us_letter_default_indentation_pts();


        println!(" ------ Testing Screenplay Element Types ------ ");
        println!("");

        let mut mock_pdf:pdf_document::PDFDocument = PDFDocument::default();
        let mut new_page = pdf_document::Page::default();
        
        new_page.lines.push(
            _get_line_with_word("Action!".to_string(), 
            indentations.action, 
            None)
        );
        new_page.lines.push(
            _get_line_with_word("CHARACTER".to_string(), 
            indentations.character, 
            None)
        );
        new_page.lines.push(
            _get_line_with_word("(wryly)".to_string(), 
            indentations.parenthetical, 
            None)
        );
        new_page.lines.push(
            _get_line_with_word("Dialogue".to_string(), 
            indentations.dialogue, 
            None)
        );

        let pn: String = "256ABC.".to_string();

        // Page Number
        // Rests at y-height of top margin
        // Is right-aligned to the right-hand margin
        new_page.lines.push(
            _get_line_with_word(pn.clone(), 
            (7.5*72.0) - (7.2 * pn.len() as f64), 
            Some(indentations.top))
        );

        // FIXME: ---------- This LINE is incorrectly parsed as SCENE_HEADING 
        // Action line with SCENE NUMBER
        let mut revised_line = pdf_document::Line::default();
        
        revised_line.words.push(_get_test_pdfword("revised_scn".to_string(), 
            indentations.action,
            None
        ));
        revised_line.words.push(_get_test_pdfword("*".to_string(), 
        (7.5*72.0)+(7.2*2.0), 
        None));
        new_page.lines.push(revised_line);

        
        //TODO: CONTINUED/MOREs
        // FIXME: How do we handle these?
        // They are part of the document content.
        // Also, we need to let the user pass in custom (MORE)/(CONTINUED) patterns
        // again, for non-english or non-standard support.
        new_page.lines.push(
            _get_line_with_word("(MORE)".to_string(), 
            indentations.parenthetical, 
            Some(60.0))
        );
        
        // TODO: Scene heading elements
        let mut scene_heading_line = pdf_document::Line::default();
        scene_heading_line.words.push(
            _get_test_pdfword(
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
            let new_word =_get_test_pdfword(
                text.clone(), 
                new_x_offset, 
                None);

            last_word = text.clone();
            last_word_pos = new_x_offset;
            return new_word;

        };

        scene_heading_line.words.push(
            _get_word_with_offset_from_previous("HOUSE".to_string(),)
        );
        scene_heading_line.words.push(
            _get_word_with_offset_from_previous("-".to_string(),)
        );
        scene_heading_line.words.push(
            _get_word_with_offset_from_previous("DAY".to_string(), )
        );
        scene_heading_line.words.push(
            _get_word_with_offset_from_previous("-".to_string(), )
        );
        scene_heading_line.words.push(
            _get_word_with_offset_from_previous("CONTINUOUS".to_string(), )
        );
        scene_heading_line.words.push(
            _get_test_pdfword(
                "*46G*".to_string(), indentations.right, None)
        );
        new_page.lines.push(scene_heading_line);
        
        //  TODO: Revision LABEL (Blue:mm/dd/yyyy)
        
        // TODO: Title Page elements
        
        // TODO: Add DEFAULT INDENTATIONS for A4
        // TODO: Test for A4 specifically 
        
        
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
            None
        ).unwrap();

        println!("\n-----\n\nPage number: {:?}\n", parsed_doc.pages.first().unwrap().page_number);

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

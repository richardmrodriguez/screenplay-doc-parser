//! This module is responsible for interpereting a (hopefully properlyformatted)
//! PDF document into a usable, semantically-typed ScreenplayDocument structure.

use core::num;
use core::time;
use std::ops::Not;

use uuid::Uuid;

use crate::pdf_document;
use crate::pdf_document::ElementIndentationsInches;
use crate::pdf_document::ElementIndentationsPoints;
use crate::screenplay_document::Environment;
use crate::screenplay_document::EnvironmentStrings;
use crate::screenplay_document::PageNumber;
use crate::screenplay_document::SPType;

use crate::screenplay_document;
use crate::screenplay_document::Scene;
use crate::screenplay_document::SceneHeadingElement;
use crate::screenplay_document::SceneNumber;
use crate::screenplay_document::ScreenplayCoordinate;
use crate::screenplay_document::TextElement;

fn _get_type_for_word(pdf_word: &pdf_document::Word,
    new_line: &screenplay_document::Line,
    element_indentaions_pts: &ElementIndentationsPoints,
    time_of_day_strs: &screenplay_document::TimeOfDayCollection,
    environment_strs: &screenplay_document::EnvironmentStrings
) -> Option<SPType>{
    
    use screenplay_document::SceneHeadingElement;
    use screenplay_document::SPType::*;

    let previous_element_type = match new_line.text_elements.last() {
        None => SPType::NONE,
        Some(e) => {

            match &e.element_type {
                None => SPType::NONE,
                Some(t) => t.clone()
            }
        }
    };

    //TODO: FIXME: Calculate actual character width from font metrics...
    let char_width = pdf_word.font_size * 0.6; 
    let position_tolerance: f64 = 0.01;

    
    match &new_line.line_type {
        None => {},
        Some(t) => {
            match t {
                SPType::SP_SCENE_HEADING(_) => {
                    if pdf_word.position.x >= element_indentaions_pts.right {
                        return Some(SPType::SP_SCENENUM);
                    }
                },

                _ => {}

            }

        }
    }
    
    // check current line type ...
    if (new_line.line_type == Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Line))) && (pdf_word.position.x >= element_indentaions_pts.right) {
        
        return Some(SPType::SP_SCENENUM);
    }



    // first pass of Word Type -- check previous element types first
    match previous_element_type {
        SP_SCENE_HEADING(SceneHeadingElement::TimeOfDay) => {
            if pdf_word.text == "-".to_string() {
                return Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Separator));
            }
            return None;
        }
        SPType::SP_SCENE_HEADING(SceneHeadingElement::Separator) => {
            let type_before_separator = match new_line.text_elements.get(new_line.text_elements.len() - 2) {
                None => SPType::NONE,
                Some(t) => match &t.element_type {
                    None => SPType::NONE,
                    Some(e) => e.clone()
                }

            };

            if time_of_day_strs.is_time_of_day(&pdf_word.text) {
                return Some(SP_SCENE_HEADING(SceneHeadingElement::TimeOfDay));
            }

            match type_before_separator {
                SPType::NONE => {return None} // ? Something has gone very wrong...
                SPType::SP_SCENE_HEADING(SceneHeadingElement::Location) => {
                    return Some(SP_SCENE_HEADING(SceneHeadingElement::SubLocation))
                }
                SPType::SP_SCENE_HEADING(SceneHeadingElement::Location) => {
                    if pdf_word.text == "-".to_string() {
                        return Some(SP_SCENE_HEADING(SceneHeadingElement::Separator));
                    }
                    return Some(SP_SCENE_HEADING(SceneHeadingElement::SubLocation))
                }
                SPType::SP_SCENE_HEADING(SceneHeadingElement::TimeOfDay) | SP_SCENE_HEADING(SceneHeadingElement::SlugOther) => {
                    return Some(SP_SCENE_HEADING(SceneHeadingElement::SlugOther))
                }
                _ => {
                    dbg!(&pdf_word.text);
                    dbg!(type_before_separator);
                    //panic!();
                    return Some(SP_SCENE_HEADING(SceneHeadingElement::SlugOther))
                }

            }
        }
        SPType::SP_PARENTHETICAL => {return Some(previous_element_type.clone())},
        SPType::SP_SCENE_HEADING(SceneHeadingElement::SubLocation) => {
            if pdf_word.text == "-".to_string() {
                return Some(SP_SCENE_HEADING(SceneHeadingElement::Separator));
            }
            return Some(SP_SCENE_HEADING(SceneHeadingElement::SubLocation));
        },
        SPType::SP_SCENE_HEADING(SceneHeadingElement::Location)=> {
            if pdf_word.text == "-".to_string() {
                return Some(SP_SCENE_HEADING(SceneHeadingElement::Separator));
            }
            return Some(SP_SCENE_HEADING(SceneHeadingElement::Location));
            
        },
        SP_SCENE_HEADING(SceneHeadingElement::Environment) => {
            return Some(SP_SCENE_HEADING(SceneHeadingElement::Location));
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
                        
                        if let Some(_) = Environment::from_str(&pdf_word.text, environment_strs) {
                            return Some(SP_SCENE_HEADING(SceneHeadingElement::Environment));
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
revision_marker: Option<String>,
time_of_day_strs: screenplay_document::TimeOfDayCollection,
environtment_strs: EnvironmentStrings) -> Option<screenplay_document::ScreenplayDocument> {

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
            element_indentaions_pts = ElementIndentationsPoints::from_inches(indentations, &Some(current_resolution));
        } else {
            element_indentaions_pts = ElementIndentationsPoints::us_letter_default(&Some(current_resolution));
        }
        for pdf_line in pdf_page.lines.iter() {
            if pdf_line.words.len() < 1 {continue};

            let mut line_revised: bool = false;
            let mut new_line = screenplay_document::Line::default();
            let mut previous_element_type: SPType = SPType::NONE;
            let mut word_counter: usize = 0;
            for pdf_word in pdf_line.words.iter() {
                //println!("Iterating over PDF WORDS!");
                let mut new_text_element = screenplay_document::TextElement::default();
                
                let new_word_type: Option<SPType> = _get_type_for_word(&pdf_word, 
                &new_line,
                &element_indentaions_pts,
                &time_of_day_strs,
                &environtment_strs);

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
                            new_line.line_type = Some(SPType::SP_ACTION);
                            
                        },
                        SPType::SP_SCENE_HEADING(SceneHeadingElement::Environment) => {
                            use screenplay_document::SceneHeadingElement;
                            new_line.line_type = Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Line));
                        },

                        //SPECIAL CASES -- DON'T add the following as TEXT ELEMENTS later down

                        SPType::SP_PAGENUM => {
                            new_page.page_number = Some(PageNumber(pdf_word.text.clone()));
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
                            
                            if pdf_word.text.contains(&r_marker) {
                                new_line.revised = true;
                            }
                            let maybe_scene_num = Some(pdf_word.text.trim_matches('*')
                            .to_string()
                            .trim_matches('.')
                            .to_string());
                            if let Some(sn) = maybe_scene_num {
                                if !sn.is_empty() {
                                    new_line.scene_number = Some(sn);
                                    use screenplay_document::SceneHeadingElement;
                                    new_line.line_type = Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Line));
                                    previous_element_type = SPType::SP_SCENENUM;
                                }
                            }
                            

                            continue; 
                        },
                        SPType::SP_LINE_REVISION_MARKER =>
                        {
                            new_line.revised = true;
                            line_revised = true;
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
                if let Some(new_type) = new_word_type.clone() {
                    previous_element_type = new_type;
                    
                }
                else {
                    previous_element_type = SPType::NONE;
                }
                
                new_line.text_elements.push(new_text_element);
                //println!("Pushing new text element!");

                word_counter += 1;
                
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

            //TODO: Create a new Scene struct --AFTER fixing the scene heading parsing...

            if let Some(last_line) = &new_page.lines.last() {

                match new_line.line_type {
                    None => {},
                    Some(SPType::SP_SCENE_HEADING(_)) => {
                        //panic!();
                        let new_scene = Scene {
                            number: {
                                if let Some(number) = &new_line.scene_number.clone(){
                                    Some(SceneNumber(number.clone()))
                                }
                                else {
                                    None
                                }
                            },
                            environment: Environment::from_str(
                                &new_line.text_elements
                                    .iter()
                                    .find(|te| te.element_type == Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Environment)))
                                    .unwrap()
                                    .text,
                                &environtment_strs
                            ).unwrap(),
                            start: ScreenplayCoordinate {
                                page: new_screenplay_doc.pages.len() as u64 + {if new_screenplay_doc.pages.len() > 0 {1} else {0}},
                                line: new_page.lines.len() as u64,
                                element: None
                            },
                            revised: new_line.revised,
                            story_location: screenplay_document::Location { 
                                elements: {
                                    
                                    new_line.text_elements
                                    .iter()
                                    .filter(
                                        |el| el.element_type == Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Location))
                                    )
                                    .map(|el| el.clone())
                                    .collect()
                                }, 
                                sublocations: None, // TODO: ????????
                                superlocation: None //TODO: ???????? what the fuck are these supposed to do...
                            },
                            story_sublocation: None, // could be multiple sublocations ...... ARGHHHHHH
                            story_time_of_day: {
                                let maybe_time: Vec<TextElement> = new_line.text_elements
                                .iter()
                                .filter(|el|el.element_type == Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::TimeOfDay)))
                                .map(|el| el.clone())
                                .collect();
                                match maybe_time.is_empty() {
                                    true => None,
                                    false => time_of_day_strs.get_time_of_day(&maybe_time.first().unwrap().text),
                                }
                            }
        
                            
                            
                        };
                        new_screenplay_doc.scenes.insert(Uuid::new_v4(), new_scene);
                    },
                    _ => {}
                }
                
                
                    
                
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

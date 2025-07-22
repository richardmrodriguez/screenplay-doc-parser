//! This module is responsible for interpereting a (hopefully properlyformatted)
//! PDF document into a usable, semantically-typed ScreenplayDocument structure.

use core::num;
use core::time;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Not;
use std::process::id;
use std::sync::RwLockReadGuard;
use std::thread::panicking;

use uuid::Uuid;

use crate::pdf_document;
use crate::pdf_document::ElementIndentationsInches;
use crate::pdf_document::ElementIndentationsPoints;
use crate::screenplay_document::Character;
use crate::screenplay_document::Environment;
use crate::screenplay_document::EnvironmentStrings;
use crate::screenplay_document::LocationID;
use crate::screenplay_document::LocationNode;
use crate::screenplay_document::PageNumber;
use crate::screenplay_document::SPType;

use crate::screenplay_document;
use crate::screenplay_document::Scene;
use crate::screenplay_document::SceneHeadingElement;
use crate::screenplay_document::SceneID;
use crate::screenplay_document::SceneNumber;
use crate::screenplay_document::ScreenplayCoordinate;
use crate::screenplay_document::TextElement;

pub mod indentations_deducer;

pub fn deduce_indentations(
    pdfdoc: &pdf_document::PDFDocument,
) -> Option<ElementIndentationsInches> {
    unimplemented!();
    let mut x_pos_vec: Vec<f64> = Vec::default();

    let mut lines_count = 0;
    for page in &pdfdoc.pages {
        for ln in &page.lines {
            if let Some(word) = ln.words.first() {
                x_pos_vec.push(word.position.x);
                lines_count += 1;
            }
        }
    }

    let mut x_freq_map: HashMap<i32, i32> = HashMap::new();

    for entry in x_pos_vec {
        let rounded = entry.round() as i32;
        let new_ent = x_freq_map.entry(rounded).or_insert(0);
        *new_ent += 1;
    }

    let mut x_freq_keys: Vec<i32> = x_freq_map.keys().cloned().collect();
    x_freq_keys.sort();

    for fk in &x_freq_keys {
        let v = x_freq_map.get(&fk);
        println!(
            "INDENT_INCHES: {:10.2} FREQUENCY: {:6.2}%",
            fk.clone() as f64 / 72.0,
            {
                if let Some(freq) = v {
                    let fr = *freq;
                    (fr as f64 / lines_count as f64) * 100.0
                } else {
                    0.0
                }
            }
        )
    }

    None
}

fn _is_word_within_content_zone(
    pdf_word: &pdf_document::Word,
    element_indentaions_pts: &ElementIndentationsPoints,
) -> bool {
    todo!()
}

fn _check_non_content_type(
    pdf_word: &pdf_document::Word,
    new_line: &screenplay_document::Line,
    element_indentaions_pts: &ElementIndentationsPoints,
    time_of_day_strs: &screenplay_document::TimeOfDayCollection,
    environment_strs: &screenplay_document::EnvironmentStrings,
    r_marker: &String,
) -> Option<SPType> {
    todo!()
}

fn _get_type_for_word(
    pdf_word: &pdf_document::Word,
    new_line: &screenplay_document::Line,
    element_indentaions_pts: &ElementIndentationsPoints,
    time_of_day_strs: &screenplay_document::TimeOfDayCollection,
    environment_strs: &screenplay_document::EnvironmentStrings,
    r_marker: &String,
) -> Option<SPType> {
    use screenplay_document::SPType::*;
    use screenplay_document::SceneHeadingElement;

    let previous_element_type = match new_line.text_elements.last() {
        None => SPType::NONE,
        Some(e) => match &e.element_type {
            None => SPType::NONE,
            Some(t) => t.clone(),
        },
    };

    //TODO: FIXME: Calculate actual character width from font metrics...
    let char_width = pdf_word.font_size * 0.6; // should be ~7.2 for 12-point font
    let position_tolerance: f64 = 0.01;

    // check current line type ...
    if (new_line.line_type == Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Line)))
        && (pdf_word.position.x >= element_indentaions_pts.right)
    {
        if pdf_word.text == *r_marker {
            return Some(SPType::SP_LINE_REVISION_MARKER);
        }
        return Some(SPType::SP_SCENENUM);
    }

    // first pass of Word Type -- check previous element types first
    match previous_element_type {
        // if previous type was "content" types...
        SPType::SP_SCENE_HEADING(SceneHeadingElement::TimeOfDay) => {
            if pdf_word.text == "-".to_string() {
                return Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Separator));
            }
            return None;
        }
        SPType::SP_SCENE_HEADING(SceneHeadingElement::Separator) => {
            let type_before_separator =
                match new_line.text_elements.get(new_line.text_elements.len() - 2) {
                    None => SPType::NONE,
                    Some(t) => match &t.element_type {
                        None => SPType::NONE,
                        Some(e) => e.clone(),
                    },
                };

            if time_of_day_strs.is_time_of_day(&pdf_word.text) {
                return Some(SP_SCENE_HEADING(SceneHeadingElement::TimeOfDay));
            }

            match type_before_separator {
                SPType::NONE => return None, // ? Something has gone very wrong...
                SPType::SP_SCENE_HEADING(SceneHeadingElement::Location) => {
                    return Some(SP_SCENE_HEADING(SceneHeadingElement::SubLocation));
                }
                SPType::SP_SCENE_HEADING(SceneHeadingElement::Location) => {
                    if pdf_word.text == "-".to_string() {
                        return Some(SP_SCENE_HEADING(SceneHeadingElement::Separator));
                    }
                    return Some(SP_SCENE_HEADING(SceneHeadingElement::SubLocation));
                }
                SPType::SP_SCENE_HEADING(SceneHeadingElement::TimeOfDay)
                | SP_SCENE_HEADING(SceneHeadingElement::SlugOther) => {
                    return Some(SP_SCENE_HEADING(SceneHeadingElement::SlugOther));
                }
                _ => {
                    dbg!(&pdf_word.text);
                    dbg!(type_before_separator);
                    //panic!();
                    return Some(SP_SCENE_HEADING(SceneHeadingElement::SlugOther));
                }
            }
        }
        SPType::SP_PARENTHETICAL => return Some(previous_element_type.clone()),
        SPType::SP_SCENE_HEADING(SceneHeadingElement::SubLocation) => {
            if pdf_word.text == "-".to_string() {
                return Some(SP_SCENE_HEADING(SceneHeadingElement::Separator));
            }
            return Some(SP_SCENE_HEADING(SceneHeadingElement::SubLocation));
        }
        SPType::SP_SCENE_HEADING(SceneHeadingElement::Location) => {
            if pdf_word.text == "-".to_string() {
                return Some(SP_SCENE_HEADING(SceneHeadingElement::Separator));
            }
            return Some(SP_SCENE_HEADING(SceneHeadingElement::Location));
        }
        SPType::SP_SCENE_HEADING(SceneHeadingElement::Environment) => {
            return Some(SP_SCENE_HEADING(SceneHeadingElement::Location));
        }
        SPType::SP_CHARACTER => {
            // TODO:
            // Create a function that checks all the "non-content-types"
            // and returns that as an optional SPType
            // then just match against that for every content type
            // if it's non-content, then return that type
            // else if it's content, then handle that within this block
            if pdf_word.text.starts_with("(") {
                return Some(SPType::SP_CHARACTER_EXTENSION);
            } else {
                if pdf_word.text == *r_marker {
                    return Some(SPType::SP_LINE_REVISION_MARKER);
                }
                return Some(SPType::SP_CHARACTER);
            }
        }
        SPType::SP_DD_L_CHARACTER => {
            if pdf_word.text.starts_with("(") {
                return Some(SPType::SP_DD_L_CHARACTER_EXTENSION);
            } else {
                return Some(SPType::SP_DD_L_CHARACTER);
            }
        }
        SPType::SP_DD_R_CHARACTER => {
            if pdf_word.text.starts_with("(") {
                return Some(SPType::SP_DD_R_CHARACTER_EXTENSION);
            } else {
                return Some(SPType::SP_DD_R_CHARACTER);
            }
        }
        SPType::SP_DD_L_CHARACTER_EXTENSION
        | SPType::SP_DD_R_CHARACTER_EXTENSION
        | SPType::SP_CHARACTER_EXTENSION => {
            return Some(previous_element_type.clone());
        }

        _ => {
            // ------------- INDENTATION PARSING --------------------------

            // Within Vertical Content Zone after this point

            //println!("{}", pdf_word.position.y - element_indentaions_pts.top);
            if pdf_word.position.y < element_indentaions_pts.top
                && pdf_word.position.y > element_indentaions_pts.bottom
            {
                //Check if it's a scene number
                // TODO: This is a NAIVE implementation... probably need additional verification at some point...

                if pdf_word.position.x < element_indentaions_pts.left {
                    return Some(SPType::SP_SCENENUM);
                }
                /*
                 */
                else if pdf_word.position.x >= element_indentaions_pts.right {
                    if pdf_word.text == *r_marker {
                        return Some(SPType::SP_LINE_REVISION_MARKER);
                    }
                    return Some(SPType::SP_SCENENUM);
                } else {
                    let _within_tolerance = |target| {
                        if (&pdf_word.position.x - &target).abs() > position_tolerance {
                            return false;
                        } else {
                            return true;
                        };
                    };

                    //Within Vertical AND Horizontal Content Zone after this point

                    //ACTION
                    if _within_tolerance(element_indentaions_pts.action) {
                        //TODO: FIXME: Let user PASS IN INT_EXT PATTERNS (i.e. for non-english scripts)

                        if let Some(_) = Environment::from_str(&pdf_word.text, environment_strs) {
                            return Some(SP_SCENE_HEADING(SceneHeadingElement::Environment));
                        } else if new_line.line_type == None {
                            return Some(SPType::SP_ACTION);
                        };
                    }
                    if _within_tolerance(element_indentaions_pts.character)
                        && new_line.line_type == None
                    {
                        return Some(SPType::SP_CHARACTER);
                    } else if _within_tolerance(element_indentaions_pts.dialogue)
                        && new_line.line_type == None
                    {
                        return Some(SPType::SP_DIALOGUE);
                    } else if _within_tolerance(element_indentaions_pts.parenthetical)
                        && pdf_word.text.starts_with("(")
                        && new_line.line_type == None
                    {
                        return Some(SPType::SP_PARENTHETICAL);
                    } else {
                        return None;
                    }
                };
            }

            // Text is either ABOVE the top margin or BELOW the bottom margins...
            // pdf_word.text == "17A.".to_string() {println!("PAGENUMBER FOUND!----------------");}
            if pdf_word.position.y >= element_indentaions_pts.top {
                let wordwidth: f64 = char_width * f64::from(pdf_word.text.len() as i32);
                let rightedge: f64 = wordwidth + pdf_word.position.x;
                if pdf_word.position.x < element_indentaions_pts.pagewidth / 3.0 {
                    return Some(SPType::NON_CONTENT_TOP);
                } else if (element_indentaions_pts.pagewidth - pdf_word.position.x)
                    < (element_indentaions_pts.pagewidth / 4.0)
                    && (pdf_word.text.ends_with("."))
                {
                    return Some(SPType::SP_PAGENUM);
                } else {
                    return Some(SPType::NON_CONTENT_TOP);
                }
            }
            // TODO: let user pass in MORE and CONTINUED strings as a struct
            if pdf_word.text.contains("(MORE)")
                | pdf_word.text.contains("(CONTINUED)")
                | pdf_word.text.contains("(CONT'D)")
            {
                return Some(SPType::SP_MORE_CONTINUED);
            } else {
                return Some(SPType::NON_CONTENT_BOTTOM);
            }
        }
    }
}

pub fn get_screenplay_doc_from_pdf_obj(
    doc: pdf_document::PDFDocument,
    element_indent_in_opt: Option<ElementIndentationsInches>,
    rev_marker_opt: Option<String>,
    time_of_day_strs_opt: Option<screenplay_document::TimeOfDayCollection>,
    env_strs_opt: Option<EnvironmentStrings>,
) -> Option<screenplay_document::ScreenplayDocument> {
    use screenplay_document::ScreenplayDocument;

    if doc.pages.len() < 1 {
        return None;
    }

    let time_of_day_strs: screenplay_document::TimeOfDayCollection;
    if let Some(tds) = time_of_day_strs_opt {
        time_of_day_strs = tds;
    } else {
        time_of_day_strs = screenplay_document::TimeOfDayCollection::default();
    }

    let environment_strs: screenplay_document::EnvironmentStrings;
    if let Some(evs) = env_strs_opt {
        environment_strs = evs;
    } else {
        environment_strs = EnvironmentStrings::default();
    }

    let r_marker;
    if let Some(rm) = rev_marker_opt {
        r_marker = rm;
    } else {
        r_marker = "*".to_string();
    }

    let mut new_screenplay_doc: ScreenplayDocument = ScreenplayDocument::default();

    for pdf_page in doc.pages.iter() {
        if pdf_page.lines.len() < 1 {
            continue;
        };
        // TODO: abstract out the "line handling" logic into "fn get_line()"??

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
        let element_indentaions_pts;
        if let Some(ref indentations) = element_indent_in_opt {
            element_indentaions_pts =
                ElementIndentationsPoints::from_inches(indentations, &Some(current_resolution));
        } else {
            element_indentaions_pts =
                ElementIndentationsPoints::us_letter_default(&Some(current_resolution));
        }
        for (pdf_l_idx, pdf_line) in pdf_page.lines.iter().enumerate() {
            if pdf_line.words.len() < 1 {
                continue;
            };

            let mut new_line = screenplay_document::Line::default();
            let mut previous_element_type: SPType = SPType::NONE;
            let mut word_counter: usize = 0;
            for pdf_word in pdf_line.words.iter() {
                //println!("Iterating over PDF WORDS!");
                let mut new_text_element = screenplay_document::TextElement::default();

                let new_word_type: Option<SPType> = _get_type_for_word(
                    &pdf_word,
                    &new_line,
                    &element_indentaions_pts,
                    &time_of_day_strs,
                    &environment_strs,
                    &r_marker,
                );

                //println!("New type! {:?}", new_word_type);
                new_text_element.element_position = Some(pdf_word.position.clone());

                if let Some(nwt) = new_word_type {
                    match nwt {
                        // Assign proper LINE TYPEs based on current WORD type
                        SPType::SP_DIALOGUE => {
                            if new_line.line_type == None {
                                new_line.line_type = Some(SPType::SP_DIALOGUE);
                            }
                        }
                        SPType::SP_PARENTHETICAL => {
                            if new_line.line_type == None {
                                new_line.line_type = Some(SPType::SP_PARENTHETICAL);
                            }
                        }
                        SPType::SP_DD_L_PARENTHETICAL
                        | SPType::SP_DD_R_PARENTHETICAL
                        | SPType::SP_DD_L_DIALOGUE
                        | SPType::SP_DD_R_DIALOGUE => {
                            new_line.line_type = Some(SPType::SP_DUAL_DIALOGUES);
                        }
                        SPType::SP_CHARACTER => {
                            if new_line.line_type == None {
                                new_line.line_type = Some(SPType::SP_CHARACTER);
                            }
                        }
                        SPType::SP_DD_L_CHARACTER | SPType::SP_DD_R_CHARACTER => {
                            if new_line.line_type == None {
                                new_line.line_type = Some(SPType::SP_DUAL_CHARACTERS);
                            }
                        }
                        SPType::SP_ACTION => {
                            if new_line.line_type == None {
                                new_line.line_type = Some(SPType::SP_ACTION);
                            }
                        }
                        SPType::SP_SCENE_HEADING(SceneHeadingElement::Environment) => {
                            use screenplay_document::SceneHeadingElement;
                            if new_line.line_type == None {
                                new_line.line_type =
                                    Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Line));
                            }
                        }

                        //SPECIAL CASES -- still add these elements to the line, but
                        // ALSO update the relevant metadata
                        // any element needs to still be available in the screenplay document, so we can
                        // re-assign it after parsing, if necessary
                        SPType::SP_PAGENUM => {
                            if new_line.line_type == None {
                                new_line.line_type = Some(SPType::SP_PAGE_HEADER);
                            }
                            new_page.page_number = Some(PageNumber(pdf_word.text.clone()));
                            //continue;
                        }
                        SPType::SP_PAGE_REVISION_LABEL => {
                            // TODO: parse revision label for COLOR and DATE
                            // then ADD metadata to PAGE
                            new_page.revised = true;
                            //continue;
                        }
                        SPType::NON_CONTENT_TOP
                        | SPType::NON_CONTENT_BOTTOM
                        | SPType::NON_CONTENT_LEFT
                        | SPType::NON_CONTENT_RIGHT => {
                            //println!("Non-Content!!!!!");
                            //println!("Current action margin: {}", element_indentaions_pts.action);
                            //println!("{} | {}", new_text_element.element_position.unwrap().x, new_text_element.element_position.unwrap().y);
                            //continue;
                        }
                        SPType::SP_SCENENUM => {
                            //println!(" ---------SCENE NUMBER -------");

                            if pdf_word.text.contains(&r_marker) {
                                new_line.revised = true;
                            }
                            let maybe_scene_num = Some(
                                pdf_word
                                    .text
                                    .trim_matches('*') //FIXME: This DOESN'T trim out the arbitrary user-defined revision marker! only asterisk! Fix this!
                                    .to_string()
                                    .trim_matches('.')
                                    .to_string(),
                            );
                            if let Some(sn) = maybe_scene_num {
                                if !sn.is_empty() {
                                    new_line.scene_number = Some(sn);
                                    use screenplay_document::SceneHeadingElement;
                                    match new_line.line_type {
                                        Some(SPType::NONE) => {
                                            new_line.line_type = Some(SPType::SP_SCENE_HEADING(
                                                SceneHeadingElement::Line,
                                            ));
                                        }
                                        _ => {}
                                    }
                                    previous_element_type = SPType::SP_SCENENUM;
                                }
                            }

                            //continue;
                        }
                        SPType::SP_LINE_REVISION_MARKER => {
                            new_line.revised = true;
                            previous_element_type = SPType::SP_LINE_REVISION_MARKER;
                            //continue;
                        }

                        _ => {}
                    }
                }

                new_text_element.element_type = new_word_type.clone();
                new_text_element.text = pdf_word.text.clone();

                // -------- WHITESPACING --------

                // CALCULATE PRECEDING WHITESPACE CHARS, IF ANY

                if word_counter > 0 {
                    if let Some(last_word) = pdf_line.words.last() {
                        let char_width: f64 = 7.2;
                        let whitespace_chars: u64 = u64::from(
                            ((pdf_word.position.x - (last_word.position.x + last_word.bbox_width))
                                / char_width)
                                .round() as u64,
                        );

                        if whitespace_chars >= 1 {
                            match previous_element_type {
                                SPType::SP_SCENENUM | SPType::SP_LINE_REVISION_MARKER => {
                                    new_text_element.preceding_whitespace_chars = 0;
                                }
                                _ => {
                                    new_text_element.preceding_whitespace_chars = whitespace_chars;
                                }
                            }
                        } else {
                            //FIXME: WTF does this whole if block even do???? Why does this print so often???
                            //println!("NEW TEXT ELEMENT OVERLAPS PREVIOUS ELEMENT! Assigned 1 unit of preceding whtiespace...");
                            new_text_element.preceding_whitespace_chars = 1
                        }
                    };
                }
                if let Some(new_type) = new_word_type.clone() {
                    previous_element_type = new_type;
                } else {
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
            if new_line.text_elements.is_empty() {
                continue;
            }

            match new_line.line_type {
                None => {}
                // CHARACTER PARSING
                Some(SPType::SP_CHARACTER) => {
                    let mut character_name = String::new();
                    for element in &new_line.text_elements {
                        if element.element_type == Some(SPType::SP_CHARACTER) {
                            if !character_name.is_empty() {
                                character_name.push(' ');
                            }
                            character_name.push_str(&element.text);
                        }
                    }
                    //panic!();
                    
                    let mut character_exists_in_doc = false;
                    if !&new_screenplay_doc.characters.is_empty() {
                        for character in &new_screenplay_doc.characters {
                            if character.name == character_name {
                                character_exists_in_doc = true;
                            }
                        }
                    }
                    if !character_exists_in_doc{
                        
                        let new_character = Character {
                            name: character_name,
                            id: screenplay_document::CharacterID::new(),
                        };
                        new_screenplay_doc.characters.insert(new_character);
                    }
                }
                // SCENE / LOCATION PARSING
                Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Line)) => {
                    // Environment Parsing
                    let maybe_first_word = &new_line
                        .text_elements
                        .iter()
                        .filter(|te| {
                            te.element_type
                                == Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Environment))
                        })
                        .take(1)
                        .next();

                    let mut new_line_env = Environment::Ext;

                    if let Some(fw) = maybe_first_word {
                        new_line_env = Environment::from_str(&fw.text, &environment_strs).unwrap();
                    }

                    // Location Parsing

                    let mut root_location_string: String = String::new();

                    let mut current_sub_location_string = String::new();
                    let mut full_path: Vec<String> = Vec::new();
                    let mut root_location_done = false;

                    for element in &new_line.text_elements {
                        match element.element_type {
                            Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Location))
                            | Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Environment)) => {
                                if !root_location_string.is_empty() {
                                    root_location_string.push(' ');
                                }
                                root_location_string.push_str(&element.text.clone());
                            }
                            Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Separator)) => {
                                if root_location_done {
                                    if !current_sub_location_string.is_empty() {
                                        full_path.push(current_sub_location_string.clone());
                                        current_sub_location_string = String::new();
                                    }
                                } else {
                                    root_location_done = true;
                                    full_path.push(root_location_string.clone());
                                }
                            }
                            Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::SubLocation)) => {
                                if !current_sub_location_string.is_empty() {
                                    current_sub_location_string.push(' ');
                                }
                                current_sub_location_string.push_str(&element.text.clone());
                            }
                            _ => {
                                if !root_location_string.is_empty() {
                                    break;
                                }
                            }
                        }
                    }

                    let mut location_id_to_insert: Option<LocationID> = None;
                    let mut exists: bool = false;

                    for (existing_id, existing_root) in &new_screenplay_doc.locations {
                        if existing_root.superlocation.is_some() {
                            continue;
                        }
                        if root_location_string == existing_root.string {
                            location_id_to_insert = Some(existing_id.clone());
                            exists = true;
                        }
                    }

                    if !exists {
                        location_id_to_insert = Some(LocationID::new());

                        let new_root_location: LocationNode = LocationNode {
                            string: root_location_string.clone(),
                            sublocations: HashSet::new(),
                            superlocation: None,
                        };
                        new_screenplay_doc
                            .locations
                            .insert(location_id_to_insert.clone().unwrap(), new_root_location);
                    }

                    // Subpath parsing and insertion

                    if let Some((id, s_path)) =
                        crate::reports::location_path_exists(&new_screenplay_doc, &full_path)
                        
                    {
                        let mut current_id = id.clone();

                        for pathstring in s_path {
                            if let Some(location) =
                                &mut new_screenplay_doc.locations.get_mut(&current_id)
                            {
                                let new_id = LocationID::new();

                                let new_location = LocationNode {
                                    string: pathstring.clone(),
                                    sublocations: HashSet::new(),
                                    superlocation: Some(current_id.clone()),
                                };
                                current_id = new_id.clone();
                                location_id_to_insert = Some(new_id.clone());
                                location.add_sublocation(new_id.clone());
                                new_screenplay_doc
                                    .locations
                                    .insert(new_id.clone(), new_location);
                            }
                        }
                    }

                    // Scene Insertion
                    let new_scene = Scene {
                        number: {
                            if let Some(num) = &new_line.scene_number.clone() {
                                Some(SceneNumber(num.clone()))
                            } else {
                                None
                            }
                        },
                        environment: new_line_env,
                        start: ScreenplayCoordinate {
                            page: new_screenplay_doc.pages.len(),
                            line: new_page.lines.len(),
                            element: None,
                        },
                        revised: new_line.revised,
                        story_locations: {
                            if let Some(id) = location_id_to_insert {
                                vec![id.clone()]
                            } else {
                                Vec::new()
                            }
                        },
                        story_time_of_day: {
                            let maybe_time: Vec<TextElement> = new_line
                                .text_elements
                                .iter()
                                .filter(|el| {
                                    el.element_type
                                        == Some(SPType::SP_SCENE_HEADING(
                                            SceneHeadingElement::TimeOfDay,
                                        ))
                                })
                                .map(|el| el.clone())
                                .collect();
                            match maybe_time.is_empty() {
                                true => None,
                                false => time_of_day_strs
                                    .get_time_of_day(&maybe_time.first().unwrap().text),
                            }
                        },
                    };
                    let new_scene_id = SceneID::new();
                    new_line.scene_id = Some(new_scene_id.clone());
                    new_screenplay_doc.scenes.insert(new_scene_id, new_scene);
                }
                _ => {}
            }

            // line number fixing
            if let Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Line)) = new_line.line_type {
                for te in &mut new_line.text_elements {
                    if te.element_type == None
                        && te.text == new_line.scene_number.clone().unwrap_or("_N?N_".to_string())
                    {
                        te.element_type = Some(SPType::SP_SCENENUM);
                        println!("{:?}", te.element_type);
                    }
                }
            } else {
                // Text Element Fixing -- overwrite previous NONE-TYPED elements to the LINE TYPE
                // if it's content
                // else leave it alone
                let mut last_element_type: &Option<SPType> = &None;
                for te in &mut new_line.text_elements {
                    if te.element_type == None {
                        match new_line.line_type {
                            Some(SPType::SP_ACTION) => {
                                te.element_type = Some(SPType::SP_ACTION);
                            }
                            Some(SPType::SP_CHARACTER) => match last_element_type {
                                Some(SPType::SP_CHARACTER_EXTENSION) => {
                                    te.element_type = Some(SPType::SP_CHARACTER_EXTENSION);
                                }
                                _ => {
                                    te.element_type = Some(SPType::SP_CHARACTER);
                                }
                            },
                            Some(SPType::SP_DIALOGUE) => {
                                te.element_type = Some(SPType::SP_DIALOGUE);
                            }

                            _ => {}
                        }
                    }

                    if new_line.line_type == Some(SPType::SP_PAGE_HEADER)
                        && te.element_type == Some(SPType::NON_CONTENT_TOP)
                    {
                        te.element_type = Some(SPType::SP_PAGE_REVISION_LABEL);
                    }
                    last_element_type = &te.element_type
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

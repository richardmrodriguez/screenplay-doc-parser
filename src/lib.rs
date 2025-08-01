use std::fmt::Error;

pub mod pdf_document;
pub mod reports;
pub mod screenplay_document;

pub mod pdf_parser;

#[cfg(feature = "mupdf-basic-parsing")]
pub mod mupdf_basic_parser;

#[cfg(test)]
mod tests {

    use crate::{
        pdf_document::{ElementIndentationsPoints, PDFDocument, TextPosition},
        pdf_parser::deduce_indentations,
        screenplay_document::{EnvironmentStrings, SPType, TimeOfDayCollection},
    };

    use super::*;

    fn _create_pdfline_with_word(
        text: String,
        element_indentation: f64,
        y_height_inches: Option<f64>,
    ) -> pdf_document::Line {
        let mut new_word = pdf_document::Word::default();

        if let Some(inches) = y_height_inches {
            new_word = _create_pdfword(text, element_indentation, y_height_inches);
        } else {
            new_word = _create_pdfword(text, element_indentation, None);
        }

        let new_line: pdf_document::Line = pdf_document::Line {
            words: vec![new_word],
        };
        new_line
    }

    fn _create_pdfword(
        text: String,
        element_indentation: f64,
        y_height_inches: Option<f64>,
    ) -> pdf_document::Word {
        let mut y_height_pts = 0.0;
        if let Some(inches) = y_height_inches {
            y_height_pts = 72.0 * inches;
        } else {
            y_height_pts = 3.0 * 72.0;
        }

        let new_word: pdf_document::Word = pdf_document::Word {
            text: text.clone(),
            bbox_width: text.len() as f64 * 7.2 as f64,
            bbox_height: 0.0,
            position: TextPosition {
                x: element_indentation,
                y: y_height_pts,
            },
            font_name: None,
            font_size: 12.0,
            font_character_width: 7.2,
        };
        new_word
    }

    #[test]
    fn indent_deduction() {
        let doc_result =
            mupdf_basic_parser::get_pdf_obj_from_filepath("test_data/VCR2L.pdf".to_string());
        if let Ok(doc) = doc_result {
            let indentations_opt = deduce_indentations(&doc);
        }
    }

    #[cfg(feature = "mupdf-basic-parsing")]
    #[test]
    fn test_mupdf_parsing() {
        let start = Instant::now();
        use crate::reports;
        use mupdf_basic_parser;
        use std::time::Instant;

        let custom_indentations = ElementIndentationsInches::us_letter_default();
        use crate::{
            pdf_document::ElementIndentationsInches, screenplay_document::ScreenplayDocument,
        };
        let screenplay_result = mupdf_basic_parser::get_screenplay_doc_from_filepath(
            //"test_data/DraftTest_02.pdf".into(),
            "test_data/VCR2L.pdf".into(),
            Some(custom_indentations),
            None,
            None,
            None,
        );
        let Ok(screenplay) = screenplay_result else {
            println!("{:#?}", screenplay_result);
            panic!();
        };

        //println!("PAGESLEN: {:?}", &screenplay.pages.len());
        //for page in &screenplay.pages {
        //    println!("     PAGE: {:?} | LINES: {:?}", page.page_number, page.lines.len())
        //}

        println!("ALL SCENES:");
        let scenes_opt = reports::get_all_scenes_ordered(&screenplay);
        if let Some(scenes) = scenes_opt {
            for (scn, scene_obj) in scenes {
                let Some(leaf_location) = &scene_obj.story_locations.last() else {
                    continue;
                };
                println!(
                    "SCENE: P:{:<3?}, L:{:<3?}  | LOCATION: {:?}",
                    scene_obj.start.page,
                    scene_obj.start.line,
                    reports::get_full_string_for_location_path(&screenplay, &leaf_location)
                );
            }
            println!("");
        }
        println!("\n- SCENES PER PAGE:");
        for (p_idx, page) in screenplay.pages.iter().enumerate() {
            let Some(scenes_for_page) =
                reports::get_all_scenes_on_page_by_index(&screenplay, p_idx)
            else {
                continue;
            };
            println!("--- PAGE: {:?}", p_idx);
            for (id, scn) in scenes_for_page {
                for loc in &scn.story_locations {
                    println!(
                        "----- {:?}",
                        reports::get_full_string_for_location_path(&screenplay, &loc)
                    )
                }
            }
        }

        // Test Get LOCATIONS...

        println!("\nALL LOCATIONS:");
        for (id, location) in &screenplay.locations {
            println!("LOCATION_ID: {:?}, | LOCATION: {:}", id, location.string);
        }

        println!("\n-----------LOCATION_HEIRARCHY----------");

        for (id, location) in &screenplay.locations {
            if location.superlocation.is_none() {
                println!("\nLOCATION_ID: {:?}, | LOCATION: {:}", id, location.string);
                let Some(leafs) = reports::get_all_location_leafs(&screenplay, id) else {
                    panic!();
                };
                println!("------- LEAFS FOR ROOT");
                for id in leafs {
                    let Some(leaf) = screenplay.locations.get(id) else {
                        continue;
                    };
                    println!(" ------- : | {:?}", leaf.string);
                }
            }
        }

        for (id, location) in &screenplay.locations {
            if location.sublocations.is_empty() {
                println!("LOCATION: {:}", location.string);
                let Some(root) = reports::get_location_root_for_node(&screenplay, id) else {
                    panic!();
                };
                let Some(root_node) = screenplay.locations.get(&root) else {
                    panic!()
                };
                println!(
                    "--------- ROOT FOR LEAF: {:?}, | {:?}",
                    root, root_node.string
                )
            }
        }

        println!("");

        for (id, location) in &screenplay.locations {
            if !location.sublocations.is_empty() {
                continue;
            }
            println!("LOCATION_LEAF: {:?}", location.string);
            let Some(path_string) = reports::get_full_string_for_location_path(&screenplay, &id)
            else {
                continue;
            };
            println!("-- FULL PATH FOR LEAF: {:?}\n", path_string);
        }

        println!("\n-----\n");

        if screenplay.characters.is_empty() {
            println!("NO CHARACTERS FOUND!");
        }

        // Test GET CHARACTER...

        println!("\n- GET CHARACTERS PER SCENE:");
        let scenes_ordered = reports::get_all_scenes_ordered(&screenplay).unwrap();
        for (scn_id, scn) in scenes_ordered {
            let Some(characters) = reports::get_characters_for_scene(&screenplay, scn_id) else {
                continue;
            };

            println!(
                "--- SCENE: P:{:<4?}, L:{:<4}",
                scn.start.page, scn.start.line
            );
            for character in characters {
                println!("----- {:?}", character.name)
            }
        }

        println!("\n- GET CHARACTERS PER LOCATION:");
        for (loc_id, loc) in &screenplay.locations {
            let Some(characters) = reports::get_characters_for_location(&screenplay, loc_id) else {
                println!("NO CHARACTERS FOUND AT LOCATION?!");
                continue;
            };

            println!(
                "--- LOCATION: {:?}",
                reports::get_full_string_for_location_path(&screenplay, loc_id)
            );

            for character in characters {
                println!("----- {:?}", character.name)
            }
        }

        println!("\n - GET CHARACTERS PER PAGE:");
        for (pidx, page) in screenplay.pages.iter().enumerate() {
            println!("PAGE INDEX: {:?} | NOMINAL PAGE NUMBER: {:?}", pidx, {
                if let Some(pn) = &page.page_number {
                    pn.to_string()
                } else {
                    "_".to_string()
                }
            });
            let Some(characters) = reports::get_all_characters_on_page_by_index(&screenplay, pidx)
            else {
                println!("No Characters on this page!");
                continue;
            };
            print!("----- ");
            for character in characters {
                print!("{:?} | ", character.name);
            }
            println!("");
        }

        println!("\n\n");

        // Test ALL LINES OF DIALOGUE per Character
        for character in &screenplay.characters {
            println!(
                "CHARACTER ID: {:?} | CHARACTER: {:?}",
                character.id, character.name
            );
            let get_all_char_lines_start = Instant::now();
            let Some(lines) =
                reports::get_all_lines_of_dialogue_for_character(&screenplay, character)
            else {
                //panic!();
                continue;
            };
            let get_all_char_liens_end = get_all_char_lines_start.elapsed();
            println!(
                "TIME TAKEN TO GET ALL DIALOGUE FOR THIS CHARACTER: {:?}",
                get_all_char_liens_end
            );
            println!("LINES OF DIALOGUE FOR CHARACTER: {:?}", lines.len());
            let mut wordcount: usize = 0;
            for (coord, line) in lines {
                let mut line_str = String::new();
                wordcount += line.text_elements.len();
                //println!("WORDS FOR LINE: {:}", line.text_elements.len());
                line.text_elements
                    .iter()
                    .map(|te| te.text.clone())
                    .for_each(|ts| {
                        if !line_str.is_empty() {
                            line_str.push(' ');
                        }
                        line_str.push_str(&ts)
                    });
                //println!("{}",line_str);
            }
            println!("WORDS FOR CHARACTER: {:}", wordcount);
        }

        // Test Get SCENES per Character
        for character in &screenplay.characters {
            let get_scenes_with_char_bench_start = Instant::now();
            let Some(scenes_with_char_speaking) =
                reports::get_all_scenes_with_character_speaking(&screenplay, &character)
            else {
                continue;
            };
            let get_scenes_with_char_bench_end = get_scenes_with_char_bench_start.elapsed();

            println!("CHARACTER: {:?}", character.name);
            println!(
                "\n--TIME TAKEN TO GET ALL SCENES WITH THIS CHARACTER: {:?}\n",
                get_scenes_with_char_bench_end
            );
            println!("--ALL SCENES WITH CHARACTER SPEAKING:");
            for (scn, scene_obj) in &scenes_with_char_speaking {
                let Some(location) = screenplay
                    .locations
                    .get(scene_obj.story_locations.last().unwrap())
                else {
                    continue;
                };
                print!("------");
                println!("{:?} | {:?}", scene_obj.start, location.string)
            }
        }

        // Test GET LOCATIONS

        println!("\n- GET LOCATIONS PER CHARACTER:");
        for character in &screenplay.characters {
            let Some(locations_per_character) =
                reports::get_all_locations_with_character_speaking(&screenplay, &character)
            else {
                println!("No locations for character?!?!");
                continue;
            };
            println!("-- CHARACTER: {:?}", character.name);
            for lc in locations_per_character {
                println!(
                    "----- {:?}",
                    reports::get_full_string_for_location_path(&screenplay, lc)
                );
            }
        }

        println!("\n- GET LOCATIONS PER PAGE:");
        for page_num in 0..=screenplay.pages.len() - 1 {
            println!("PAGE: {}", page_num);
            let Some(locations_on_page) =
                reports::get_all_locations_on_page_by_index(&screenplay, page_num)
            else {
                println!(" NO LOCATIONS ON PAGE????");
                continue;
            };
            for loc in locations_on_page {
                println!(
                    "----- {:?}",
                    reports::get_full_string_for_location_path(&screenplay, loc)
                );
            }
        }

        // Test GET PAGES
        println!("\n - GET PAGES FOR LOCATION:");
        for (l_id, location) in &screenplay.locations {
            let Some(pages) = reports::get_all_pages_for_location(&screenplay, &l_id) else {
                continue;
            };
            println!("--- LOCATION: {:?}", location.string);
            for (pidx, _page) in pages {
                println!("----- {:?}", pidx);
            }
        }

        println!("\n - GET PAGES FOR CHARACTER:");
        for character in &screenplay.characters {
            println!("--- CHARACTER: {:?}", character.name);
            let Some(pages) =
                reports::get_all_pages_for_character_speaking(&screenplay, &character)
            else {
                //panic!();
                continue;
            };
            for (pidx, _page) in pages {
                println!("----- {:?}", pidx)
            }
        }

        // print filtered dialogue...

        let print_filtered_dialogue = false;

        println!("\nLOCATION FILTERED CHARACTER DIALOGUE LINES:");
        for (location_id, location) in &screenplay.locations {
            if !print_filtered_dialogue {
                break;
            }
            println!("\nLOCATION: {:?}", location.string);
            for character in &screenplay.characters {
                use crate::screenplay_document::ScreenplayCoordinate;

                println!("\n-- CHARACTER: {:?}", character.name);
                let Some(lines) =
                    reports::get_all_lines_of_dialogue_for_character(&screenplay, character)
                else {
                    continue;
                };
                let Some(scenes_with_char_speaking) =
                    reports::get_all_scenes_with_character_speaking(&screenplay, &character)
                else {
                    continue;
                };
                let filter_benchmark_start = Instant::now();
                let Some(filtered_scenes) = reports::filter_scenes_by_locations(
                    &screenplay,
                    scenes_with_char_speaking,
                    vec![location_id],
                ) else {
                    println!("----- NO LINES -----");
                    continue;
                };
                for (scn, sceneobj) in &filtered_scenes {
                    let Some(page) = &screenplay.pages.get(sceneobj.start.page) else {
                        panic!("Could not find page.");
                    };
                    let Some(scene_line) = page.lines.get(sceneobj.start.line) else {
                        panic!("Could not find line in this page.");
                    };
                }

                if filtered_scenes.len() == screenplay.scenes.len() {
                    panic!("NO SCENES ACTUALLY FILTERED!");
                }
                let Some(mut filtered_lines) =
                    reports::filter_lines_by_multiple_scenes(&screenplay, &lines, filtered_scenes)
                else {
                    continue; // all lines should categorically be part of SOME scene... unless there's ZERO "proper" scene headings...
                };

                let filter_bench_end = filter_benchmark_start.elapsed();
                println!(
                    "TIME TAKEN TO FILTER DIALOGUE FOR THIS LOCATION: {:?}",
                    filter_bench_end
                );

                let mut wordcount: usize = 0;
                for (coord, f_line) in &filtered_lines {
                    let mut line_str = String::new();
                    wordcount += f_line.text_elements.len();
                    //println!("WORDS FOR LINE: {:}", line.text_elements.len());
                    f_line
                        .text_elements
                        .iter()
                        .map(|te| te.text.clone())
                        .for_each(|ts| {
                            if !line_str.is_empty() {
                                line_str.push(' ');
                            }
                            line_str.push_str(&ts)
                        });
                    println!(
                        "----- {:<40} | PAGE: {:>4} | LINE: {:>4}",
                        line_str, &coord.page, &coord.line
                    );
                }
                print!(
                    "----- LINES OF DIALOGUE FOR CHARACTER: {:?} | ",
                    filtered_lines.len()
                );
                println!("WORDS FOR CHARACTER: {:}", wordcount);
            }
        }

        let print_pages: bool = false;

        //println!("PAGESLEN: {:?}", screenplay.pages.len());
        //for page in &screenplay.pages {
        //    println!("     PAGE: {:?} | LINES: {:?}", page.page_number, page.lines.len())
        //}

        for page in screenplay.pages {
            if !print_pages {
                break;
            }
            println!("PAGE: {:?}", page.page_number);
            for line in page.lines {
                println!(
                    "-----LINE | Y: {:?} | TYPE: {:?} | REVISED: {} | NUM: {:?}",
                    {
                        if let Some(te) = line.text_elements.first() {
                            if let Some(ep) = te.element_position {
                                Some(ep.y)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    },
                    line.line_type,
                    line.revised,
                    line.scene_number
                );
                for elm in line.text_elements {
                    println!(
                        "{:38} | X: {:7.2?} '{}' ",
                        format!("{:?}", elm.element_type),
                        elm.element_position.unwrap().x,
                        elm.text,
                    );
                }
                println!("\n");
            }
        }
        println!("\nTime elapsed: {:?}", start.elapsed());
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

        let mut mock_pdf: pdf_document::PDFDocument = PDFDocument::default();
        let mut new_page = pdf_document::Page::default();

        new_page.lines.push(_create_pdfline_with_word(
            "Action!".to_string(),
            indentations.action,
            None,
        ));
        new_page.lines.push(_create_pdfline_with_word(
            "CHARACTER".to_string(),
            indentations.character,
            None,
        ));
        new_page.lines.push(_create_pdfline_with_word(
            "(wryly)".to_string(),
            indentations.parenthetical,
            None,
        ));
        new_page.lines.push(_create_pdfline_with_word(
            "Dialogue".to_string(),
            indentations.dialogue,
            None,
        ));

        let pn: String = "256ABC.".to_string();

        // Page Number / Page Revision Date
        // Rests at y-height of top margin
        // Is right-aligned to the right-hand margin
        let mut page_num_line = pdf_document::Line::default();
        page_num_line.words.push(_create_pdfword(
            "(26/04/25)".to_string(),
            indentations.character,
            Some(indentations.top),
        ));
        page_num_line.words.push(_create_pdfword(
            pn.clone(),
            (7.5 * 72.0) - (7.2 * pn.len() as f64),
            Some(indentations.top),
        ));

        new_page.lines.push(page_num_line);

        // Revised line
        let mut revised_line = pdf_document::Line::default();

        revised_line.words.push(_create_pdfword(
            "revised_scn".to_string(),
            indentations.action,
            None,
        ));
        revised_line.words.push(_create_pdfword(
            "*".to_string(),
            (7.5 * 72.0) + (7.2 * 2.0),
            None,
        ));
        new_page.lines.push(revised_line);

        //TODO: CONTINUED/MOREs
        // FIXME: How do we handle these?
        // They are part of the document content.
        // Also, we need to let the user pass in custom (MORE)/(CONTINUED) patterns
        // again, for non-english or non-standard support.
        new_page.lines.push(_create_pdfline_with_word(
            "(MORE)".to_string(),
            indentations.parenthetical,
            Some(60.0),
        ));

        // TODO: Scene heading elements
        new_page.lines.push(get_scene_heading_line(
            "INT.",
            "HOUSE - DAY - CONTINUOUS",
            "*46G*",
            &indentations,
        ));

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

        let parsed_doc =
            pdf_parser::get_screenplay_doc_from_pdf_obj(mock_pdf, None, None, None, None).unwrap();

        println!(
            "\n-----\n\nPage number: {:>8} | Rev. label/date(?): {:12} | {}\n",
            format!("{:?}", parsed_doc.pages.first().unwrap().page_number),
            format!("{:?}", parsed_doc.pages.first().unwrap().revision_label),
            format!("{:?}", parsed_doc.pages.first().unwrap().revision_date),
        );

        let lines = &parsed_doc.pages.first().unwrap().lines;

        // TODO: panic!() for each line type that doesn't fully pass
        // this means iterating manually... :<
        for line in lines {
            println!(
                "LT: {:-<70} \nScene Num: {:8} \nRevised: {}",
                if let Some(l_type) = &line.line_type {
                    format!("{:?}", l_type)
                        .strip_prefix("SP_")
                        .unwrap()
                        .to_string()
                } else {
                    format!("{:?}", SPType::NONE)
                },
                if let Some(sc_num) = line.scene_number.clone() {
                    sc_num
                } else {
                    "None".to_string()
                },
                if line.revised { "Y" } else { "N" },
            );
            println!("{:^30}|{:^8}{:^8}|{:^8}", "Element", "x", "y", "Text");
            println!("{:-<58}", "  -");
            //println!("---");
            for el in &line.text_elements {
                println!(
                    "     {:24} | {:.2}, {:.2} | '{}'",
                    if let Some(l_type) = el.element_type.clone() {
                        format!("{:?}", l_type)
                            .strip_prefix("SP_")
                            .unwrap_or(&format!("{:?}", l_type))
                            .to_string()
                    } else {
                        format!("{:?}", SPType::NONE)
                    },
                    el.element_position.unwrap().x,
                    el.element_position.unwrap().y,
                    el.text,
                );
            }
            println!("");
        }

        //TODO: Add SEPARATE test function just for Scene Parsing

        println!("{:#?}", parsed_doc.scenes)
    }

    fn get_scene_heading_line(
        env: &str,
        text: &str,
        scn_num: &str,
        indentations: &ElementIndentationsPoints,
    ) -> pdf_document::Line {
        let mut scene_heading_line = pdf_document::Line::default();

        scene_heading_line
            .words
            .push(_create_pdfword(env.to_string(), indentations.action, None));
        let mut last_word: String = env.to_string();
        let mut last_word_pos: f64 = scene_heading_line.words.last().unwrap().position.x;
        let mut _get_word_with_offset_from_previous = |text: String| {
            //println!("last_word: {}, len_in_pts: {:?}", last_word, last_word.len() as f64 * 7.2);
            let new_x_offset = (last_word.len() as f64 * 7.2) + 7.2 + last_word_pos;

            //println!("offset x pos: {}", new_x_offset,);
            let new_word = _create_pdfword(text.clone(), new_x_offset, None);

            last_word = text.clone();
            last_word_pos = new_x_offset;
            return new_word;
        };

        let scene_heading_words = text.split_whitespace();

        for word in scene_heading_words {
            scene_heading_line
                .words
                .push(_get_word_with_offset_from_previous(word.to_string()));
        }

        scene_heading_line.words.push(_create_pdfword(
            scn_num.to_string(),
            indentations.right,
            None,
        ));

        return scene_heading_line;
    }

    // TODO: Implement and Test FORCING Scene Numbers for scenes that aren't assigned numbers
    //
    #[test]
    fn scene_parsing() {
        let indentations = ElementIndentationsPoints::us_letter_default(&None);

        println!(" ------ Scene Parsing ------ ");
        println!("");

        let mut mock_pdf: pdf_document::PDFDocument = PDFDocument::default();
        let mut new_page = pdf_document::Page::default();

        let scene_heading_line =
            get_scene_heading_line("INT.", "HOUSE - NIGHT - CONTINUOUS", "1A", &indentations);

        new_page.lines.push(scene_heading_line);

        mock_pdf.pages.push(new_page);

        let mut second_page = pdf_document::Page::default();
        let second_line = get_scene_heading_line(
            "EXT.",
            "BASEBALL FIELD - PITCHER'S MOUND - EARLIER - NIGHT",
            "6G",
            &indentations,
        );

        second_page.lines.push(second_line);
        mock_pdf.pages.push(second_page);

        let parsed_doc =
            pdf_parser::get_screenplay_doc_from_pdf_obj(mock_pdf, None, None, None, None);

        println!("{:#?}", parsed_doc);
    }
}

// TODO: move all the gettrs from impl ScreenplayDocument to here...

use std::{collections::{HashMap, HashSet}, time::Instant};

use crate::screenplay_document::{self, SPType, SceneID};

// ------------ Get LOCATIONs...
// TODO: All Get LOCATION funcs should return some kind of Vec of Tuples that includes at least (&LocationID, &Location)

///
/// Returns an Optional Vec of LocationIDs, up to and including this LocationID.
///
pub fn get_full_location_path_for_node<'a>(
    screenplay_doc: &'a crate::screenplay_document::ScreenplayDocument,
    location: &'a screenplay_document::LocationID,
) -> Option<(Vec<&'a screenplay_document::LocationID>)> {
    unimplemented!();
    None
}

pub fn get_full_string_for_location_path(
    screenplay_doc: &crate::screenplay_document::ScreenplayDocument,
    location_leaf_node: &screenplay_document::LocationID,
) -> Option<String> {
    unimplemented!();
    None
}

///
/// Determines if a "location path" exists.
///
/// Returns `None` if nothing matches the root of the path.
///
/// Returns `Some((&LocationID, Vec<String>))` if a partial match is found.
/// The `&LocationID` is the last valid location the path matched.
/// The `Vec<String>` is the remaining subpath, which does not yet exist as `LocationNode`s.
///
/// If the full path exists, the `Vec<String>` will be empty.
///
/// The caller can afterwards handle creating the rest of the Location path,
/// and appending a new LocationID to the sublocations field
///
///
pub fn location_path_exists<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    path: &[String],
) -> Option<(&'a screenplay_document::LocationID, Vec<String>)> {
    if path.is_empty() {
        return None;
    }

    let path_root = &path[0];

    for (id, location) in &screenplay_document.locations {
        if location.string == *path_root {
            if path.len() == 1 {
                return Some((id, Vec::new()));
            }
            if path.len() > 1 && location.sublocations.is_empty() {
                return Some((id, Vec::from(&path[1..])));
            }

            return location.subpath_exists(&id, &path[1..], screenplay_document);
        }
    }

    None
}

pub fn get_location_mutable<'a>(
    screenplay_document: &'a mut screenplay_document::ScreenplayDocument,
    id: &'a screenplay_document::LocationID,
) -> Option<&'a mut screenplay_document::LocationNode> {
    for (existing_id, loc) in &mut screenplay_document.locations {
        if existing_id == id {
            return Some(loc);
        }
    }
    None
}
pub fn get_locations_with_matching_str<'a>(
    screenplay_document: &'a screenplay_document::ScreenplayDocument,
    location_string: &'a String,
) -> Option<Vec<&'a screenplay_document::LocationID>> {
    let mut loc_id_vec: Vec<&screenplay_document::LocationID> = Vec::new();
    for (id, location) in &screenplay_document.locations {
        if location.string == *location_string {
            loc_id_vec.push(id);
        }
    }
    if loc_id_vec.is_empty() {
        return None;
    }
    Some(loc_id_vec)
}

// !--------- TODO: redo these location getters to use a lifetime to return a ref, instead of using cloning

///
/// Gets the Location ID of the root in this location path, from a given LocationID.
///
/// Returns `'None` if the LocationID is invalid.
pub fn get_location_root_for_node<'a>(
    screenplay_document: &'a screenplay_document::ScreenplayDocument,
    location_id: &'a screenplay_document::LocationID,
) -> Option<&'a screenplay_document::LocationID> {
    let location = screenplay_document.locations.get(location_id)?;
    let Some(superlocation) = &location.superlocation else {
        return Some(location_id);
    };
    return get_location_root_for_node(screenplay_document, &superlocation);
}

pub fn get_all_location_leafs<'a>(
    screenplay_document: &'a screenplay_document::ScreenplayDocument,
    location_id: &'a screenplay_document::LocationID,
) -> Option<HashSet<&'a screenplay_document::LocationID>> {
    let Some(location) = screenplay_document.locations.get(location_id) else {
        return None;
    };
    if location.sublocations.is_empty() {
        let mut this_leaf_as_set: HashSet<&screenplay_document::LocationID> = HashSet::new();
        this_leaf_as_set.insert(location_id);
        return Some(this_leaf_as_set);
    }
    let mut location_id_set: HashSet<&screenplay_document::LocationID> = HashSet::new();
    for sublocation_id in &location.sublocations {
        let Some(subset) = get_all_location_leafs(screenplay_document, &sublocation_id) else {
            continue;
        };
        location_id_set.extend(subset);
    }
    if location_id_set.is_empty() {
        return None;
    }
    Some(location_id_set)
}

pub fn get_locations_with_character_speaking<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    character: &'a screenplay_document::Character,
) -> Option<Vec<&'a screenplay_document::LocationID>> {
    let scenes = get_scenes_with_character_speaking(screenplay_document, character)?;
    let mut locations: HashSet<&screenplay_document::LocationID> = HashSet::new();

    scenes
        .iter()
        .filter_map(|s| screenplay_document.scenes.get(s))
        .map(|scn| &scn.story_locations)
        .for_each(|lcv| {
            lcv.iter().for_each(|lc| {
                locations.insert(lc);
            });
        });
    if locations.is_empty() {
        return None;
    }

    Some(locations.iter().copied().collect())
}

pub fn get_locations_on_page_by_idx(
    screenplay_document: &crate::screenplay_document::ScreenplayDocument,
    page: usize,
) -> Option<Vec<&screenplay_document::LocationID>> {
    let mut locations: HashSet<&screenplay_document::LocationID> = HashSet::new();
    let scenes = get_scenes_on_page_by_idx(screenplay_document,page)?;
    scenes
        .iter()
        .filter_map(|s| {
            let Some(scene) = screenplay_document.scenes.get(s) else {
                return None;
            };

            if scene.story_locations.is_empty() {
                return None;
            }

            Some(&scene.story_locations)
        })
        .for_each(|s| {
            s.iter().for_each(|l| {
                locations.insert(l);
            })
        });

    if locations.is_empty() {
        return Some(locations.iter().copied().collect());
    }
    None
}

// ------------ Get COORDINATEs...

// ------------ Get LINEs...
// TODO: All Get LINE funcs should return a tuple (&Line, ScreenplayCoordinate)

// This takes 100 ish microseconds to filter for a single scene, for a script < 20 pages...
// this might be a candidate for optimization in the future...
pub fn filter_lines_by_scene<'a>(
    screenplay_document: &crate::screenplay_document::ScreenplayDocument,
    lines: &HashMap<screenplay_document::ScreenplayCoordinate, &'a screenplay_document::Line>,
    scenes_filter: Vec<&screenplay_document::SceneID>,
) -> Option<HashMap<screenplay_document::ScreenplayCoordinate, &'a screenplay_document::Line>> {
    if scenes_filter.is_empty() {
        //panic!("EMPTY FILTER!");
        return None;
    }
    let bench_time_test = Instant::now();
    let mut scn_filtered: HashMap<screenplay_document::ScreenplayCoordinate, &screenplay_document::Line> = HashMap::new();

    lines
        .iter()
        .filter(|(coord, ln)| {
            let Some(scene_id) = get_scene_id_for_screenplay_coordinate(screenplay_document,&coord) else {
                return false;
            };
            if scenes_filter.contains(&scene_id) {
                return true;
            }
            false
        })
        .for_each(|(coord, line)| {
            scn_filtered.insert(coord.clone(), line);
        });

    if scn_filtered.is_empty() {
        return None;
    }
    if scn_filtered.len() == lines.len() {
        //panic!("DIDN'T FILTER ANY LINES AT ALL!!!");
    }
    let bench_res = bench_time_test.elapsed();
    println!("\nDURATION TO FILTER BY SCENES: {:?}", bench_res);
    Some(scn_filtered)
}

pub fn get_all_lines_of_dialogue_for_character<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    character: &'a screenplay_document::Character,
) -> Option<HashMap<screenplay_document::ScreenplayCoordinate, &'a screenplay_document::Line>> {
    let mut lines_with_coords: HashMap<screenplay_document::ScreenplayCoordinate, &screenplay_document::Line> = HashMap::new();
    //let mut lines: Vec<&Line> = Vec::new();
    let mut is_dialogue = false;

    for (p_index, page) in screenplay_document.pages.iter().enumerate() {
        for (l_index, line) in page.lines.iter().enumerate() {
            if line.line_type != Some(SPType::SP_CHARACTER)
                && line.line_type != Some(SPType::SP_DUAL_CHARACTERS)
                && line.line_type != Some(SPType::SP_DIALOGUE)
                && line.line_type != Some(SPType::SP_DUAL_DIALOGUES)
            {
                continue;
            }
            //println!("MIGHT BE CHARACTER OR DIALOGUE");
            if character.is_line(line) {
                //println!("Oh boy!!!! | {:?} | {:?}", line.line_type, line.text_elements);
                is_dialogue = true;
                continue;
            }
            if !is_dialogue {
                continue;
            }
            match line.line_type {
                Some(SPType::SP_DIALOGUE) | Some(SPType::SP_DUAL_DIALOGUES) => {
                    lines_with_coords.insert(
                        screenplay_document::ScreenplayCoordinate {
                            page: p_index,
                            line: l_index,
                            element: None,
                        },
                        line,
                    );
                }
                _ => {
                    is_dialogue = false;
                    continue;
                }
            }
        }
    }
    if lines_with_coords.is_empty() {
        return None;
    }
    Some(lines_with_coords)
}

pub fn get_line_from_coordinate<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    coordinate: &'a screenplay_document::ScreenplayCoordinate,
) -> Option<&'a screenplay_document::Line> {
    let page = screenplay_document.pages.get(coordinate.page)?;
    page.lines.get(coordinate.line)
}

// ------------ Get CHARACTERS...

pub fn get_characters_for_scene<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    scene_id: &'a screenplay_document::SceneID,
) -> Option<Vec<&'a screenplay_document::Character>> {
    let scn = screenplay_document.scenes.get(scene_id)?;
    let mut current_page = scn.start.page;
    let mut characters_in_scene: HashSet<&screenplay_document::Character> = HashSet::new();
    let current_characters = &screenplay_document.characters;

    'seeking: loop {
        let Some(page) = screenplay_document.pages.get(current_page) else {
            break 'seeking;
        };
        'lines: for (l_idx, line) in page.lines.iter().enumerate() {
            if (current_page, l_idx) < (scn.start.page, scn.start.line) {
                continue 'lines;
            }
            if line.line_type == Some(SPType::SP_SCENE_HEADING(screenplay_document::SceneHeadingElement::Line)) {
                if (current_page, l_idx) > (scn.start.page, scn.start.line) {
                    break 'seeking;
                }
            }
            if line.line_type != Some(SPType::SP_CHARACTER) {
                continue 'lines;
            }
            if characters_in_scene.len() == current_characters.len() {
                break 'seeking;
            }
            for character in current_characters {
                if character.is_line(line) {
                    characters_in_scene.insert(&character);
                }
            }
        }
        current_page += 1;
    }
    if characters_in_scene.is_empty() {
        return None;
    }

    Some(characters_in_scene.iter().copied().collect())
}

pub fn get_characters_for_page(
    screenplay_document: &crate::screenplay_document::ScreenplayDocument,
    page_index: usize,
) -> Option<Vec<&screenplay_document::Character>> {
    let mut characters_on_page: HashSet<&screenplay_document::Character> = HashSet::new();
    let current_characters = &screenplay_document.characters;

    let Some(page) = screenplay_document.pages.get(page_index) else {
        return None;
    };
    'lines: for line in &page.lines {
        if line.line_type != Some(SPType::SP_CHARACTER) {
            continue 'lines;
        }
        if characters_on_page.len() == current_characters.len() {
            break;
        }
        for character in current_characters {
            if character.is_line(line) {
                characters_on_page.insert(&character);
            }
        }
    }

    if characters_on_page.is_empty() {
        return None;
    }

    Some(characters_on_page.iter().copied().collect())
}

pub fn get_characters_for_location<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    location_id: &'a screenplay_document::LocationID,
) -> Option<HashSet<&'a screenplay_document::Character>> {
    let mut characters_at_this_location: HashSet<&screenplay_document::Character> = HashSet::new();
    let scenes_with_location: Vec<(&screenplay_document::SceneID, &screenplay_document::Scene)> = screenplay_document
        .scenes
        .iter()
        .filter(|(id, scn)| scn.story_locations.contains(location_id))
        .collect();

    for (scn_id, scn) in scenes_with_location {
        if characters_at_this_location.len() == screenplay_document.characters.len() {
            break;
        }
        let Some(characters_in_this_scene) = get_characters_for_scene(screenplay_document,scn_id) else {
            return None; // something went very wrong...,
        };
        characters_at_this_location.extend(characters_in_this_scene);
    }
    if characters_at_this_location.is_empty() {
        return None;
    }

    Some(characters_at_this_location.iter().copied().collect())
}

// ------------ Get SCENES...
// TODO: All Get SCENE funcs should return a tuple (&SceneID, &Scene, ScreenplayCoordinate)

pub fn filter_scenes_by_locations<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    scene_ids: Vec<&'a screenplay_document::SceneID>,
    locations: Vec<&'a screenplay_document::LocationID>,
) -> Option<Vec<&'a screenplay_document::SceneID>> {
    let mut filtered: Vec<&screenplay_document::SceneID> = Vec::new();
    for loc in locations {
        let Some(scenes_for_loc) = get_scenes_with_location(screenplay_document,loc) else {
            panic!("COULDN'T FIND SCENES WITH THIS LOCATION?!");
            continue;
        };

        for scene in scenes_for_loc {
            //println!("----- FILTERING BY LOCATION-SCENE....");
            if scene_ids.contains(&scene) && !filtered.contains(&scene) {
                filtered.push(scene);
            }
        }
    }
    if filtered.is_empty() {
        return None;
    }
    Some(filtered)
}

/// Gets a Vec of all `SceneID`s in the document, sorted by document order.
///
/// # Example
///
/// ```rust
/// use screenplay_doc_parser_rs::screenplay_document::*;
/// let mut doc = ScreenplayDocument::new();
/// let scene1: Scene = Scene {
///     start: ScreenplayCoordinate {page: 0 as usize, line: 10 as usize, element: None},
///     number: None,
///     revised: false,
///     environment: Environment::Int,
///     story_location: Location::default(),
///     story_sublocation: None,
///     story_time_of_day: None
/// };
/// let scene2: Scene = Scene {
///     start: ScreenplayCoordinate {page: 1 as usize, line: 5 as usize, element: None},
///     number: None,
///     revised: false,
///     environment: Environment::Int,
///     story_location: Location::default(),
///     story_sublocation: None,
///     story_time_of_day: None
/// };
/// let id_1 = SceneID::new();
/// let id_2 = SceneID::new();
/// doc.scenes.insert(id_2.clone(), scene2);
/// doc.scenes.insert(id_1.clone(), scene1);
///
/// let sorted = doc.get_all_scenes_sorted().unwrap();
///
/// assert_eq!(**sorted.get(0).unwrap(), id_1);
/// assert_eq!(**sorted.get(1).unwrap(), id_2)
/// ```
pub fn get_all_scenes_ordered(
    screenplay_document: &crate::screenplay_document::ScreenplayDocument,
) -> Option<Vec<&screenplay_document::SceneID>> {
    if screenplay_document.scenes.len() == 0 {
        return None;
    }
    if screenplay_document.scenes.len() == 1 {
        return Some(screenplay_document.scenes.keys().collect());
    }
    let mut scene_ids: Vec<_> = screenplay_document.scenes.keys().collect();

    scene_ids.sort_by(|a, b| {
        let scn_a = screenplay_document.scenes.get(a).unwrap();
        let scn_b = screenplay_document.scenes.get(b).unwrap();

        (scn_a.start.page, scn_a.start.line).cmp(&(scn_b.start.page, scn_b.start.line))
    });
    return Some(scene_ids);
}

pub fn get_scene_id_for_screenplay_coordinate<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    checked_coordinate: &screenplay_document::ScreenplayCoordinate,
) -> Option<&'a screenplay_document::SceneID> {
    let Some(page) = screenplay_document.pages.get(checked_coordinate.page) else {
        //println!("DUCK SAUCE 2: HELP!");
        return None;
    };

    // lines are properly enumerated FORWARDS, then ENTIRE ITERATOR is reversed
    for (line_index, line) in page.lines.iter().enumerate().rev() {
        // if this line is EQUAL or EARLIER THAN the coordinate...
        if line_index > checked_coordinate.line {
            continue;
        }
        let Some(SPType::SP_SCENE_HEADING(screenplay_document::SceneHeadingElement::Line)) = line.line_type else {
            continue;
        };
        let Some(scn_id) = &line.scene_id else {
            //println!("SCENE HEADING HAS NO SCENE ID!");
            //panic!();
            continue;
        };
        //println!("SUCCESS PART ONE...?");
        return Some(scn_id);
    }
    // couldn't find the scene on this page. try the previous page...
    // recursively check all previous pages

    let Some(previous_page_idx) = checked_coordinate.page.checked_sub(1) else {
        return None;
    };
    let Some(previous_page) = screenplay_document.pages.get(previous_page_idx) else {
        return None;
    };
    let Some(last_line_idx) = previous_page.lines.len().checked_sub(1) else {
        return None;
    };
    let last_page_last_line_coord = screenplay_document::ScreenplayCoordinate {
        page: previous_page_idx,
        line: last_line_idx,
        element: None,
    };
    //println!("CHECKING RECURSIVELY FOR SCENE!");
    //println!("COORDS OF PREVIOUS_PAGE: {:?}", last_page_last_line_coord);

    let Some(id_opt) = get_scene_id_for_screenplay_coordinate(screenplay_document,&last_page_last_line_coord)
    else {
        //println!("RECURSIVE CHECK FAILED!");
        //panic!("DUCK SAUCE");
        return None;
    };
    //println!("SUCCESS PART TWO...?");
    return Some(id_opt);
}

// This may be a contender for optimization....
pub fn get_scene_ids_from_range<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    start: & screenplay_document::ScreenplayCoordinate,
    end: & screenplay_document::ScreenplayCoordinate,
) -> Option<Vec<&'a screenplay_document::SceneID>> {
    if (screenplay_document.pages.get(start.page)).is_none() {
        return None;
    }
    if screenplay_document.pages.get(end.page).is_none() {
        return None;
    }

    let mut scenes: Vec<&screenplay_document::SceneID> = Vec::new();
    for page_index in start.page..=end.page {
        let Some(page) = screenplay_document.pages.get(page_index) else {
            continue;
        };

        for (l_idx, line) in page.lines.iter().enumerate() {
            if page_index == start.page && l_idx < start.line {
                continue;
            } else if page_index == end.page && l_idx > end.line {
                break;
            }
            // TODO: keep track of CURRENT line type and PREVIOUS linetype
            // if PREV line was a SCENE HEADING, just continue
            // if CURRENT LINE is NOT a scene heading, AND we already HAVE an entry in the scenes vec, CONTINUE
            if Some(SPType::SP_SCENE_HEADING(screenplay_document::SceneHeadingElement::Line)) != line.line_type
                && !scenes.is_empty()
            {
                continue;
            }
            let Some(scene_id) =
                get_scene_id_for_screenplay_coordinate(screenplay_document,&screenplay_document::ScreenplayCoordinate {
                    page: page_index,
                    line: l_idx,
                    element: None,
                })
            else {
                continue;
            };
            if !scenes.contains(&scene_id) {
                scenes.push(scene_id);
            }
        }
    }
    if scenes.is_empty() {
        return None;
    }

    Some(scenes)
}

pub fn get_scenes_from_ids<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    ids: Vec<&'a screenplay_document::SceneID>,
) -> Option<Vec<&'a screenplay_document::Scene>> {
    let mut scenes: Vec<&screenplay_document::Scene> = Vec::new();
    for id in ids {
        let scene = screenplay_document.scenes.get(id)?;
        scenes.push(scene);
    }
    if scenes.is_empty() {
        return None;
    }

    Some(scenes)
}

// I wrote this func but not sure what to use it for...?
// Might remove later if not useful in the future
pub fn get_scenes_with_scene_heading_element<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    heading_element: &'a screenplay_document::SceneHeadingElement,
) -> Option<Vec<&'a screenplay_document::SceneID>> {
    let mut scene_ids: Vec<&screenplay_document::SceneID> = Vec::new();

    for (scene_id, scene) in &screenplay_document.scenes {
        let Some(scene_heading_line) = get_line_from_coordinate(screenplay_document, &scene.start) else {
            continue;
        };
        for element in &scene_heading_line.text_elements {
            let Some(eltype) = &element.element_type else {
                continue;
            };
            let SPType::SP_SCENE_HEADING(sch) = eltype else {
                continue;
            };

            if sch == heading_element {
                scene_ids.push(scene_id);
            }
        }
    }

    if scene_ids.is_empty() {
        return None;
    }
    Some(scene_ids)
}

pub fn get_scenes_on_page_by_idx(
    screenplay_document: &crate::screenplay_document::ScreenplayDocument,
    page_index: usize,
) -> Option<Vec<&screenplay_document::SceneID>> {
    let page = screenplay_document.pages.get(page_index)?;
    if page.lines.is_empty() {
        return None;
    }
    let last_line_idx = page.lines.len().checked_sub(1).unwrap_or(0);
    let start = screenplay_document::ScreenplayCoordinate {
        page: page_index,
        line: 0,
        element: None,
    };
    let end = screenplay_document::ScreenplayCoordinate {
        page: page_index,
        line: last_line_idx,
        element: None,
    };
    let Some(scenes) = get_scene_ids_from_range(screenplay_document, &start, &end) else {
        return None;
    };

    let collected: Vec<&SceneID> = scenes.iter().copied().collect();

    Some(collected)
}

// this one seems inefficient....
// could skip lines of loop if we already pushed the current valid scene, and skip
// to the first line of the next scene to check each time...
pub fn get_scenes_with_character_speaking<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    character: &screenplay_document::Character,
) -> Option<Vec<&'a screenplay_document::SceneID>> {
    let mut latest_scene_id: &Option<screenplay_document::SceneID> = &None;
    let mut next_scene_id: &Option<screenplay_document::SceneID> = &None;
    let Some(scenes_sorted) = get_all_scenes_ordered(screenplay_document) else {
        return None;
    };

    let mut scenes_vec: Vec<&screenplay_document::SceneID> = Vec::new();
    for (p_idx, page) in screenplay_document.pages.iter().enumerate() {
        for (l_idx, line) in page.lines.iter().enumerate() {
            if line.scene_id.is_some() {
                latest_scene_id = &line.scene_id;
            }

            if character.is_line(&line) {
                let Some(sceneid) = latest_scene_id else {
                    continue;
                };
                if !scenes_vec.contains(&sceneid) {
                    scenes_vec.push(sceneid);
                }
            }
        }
    }
    if scenes_vec.is_empty() {
        return None;
    } else {
        return Some(scenes_vec);
    }
}

pub fn get_scenes_with_location<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    location: &'a screenplay_document::LocationID,
) -> Option<Vec<&'a screenplay_document::SceneID>> {
    let scenes: Vec<&screenplay_document::SceneID> = screenplay_document
        .scenes
        .iter()
        .filter(|(_, scn)| scn.story_locations.contains(&location))
        .map(|(id, _)| id)
        .collect();

    if scenes.is_empty() {
        let location_opt = screenplay_document.locations.get(&location);
        println!("LOCATION: {:?}", location_opt);
        //panic!("COULDN'T FIND SCENES FOR LOCATION!");
        return None;
    }
    Some(scenes)
}

// ------------ Get PAGEs...
// TODO: all get PAGE funcs should return a Vec of tuples (usize, &Page)
pub fn get_pages_for_scene<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    scene_id: &'a screenplay_document::SceneID,
) -> Option<Vec<(usize, &'a screenplay_document::Page)>> {
    let checked_scene = screenplay_document.scenes.get(scene_id)?;

    let scenes_ordered = get_all_scenes_ordered(screenplay_document)?;
    //pages.push(checked_scene.start.page.clone());

    for this_id in scenes_ordered {
        let Some(scene_obj) = screenplay_document.scenes.get(this_id) else {
            continue; // ERROR -- Tried to find a scene by ID that DOESN'T EXIST in the current script
        };
        if scene_obj.start.page < checked_scene.start.page
            || (scene_obj.start.page == checked_scene.start.page
                && scene_obj.start.line < checked_scene.start.line)
        {
            continue;
        }
        let page_indecies: Vec<usize> =
            (checked_scene.start.page.clone()..=scene_obj.start.page.clone()).collect();

        let mut pages: Vec<(usize, &screenplay_document::Page)> = Vec::new();
        for pi in page_indecies {
            let Some(page) = screenplay_document.pages.get(pi) else {
                continue;
            };
            pages.push((pi.clone(), page));
        }
        if pages.is_empty() {
            return None;
        }
        return Some(pages);
    }

    None
}

pub fn get_pages_for_scenes<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    scene_ids: Vec<&screenplay_document::SceneID>,
) -> Option<Vec<(usize, &'a screenplay_document::Page)>> {
    let scenes_ordered = &get_all_scenes_ordered(screenplay_document)?;

    let mut all_pages: Vec<(usize, &screenplay_document::Page)> = Vec::new();

    for scene_id in scene_ids {
        let checked_scene = screenplay_document.scenes.get(scene_id)?;

        //pages.push(checked_scene.start.page.clone());

        for this_id in scenes_ordered {
            let Some(scene_obj) = screenplay_document.scenes.get(this_id) else {
                continue; // ERROR -- Tried to find a scene by ID that DOESN'T EXIST in the current script
            };
            if scene_obj.start.page < checked_scene.start.page
                || (scene_obj.start.page == checked_scene.start.page
                    && scene_obj.start.line < checked_scene.start.line)
            {
                continue;
            }
            let page_indecies: Vec<usize> =
                (checked_scene.start.page.clone()..=scene_obj.start.page.clone()).collect();

            let mut pages: Vec<(usize, &screenplay_document::Page)> = Vec::new();
            for pi in page_indecies {
                let Some(page) = screenplay_document.pages.get(pi) else {
                    continue;
                };
                pages.push((pi.clone(), page));
            }
            if pages.is_empty() {
                continue;
            }
            for (idx, page) in pages {
                if !all_pages.contains(&(idx, page)) {
                    all_pages.push((idx, page));
                }
            }
        }
    }
    if all_pages.is_empty() {
        return None;
    }

    Some(all_pages)
}

pub fn get_pages_for_location<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    location: &'a screenplay_document::LocationID,
) -> Option<Vec<(usize, &'a screenplay_document::Page)>> {
    let Some(scenes_ordered) = get_all_scenes_ordered(screenplay_document) else {
        return None;
    };
    let scenes_filtered = filter_scenes_by_locations(screenplay_document,scenes_ordered, vec![location])?;

    return get_pages_for_scenes(screenplay_document,scenes_filtered);
}

// ... this func may not be necessary, or is otherwise way too gross and like idek
pub fn get_pages_for_character<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    character: &'a screenplay_document::Character,
) -> Option<Vec<(usize, &'a screenplay_document::Page)>> {
    let mut pages: Vec<(usize, &screenplay_document::Page)> = Vec::new();

    'pages: for (idx, page) in screenplay_document.pages.iter().enumerate() {
        for ln in &page.lines {
            if character.is_line(ln) {
                //panic!("WHAT?");
                pages.push((idx, page));
                continue 'pages;
            }
        }
    }

    if pages.is_empty() {
        return None;
    }

    Some(pages)
}
